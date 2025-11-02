//! Minimal CLI verifier (v2 format)
//!
//! Reads a strict, versioned proof file:
//!   magic: b"SSZKPv2\0" (8 bytes) + u16 version (=2) + ark-compressed `Proof`
//!
//! Updates in this revision (format unchanged):
//! - **SRS digest enforcement**: compare loaded SRS (G1/G2) digests against the
//!   proof header and error clearly on mismatch.
//! - **Header authority**: the verifier *trusts the proof header* for domain
//!   parameters. Any `--zh-c` CLI flag is politely ignored (we print a note).
//! - **Basis override policy**: the **header’s wire basis** is used. If the CLI
//!   provided `--basis`, we warn on divergence and proceed with the header basis.
//! - **Feature-aware shape checks**: expected openings are computed in a way that
//!   matches feature flags (e.g., `zeta-shift` adds Z@ω·ζ).
//! - Delegation to `scheduler::Verifier` is unchanged; this wrapper only handles
//!   IO, basic shape sanity, and environment/header consistency.

#![forbid(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::{env, fs, io::Read, path::Path};

use ark_ff::{fields::Field, FftField, One, Zero};
use ark_serialize::CanonicalDeserialize;
use myzkp::{
    domain::{self, domain_digest},
    pcs::{self, Basis, PcsParams},
    scheduler::Verifier,
    VerifyParams, F,
};

// 8-byte magic: "SSZKPv2" + NUL terminator to match the 8-byte read/write.
const FILE_MAGIC: &[u8; 8] = b"SSZKPv2\0";
const FILE_VERSION_SUPPORTED: u16 = 2;

fn parse_flag(args: &[String], key: &str) -> Option<String> {
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == key {
            return it.next().cloned();
        }
    }
    None
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Optional CLI hint for wires basis — for UX only. The header is authoritative.
    let basis_str = parse_flag(&args, "--basis").unwrap_or_else(|| "eval".to_string());
    let basis_wires_cli = match basis_str.as_str() {
        "coeff" | "coefficient" => Basis::Coefficient,
        _ => Basis::Evaluation,
    };

    // Users may pass --zh-c out of habit; make it explicit we ignore it.
    if let Some(cli_zh) = parse_flag(&args, "--zh-c") {
        eprintln!("note: ignoring CLI --zh-c={}; verifier uses zh_c from the proof header.", cli_zh);
    }

    // Optional proof file path (default: proof.bin)
    let proof_path = parse_flag(&args, "--proof")
        .unwrap_or_else(|| "proof.bin".to_string());

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

    // --- Read versioned proof file ---
    let mut file = fs::File::open(&proof_path)
        .map_err(|e| anyhow::anyhow!("open {}: {e}", proof_path))?;
    let mut magic = [0u8; 8];
    file.read_exact(&mut magic)?;
    if &magic != FILE_MAGIC {
        return Err(anyhow::anyhow!("bad proof file: missing magic header"));
    }
    let mut ver_bytes = [0u8; 2];
    file.read_exact(&mut ver_bytes)?;
    let file_ver = u16::from_be_bytes(ver_bytes);
    if file_ver != FILE_VERSION_SUPPORTED {
        return Err(anyhow::anyhow!(
            "unsupported proof version: got {}, support {}",
            file_ver, FILE_VERSION_SUPPORTED
        ));
    }
    let mut payload = Vec::new();
    file.read_to_end(&mut payload)?;
    let mut slice = payload.as_slice();
    let proof: myzkp::Proof = CanonicalDeserialize::deserialize_compressed(&mut slice)
        .map_err(|e| anyhow::anyhow!("deserialize proof: {}", e))?;

    // --- Sanity-check header vs loaded environment ---
    // SRS digests are the *only* binding between proof and locally loaded SRS.
    let srs_g1_d = pcs::srs_g1_digest();
    let srs_g2_d = pcs::srs_g2_digest();
    if proof.header.srs_g1_digest != srs_g1_d {
        return Err(anyhow::anyhow!("SRS G1 digest mismatch vs proof header"));
    }
    if proof.header.srs_g2_digest != srs_g2_d {
        return Err(anyhow::anyhow!("SRS G2 digest mismatch vs proof header"));
    }

    // Domain from the header (authoritative). We do not accept CLI overrides.
    let domain = myzkp::domain::Domain {
        n: proof.header.domain_n as usize,
        omega: proof.header.domain_omega,
        zh_c: proof.header.zh_c,
    };
    let dom_digest = domain_digest(&domain);

    // PCS params: prefer the **header basis** for wires; warn on CLI divergence.
    let basis_wires = proof.header.basis_wires;
    if basis_wires != basis_wires_cli {
        eprintln!(
            "note: ignoring CLI --basis={:?}; using header basis={:?}",
            basis_wires_cli, basis_wires
        );
    }
    let pcs_wires = PcsParams { max_degree: domain.n - 1, basis: basis_wires, srs_placeholder: () };
    let pcs_coeff = PcsParams { max_degree: domain.n - 1, basis: Basis::Coefficient, srs_placeholder: () };

    // Human-friendly summary for quick inspection.
    eprintln!(
        "header: N={}, k={}, zh_c={}, basis_wires={:?}",
        proof.header.domain_n, proof.header.k, proof.header.zh_c, basis_wires
    );
    eprintln!("domain digest (header): {:02x?}", dom_digest);

    // --------------------- Shape sanity checks (feature-aware) ---------------------
    let k = proof.wire_comms.len();
    let s = proof.eval_points.len();
    let has_z = proof.z_comm.is_some();

    // Base expected items: [wires@ζ] + [Z@ζ?] + [Q@ζ]
    let mut expected_items = (k + usize::from(has_z)) * s + /* Q */ s;

    // If the binary is compiled with zeta-shift support, expect Z@ω·ζ as well.
    #[cfg(feature = "zeta-shift")]
    {
        if has_z {
            expected_items += s;
        }
    }

    if proof.opening_proofs.len() != expected_items || proof.evals.len() != expected_items {
        return Err(anyhow::anyhow!(
            "proof shape mismatch: k={}, s={}, has_z={}, expected items={}, got evals={}, proofs={}",
            k, s, has_z, expected_items, proof.evals.len(), proof.opening_proofs.len()
        ));
    }

    // --------------------- Delegate to protocol verifier ---------------------
    let verify_params = VerifyParams { domain: domain.clone(), pcs_wires, pcs_coeff };
    let verifier = Verifier { params: &verify_params };

    // Replay Fiat–Shamir and enforce pairings via the scheduler.
    verifier.verify(&proof).map_err(|e| anyhow::anyhow!("verification failed: {e}"))?;

    println!("Verifier result: ok");
    Ok(())
}
