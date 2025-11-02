//! Minimal CLI prover (v2 format)
//!
//! Writes a strict, versioned proof file:
//!   magic: b"SSZKPv2\0" (8 bytes) + u16 version (=2) + ark-compressed `Proof`
//!
//! Updates in this revision (IO is unchanged):
//! - **Fast ω/N checks** via exponentiation-by-squaring (no naive loops).
//! - `--zh-c` continues to select the coset vanishing constant in Z_H(X)=X^N−c.
//! - **Robust CSV selectors** loader: ragged-row detection, comments, clearer errors.
//! - Human-friendly diagnostics: domain/SRS digests and header summary.

#![forbid(unsafe_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_imports)]

use std::{env, fs, io::Write, path::Path};

use ark_ff::{fields::Field, FftField, One, Zero};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use myzkp::{
    air::{AirSpec, Row},
    domain::{self, domain_digest},
    pcs::{self, Basis, PcsParams},
    scheduler::Prover,
    F, ProveParams,
};

/// 8-byte magic: "SSZKPv2" + NUL terminator to match the 8-byte read/write.
const FILE_MAGIC: &[u8; 8] = b"SSZKPv2\0";
const FILE_VERSION: u16 = 2;

fn parse_flag(args: &[String], key: &str) -> Option<String> {
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == key {
            return it.next().cloned();
        }
    }
    None
}
fn parse_bool(s: &str) -> bool {
    matches!(s, "1" | "true" | "True" | "TRUE" | "yes" | "y")
}
fn parse_u64(s: &str) -> Option<u64> {
    s.parse::<u64>().ok()
}
fn next_power_of_two(mut n: usize) -> usize {
    if n == 0 { return 1; }
    n -= 1;
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;
    n |= n >> 32;
    n + 1
}

/// Fast pow: exponentiation-by-squaring on the field.
#[inline]
fn pow_u64(mut base: F, mut exp: u64) -> F {
    let mut acc = F::one();
    while exp > 0 {
        if exp & 1 == 1 { acc *= base; }
        base.square_in_place();
        exp >>= 1;
    }
    acc
}

/// Very small CSV-ish loader for selectors/fixed columns.
///
/// - Splits on commas **or** whitespace.
/// - Ignores empty tokens and inline comments after `#`.
/// - Ensures all rows have the **same** number of columns (ragged = error).
/// - Each *column* is one selector polynomial over all rows.
/// - Returned shape: `Vec<Box<[F]>>` with length **S** (column-major).
fn load_selectors_csv(path: &Path) -> anyhow::Result<Vec<Box<[F]>>> {
    let text = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("read selectors file {}: {e}", path.display()))?;

    let mut rows: Vec<Vec<F>> = Vec::new();
    for (lineno, line_raw) in text.lines().enumerate() {
        let mut line = line_raw.trim();
        if let Some(hash) = line.find('#') {
            line = &line[..hash];
        }
        if line.trim().is_empty() {
            continue;
        }

        let mut row_vals = Vec::new();
        for (colno, tok) in line
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|t| !t.is_empty())
            .enumerate()
        {
            // Accept decimal u128; callers can pre-scale if needed.
            let v = tok.parse::<u128>().map_err(|e| {
                anyhow::anyhow!(
                    "selectors parse error at line {}, column {}: token `{}` ({})",
                    lineno + 1,
                    colno + 1,
                    tok,
                    e
                )
            })?;
            row_vals.push(F::from(v as u64));
        }
        if !row_vals.is_empty() {
            rows.push(row_vals);
        }
    }

    if rows.is_empty() {
        // No selectors provided → treat as “no fixed columns”.
        return Ok(Vec::new());
    }

    // Ensure all rows have the same number of columns.
    let s_cols = rows[0].len();
    for (i, r) in rows.iter().enumerate() {
        if r.len() != s_cols {
            // Provide precise diagnostic to help fix CSV quickly.
            return Err(anyhow::anyhow!(
                "selectors file is ragged: row 1 has {} column(s), but row {} has {}. \
                 Please ensure a rectangular matrix; use 0s if needed.",
                s_cols,
                i + 1,
                r.len()
            ));
        }
    }

    // Transpose to column-major.
    let mut cols: Vec<Vec<F>> = vec![Vec::with_capacity(rows.len()); s_cols];
    for r in rows {
        for (j, v) in r.into_iter().enumerate() {
            cols[j].push(v);
        }
    }
    Ok(cols.into_iter().map(|v| v.into_boxed_slice()).collect())
}

/// Minimal ω sanity check for power-of-two N (fast pow).
fn validate_domain_params(n: usize, omega: F, zh_c: F) -> anyhow::Result<()> {
    if n == 0 {
        return Err(anyhow::anyhow!("domain size N must be positive"));
    }
    if zh_c.is_zero() {
        return Err(anyhow::anyhow!("zh_c must be non-zero (Z_H(X)=X^N - zh_c)"));
    }
    // ω^N == 1
    if pow_u64(omega, n as u64) != F::one() {
        return Err(anyhow::anyhow!("omega^N != 1; invalid subgroup generator"));
    }
    // ω is primitive: ω^{N/2} != 1 (only meaningful if N >= 2)
    if n >= 2 && pow_u64(omega, (n as u64) / 2) == F::one() {
        return Err(anyhow::anyhow!("omega does not have exact order N (omega^(N/2) == 1)"));
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    let n_rows: usize = parse_flag(&args, "--rows").and_then(|s| s.parse().ok()).unwrap_or(1024);
    let b_blk: usize = parse_flag(&args, "--b-blk").and_then(|s| s.parse().ok()).unwrap_or(128);
    let k_regs: usize = parse_flag(&args, "--k").and_then(|s| s.parse().ok()).unwrap_or(3);
    let basis_str = parse_flag(&args, "--basis").unwrap_or_else(|| "eval".to_string());
    
    // Optional proof output path (default: proof.bin)
    let proof_output = parse_flag(&args, "--proof-output")
        .or_else(|| parse_flag(&args, "--proof"))  // Accept both forms
        .unwrap_or_else(|| "proof.bin".to_string());
    let basis_wires = match basis_str.as_str() {
        "coeff" | "coefficient" => Basis::Coefficient,
        _ => Basis::Evaluation,
    };
    let commit_z = parse_flag(&args, "--commit-z").map(|s| parse_bool(&s)).unwrap_or(true);

    // CLI-selectable Z_H(X)=X^N − zh_c (default 1)
    let zh_c_str = parse_flag(&args, "--zh-c").unwrap_or_else(|| "1".into());
    let zh_c = F::from(
        zh_c_str
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("--zh-c must be a u64 (got `{}`)", zh_c_str))?,
    );

    // Optional: load selector/fixed columns
    let selectors: Vec<Box<[F]>> = if let Some(p) = parse_flag(&args, "--selectors") {
        let path = Path::new(&p);
        eprintln!("loading selectors from {}", path.display());
        let cols = load_selectors_csv(path)?;
        eprintln!("loaded {} selector column(s)", cols.len());
        cols
    } else {
        Vec::new()
    };

    // --- Domain (with optional omega) ---
    let n_domain = next_power_of_two(n_rows);
    let omega_override = parse_flag(&args, "--omega").and_then(|s| parse_u64(&s));

    let omega = if let Some(u) = omega_override {
        F::from(u)
    } else {
        F::get_root_of_unity(n_domain as u64)
            .expect("field does not support an N-th root of unity for this N")
    };

    // Subgroup domain => Z_H(X) = X^N − zh_c (CLI-selectable)
    validate_domain_params(n_domain, omega, zh_c)?;

    let domain = myzkp::domain::Domain { n: n_domain, omega, zh_c };
    let dom_digest = domain_digest(&domain);

    // ---------------- SRS loading (G1 + G2 required in non-dev) ----------------
    let srs_g1_path = parse_flag(&args, "--srs-g1");
    let srs_g2_path = parse_flag(&args, "--srs-g2");

    #[cfg(feature = "dev-srs")]
    {
        if srs_g1_path.is_none() || srs_g2_path.is_none() {
            eprintln!("(dev-srs) Using deterministic in-crate SRS. For production, pass --srs-g1/--srs-g2.");
        }
    }
    #[cfg(not(feature = "dev-srs"))]
    {
        if srs_g1_path.is_none() || srs_g2_path.is_none() {
            return Err(anyhow::anyhow!(
                "Non-dev build: --srs-g1 and --srs-g2 are REQUIRED for trusted KZG verification."
            ));
        }
    }

    if let Some(p) = srs_g1_path {
        let bytes = fs::read(Path::new(&p)).map_err(|e| anyhow::anyhow!("read G1 SRS: {e}"))?;
        let mut slice = bytes.as_slice();
        let g1_powers: Vec<ark_bn254::G1Affine> =
            CanonicalDeserialize::deserialize_compressed(&mut slice)
                .map_err(|e| anyhow::anyhow!("deserialize G1 SRS: {e}"))?;
        pcs::load_srs_g1(&g1_powers);
        eprintln!("loaded G1 SRS ({} powers) from {}", g1_powers.len(), p);
    }
    if let Some(p) = srs_g2_path {
        let bytes = fs::read(Path::new(&p)).map_err(|e| anyhow::anyhow!("read G2 SRS: {e}"))?;
        let mut slice = bytes.as_slice();
        let g2_powers: Vec<ark_bn254::G2Affine> =
            CanonicalDeserialize::deserialize_compressed(&mut slice)
                .map_err(|e| anyhow::anyhow!("deserialize G2 SRS: {e}"))?;
        // Accept either [1]G2,[τ]G2 or just [τ]G2; pick a [τ] element.
        let tau_g2 = *g2_powers
            .get(1)
            .or_else(|| g2_powers.get(0))
            .ok_or_else(|| anyhow::anyhow!("G2 SRS file must contain at least one element"))?;
        pcs::load_srs_g2(tau_g2);
        eprintln!("loaded G2 SRS ({} elements) from {}", g2_powers.len(), p);
    }

    // Digests (stable, length-prefixed hash in pcs/domain modules)
    let srs_g1_d = pcs::srs_g1_digest();
    let srs_g2_d = pcs::srs_g2_digest();
    eprintln!("domain digest: {:02x?}", dom_digest);
    eprintln!("SRS  G1 dig:   {:02x?}", srs_g1_d);
    eprintln!("SRS  G2 dig:   {:02x?}", srs_g2_d);

    // --- Build AIR, PCS params ---
    let air = AirSpec { k: k_regs, id_table: Vec::new(), sigma_table: Vec::new(), selectors };

    // Keep PCS shapes identical to previous build; wires basis selectable.
    let pcs_wires = PcsParams { max_degree: n_domain - 1, basis: basis_wires, srs_placeholder: () };
    let pcs_coeff = PcsParams { max_degree: n_domain - 1, basis: Basis::Coefficient, srs_placeholder: () };

    let prove_params = ProveParams { domain: domain.clone(), pcs_wires, pcs_coeff, b_blk };

    // Non-trivial witness (deterministic)
    let witness_rows: Vec<Row> = (0..n_rows)
        .map(|i| {
            let mut regs = vec![F::from(0u64); k_regs];
            let base = F::from((i as u64) + 1);
            for m in 0..k_regs {
                regs[m] = base.pow([(m as u64) + 1]);
            }
            Row { regs: regs.into_boxed_slice() }
        })
        .collect();

    // --- Run scheduler.Prover (restreaming path kept intact) ---
    let prover = Prover { air: &air, params: &prove_params };
    let proof = prover.prove_with_restreamer(&witness_rows)
        .map_err(|e| anyhow::anyhow!("prover failed: {e}"))?;

    // Header quick summary for humans (single concise line).
    eprintln!(
        "header: N={}, k={}, ω^N=1 ✓, ω^(N/2)≠1 ✓, zh_c={}, basis_wires={:?}",
        proof.header.domain_n, proof.header.k, proof.header.zh_c, proof.header.basis_wires
    );

    // --- Emit versioned proof file (magic + version + ark-compressed Proof) ---
    let mut payload = Vec::new();
    proof.serialize_compressed(&mut payload)
        .map_err(|e| anyhow::anyhow!("serialize proof: {e}"))?;

    let mut f = fs::File::create(&proof_output)
        .map_err(|e| anyhow::anyhow!("create {}: {e}", proof_output))?;
    f.write_all(FILE_MAGIC)?;
    f.write_all(&FILE_VERSION.to_be_bytes())?;
    f.write_all(&payload)?;
    f.flush().ok();

    eprintln!("wrote {} (v{}, {} bytes payload)", proof_output, FILE_VERSION, payload.len());
    Ok(())
}
