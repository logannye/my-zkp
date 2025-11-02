//! tinyzkp_api: a minimal REST façade for the sublinear-space ZKP prover/verifier with
//! usage-based pricing, API key auth, and Redis-backed account management.
//!
//! Public endpoints (JSON unless noted):
//! - GET  /v1/health
//! - GET  /v1/version
//! - POST /v1/domain/plan          { rows, b_blk?, zh_c? } -> N, b_blk_hint, omega_ok, mem_hint
//! - POST /v1/pricing/estimate     { rows, registers } -> cost breakdown + examples (NEW)
//! - POST /v1/auth/signup          { email, password } -> { user_id, api_key, session_token }
//! - POST /v1/auth/login           { email, password } -> { user_id, api_key, session_token }
//! - GET  /v1/me                   (Authorization: Bearer <session>) -> account balance + usage
//! - POST /v1/keys/rotate          (Authorization: Bearer <session>) -> { api_key }
//!
//! Paid endpoints (require X-API-Key, deduct from balance):
//! - POST /v1/prove                ProveRequest -> ProveResponse (includes cost_cents, balance_remaining)
//! - POST /v1/verify               (multipart: field "proof") -> { status }
//! - POST /v1/proof/inspect        (multipart: "proof") -> parsed header summary
//!
//! Billing endpoints (Stripe):
//! - POST /v1/billing/topup        (requires X-API-Key) { pack: "small"|"medium"|"large" } -> checkout URL (NEW)
//! - POST /v1/billing/checkout     (DEPRECATED subscription) -> { url }
//! - POST /v1/stripe/webhook       (Stripe calls) -> { ok: true } (handles credit additions)
//!
//! Admin endpoints (require X-Admin-Token):
//! - POST /v1/admin/keys              -> { key, tier }
//! - POST /v1/admin/keys/:key/tier    -> { key, tier }
//! - GET  /v1/admin/keys/:key/usage   -> { key, month, used, cap, tier }
//!
//! Pricing Model:
//! - Usage-based: base_cost + (rows × registers / 1M) × cost_per_million_ops
//! - Free tier: 10M ops/month (configurable via TINYZKP_FREE_MONTHLY_OPS)
//! - Credit packs: $10 (1000¢), $50 (6000¢ + 20% bonus), $500 (75000¢ + 50% bonus)
//! - Example: 16M rows × 32 regs = 512M ops → ~$51 (vs $16GB RAM requirement traditional)
//!
//! Notes:
//! - Proof format is v2 (magic + u16 + ark-compressed).
//! - Dev runs don't require SRS files (feature `dev-srs`).

#![forbid(unsafe_code)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::net::SocketAddr;

use anyhow::{self};
use ark_ff::FftField; // for get_root_of_unity
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Multipart, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use base64::Engine;
use chrono::{Datelike, TimeZone, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use myzkp::{
    air::{AirSpec, Row},
    pcs::{Basis, PcsParams},
    scheduler::{Prover, Verifier as SchedVerifier},
    F, Proof, ProveParams, VerifyParams,
};

// Stripe SDK (async-stripe 0.37.x)
use stripe::{
    CheckoutSession, CheckoutSessionMode, Client as StripeClient, CreateCheckoutSession,
    CreateCheckoutSessionLineItems,
};
// NOTE: We intentionally avoid importing `stripe::Event` or `stripe::webhook` here,
// to remain compatible with async-stripe 0.37.x without extra features.

// ---------- NEW: password hashing + sessions ----------
// Cargo.toml (add):
// argon2 = "0.5"
// rand = "0.8"
// serde_json = "1"
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::rngs::OsRng;

// ------------------------------ KVS (Upstash) ------------------------------

#[derive(Deserialize)]
struct UpstashResp<T> {
    result: T,
}

#[derive(Clone)]
struct Kvs {
    url: String,           // e.g. https://pleasing-serval-14178.upstash.io
    token: String,         // ATdiAA...
    http: reqwest::Client, // reuse client
}

impl Kvs {
    fn from_env() -> anyhow::Result<Self> {
        let mut url = std::env::var("UPSTASH_REDIS_REST_URL")?;
        // Normalize: remove trailing slash if present
        if url.ends_with('/') {
            url.pop();
        }
        let token = std::env::var("UPSTASH_REDIS_REST_TOKEN")?;
        Ok(Self {
            url,
            token,
            http: reqwest::Client::new(),
        })
    }

    #[inline]
    fn auth(&self, rb: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        rb.header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
    }

    /// GET key -> Ok(Some(val)) / Ok(None)
    async fn get(&self, key: &str) -> anyhow::Result<Option<String>> {
        // Upstash REST: GET {URL}/get/<key> returns {"result": "<value>"} or {"result": null}
        let url = format!("{}/get/{}", self.url, key);
        let res = self.auth(self.http.get(&url)).send().await?;
        let status = res.status();
        let text = res.text().await?;
        if !status.is_success() {
            anyhow::bail!("kvs GET {} {} {}", key, status, text);
        }
        let parsed: UpstashResp<Option<serde_json::Value>> = serde_json::from_str(&text)?;
        Ok(match parsed.result {
            None => None,
            Some(serde_json::Value::String(s)) => Some(s),
            Some(other) => Some(other.to_string()),
        })
    }

    /// SETEX key seconds value
    async fn set_ex(&self, key: &str, val: &str, seconds: u64) -> anyhow::Result<()> {
        // Upstash REST: POST {URL}/setex/<key>/<seconds> with a JSON string body
        let url = format!("{}/setex/{}/{}", self.url, key, seconds);
        let body = serde_json::to_string(val)?; // sends `"value"`
        let res = self.auth(self.http.post(&url)).body(body).send().await?;
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        if !status.is_success() {
            anyhow::bail!("kvs SETEX {} {} {}", key, status, text);
        }
        // Optional: assert OK
        if let Ok(parsed) = serde_json::from_str::<UpstashResp<String>>(&text) {
            if parsed.result != "OK" {
                anyhow::bail!("kvs SETEX non-OK: {}", parsed.result);
            }
        }
        Ok(())
    }

    /// INCR key -> new value
    async fn incr(&self, key: &str) -> anyhow::Result<i64> {
        // Upstash REST: POST {URL}/incr/<key> returns {"result": <integer>}
        let url = format!("{}/incr/{}", self.url, key);
        let res = self.auth(self.http.post(&url)).send().await?;
        let status = res.status();
        let text = res.text().await?;
        if !status.is_success() {
            anyhow::bail!("kvs INCR {} {} {}", key, status, text);
        }
        let v: UpstashResp<i64> = serde_json::from_str(&text)?;
        Ok(v.result)
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
enum Tier {
    Free,
    Pro,
    Scale,
}

// ------------------------------ NEW: Usage-based Account Model ------------------------------

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Account {
    balance_cents: i64,       // Prepaid balance in cents
    total_spent_cents: i64,   // Lifetime spend
    free_ops_used: i64,       // Free tier ops used this month
}

impl Default for Account {
    fn default() -> Self {
        Self {
            balance_cents: 0,
            total_spent_cents: 0,
            free_ops_used: 0,
        }
    }
}

fn month_bucket() -> String {
    Utc::now().format("%Y-%m").to_string()
}

fn monthly_usage_key(api_key: &str) -> String {
    format!("tinyzkp:usage:{}:{}", month_bucket(), api_key)
}

fn account_key(api_key: &str) -> String {
    format!("tinyzkp:account:{}", api_key)
}

// ------------------------------ Pricing Functions ------------------------------

/// Calculate cost for a proof in cents
/// Formula: base_cost + (ops / 1_000_000) * cost_per_million_ops
/// where ops = rows × registers
fn calculate_cost(rows: usize, registers: usize, config: &PricingConfig) -> i64 {
    let ops = (rows as i64).saturating_mul(registers as i64);
    let base_cents = config.base_cost_cents;
    let compute_cents = ops
        .saturating_mul(config.cost_per_million_ops_cents)
        .saturating_div(1_000_000);
    base_cents.saturating_add(compute_cents)
}

#[derive(Clone)]
struct PricingConfig {
    base_cost_cents: i64,           // Base fee per proof (e.g., 1 cent = $0.01)
    cost_per_million_ops_cents: i64, // Cost per million row-ops (e.g., 10 cents = $0.10)
    free_monthly_ops: i64,          // Free tier allowance per month
}

fn end_of_month_ttl_secs() -> u64 {
    let now = Utc::now();
    let (y, m) = (now.year(), now.month());
    let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    let eom = Utc.with_ymd_and_hms(ny, nm, 1, 0, 0, 0).earliest().unwrap();
    let secs = (eom - now).num_seconds().max(86400) as u64;
    secs
}

// ------------------------------ Types ------------------------------

#[derive(Serialize)]
struct Health {
    status: &'static str,
}
#[derive(Serialize)]
struct Version {
    api: &'static str,
    protocol: &'static str,
    curve: &'static str,
    features: VersionFeatures,
}
#[derive(Serialize)]
struct VersionFeatures {
    dev_srs: bool,
    zeta_shift: bool,
    lookups: bool,
}

#[derive(Deserialize)]
struct DomainPlanReq {
    rows: usize,
    #[serde(default)]
    b_blk: Option<usize>,
    #[serde(default)]
    zh_c: Option<String>, // decimal u64 string (reserved, not used in planning)
}
#[derive(Serialize)]
struct DomainPlanRes {
    n: usize,
    b_blk: usize,
    omega_ok: bool,
    mem_hint_bytes: usize,
}

// NEW: Cost estimation types
#[derive(Deserialize)]
struct CostEstimateReq {
    rows: usize,
    registers: usize,
}
#[derive(Serialize)]
struct CostEstimateRes {
    cost_cents: i64,
    ops: i64,
    pricing: PricingInfoView,
    examples: Vec<CostExample>,
}
#[derive(Serialize)]
struct CostExample {
    description: String,
    rows: usize,
    registers: usize,
    cost_cents: i64,
    cost_usd: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", tag = "format")]
enum WitnessInput {
    JsonRows { rows: Vec<Vec<u64>> },
}

#[derive(Deserialize)]
struct ProveReq {
    air: AirCfg,
    domain: DomainCfg,
    pcs: PcsCfg,
    #[serde(default)]
    srs: Option<SrsCfg>,
    witness: WitnessInput,
    #[serde(default)]
    return_proof: bool,
}
#[derive(Deserialize)]
struct AirCfg {
    k: usize,
    #[serde(default)]
    selectors: Option<SelectorsCfg>,
}
#[derive(Deserialize)]
struct SelectorsCfg {
    #[serde(rename = "format")]
    _format: String, // e.g., "csv_inline"
    csv: String,     // demo only
}
#[derive(Deserialize)]
struct DomainCfg {
    rows: usize,
    b_blk: usize,
    #[serde(default = "one_str")]
    zh_c: String,
}
#[derive(Deserialize)]
struct PcsCfg {
    #[serde(default = "eval_basis")]
    basis_wires: String,
}
#[derive(Deserialize)]
struct SrsCfg {
    #[allow(dead_code)]
    id: String,
}

fn one_str() -> String {
    "1".into()
}
fn eval_basis() -> String {
    "eval".into()
}

#[derive(Serialize)]
struct ProveRes {
    header: ProofHeaderView,
    #[serde(skip_serializing_if = "Option::is_none")]
    proof_b64: Option<String>,
    // NEW: Cost transparency fields
    cost_cents: i64,
    balance_remaining_cents: i64,
}
#[derive(Serialize)]
struct ProofHeaderView {
    n: usize,
    omega_hex: String,
    zh_c_hex: String,
    k: usize,
    basis_wires: String,
    srs_g1_digest_hex: String,
    srs_g2_digest_hex: String,
}

#[derive(Serialize)]
struct VerifyRes {
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

#[derive(Serialize)]
struct ApiKeyInfo {
    key: String,
    tier: Tier,
}

#[derive(Serialize)]
struct UsageRes {
    key: String,
    month: String,
    used: i64,
    cap: i64,
    tier: Tier,
}

#[derive(Deserialize)]
struct SetTierReq {
    tier: String,
}

// ---- Billing types ----
#[derive(Deserialize)]
struct CheckoutReq {
    /// Optional email to prefill Checkout / receipt
    #[serde(default)]
    customer_email: Option<String>,
    /// Optional plan: "pro" (default) or "scale"
    #[serde(default)]
    plan: Option<String>,
}
#[derive(Serialize)]
struct CheckoutRes {
    url: String,
}
#[derive(Serialize)]
struct HookAck {
    ok: bool,
}

// NEW: Credit topup types
#[derive(Deserialize)]
struct TopupReq {
    /// Credit pack size: "small" ($10), "medium" ($50), "large" ($500)
    pack: String,
    /// Optional email to prefill Checkout
    #[serde(default)]
    customer_email: Option<String>,
}
#[derive(Serialize)]
struct TopupRes {
    url: String,
    amount_cents: i64,
}

// ---------- NEW: Accounts DTOs ----------
#[derive(Deserialize)]
struct SignupReq {
    email: String,
    password: String,
}
#[derive(Serialize)]
struct SignupRes {
    user_id: String,
    api_key: String,
    tier: String,
    session_token: String,
}

#[derive(Deserialize)]
struct LoginReq {
    email: String,
    password: String,
}
#[derive(Serialize)]
struct LoginRes {
    user_id: String,
    api_key: String,
    tier: String,
    session_token: String,
}

#[derive(Serialize)]
struct MeRes {
    user_id: String,
    email: String,
    api_key: String,
    // NEW: Usage-based fields
    balance_cents: i64,
    free_ops_remaining: i64,
    current_month_ops_used: i64,
    total_spent_cents: i64,
    pricing: PricingInfoView,
    // DEPRECATED: Old subscription fields (for backward compat)
    #[serde(skip_serializing_if = "Option::is_none")]
    tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    month: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    used: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    caps: Option<CapsView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limits: Option<LimitsView>,
}

#[derive(Serialize)]
struct PricingInfoView {
    base_cost_cents: i64,
    cost_per_million_ops_cents: i64,
    free_monthly_ops: i64,
}

#[derive(Serialize)]
struct CapsView {
    free: i64,
    pro: i64,
    scale: i64,
}
#[derive(Serialize)]
struct LimitsView {
    free_max_rows: usize,
    pro_max_rows: usize,
    scale_max_rows: usize,
}

#[derive(Serialize)]
struct RotateRes {
    api_key: String,
}

#[derive(Clone)]
struct AppState {
    addr: SocketAddr,
    kvs: Kvs,
    admin_token: String,
    // NEW: Usage-based pricing config
    pricing: PricingConfig,
    // per-tier monthly caps (DEPRECATED - keep for backward compat during migration)
    free_cap: i64,
    pro_cap: i64,
    scale_cap: i64,
    // global hard ceiling + per-tier row ceilings
    max_rows: usize,       // safety ceiling (global)
    free_max_rows: usize,  // default: 4_096
    pro_max_rows: usize,   // default: 16_384
    scale_max_rows: usize, // default: 65_536
    allow_dev_srs: bool,
    // Stripe config
    stripe: StripeClient,
    price_pro: String,       // DEPRECATED: subscription pricing
    price_scale: String,     // DEPRECATED: subscription pricing
    // NEW: Credit pack Stripe price IDs
    price_pack_small: String,  // $10 pack
    price_pack_medium: String, // $50 pack
    price_pack_large: String,  // $500 pack
    success_url: String,
    cancel_url: String,
    portal_return_url: String,
}

// ------------------------------ Helpers ------------------------------

fn parse_basis(s: &str) -> Basis {
    match s {
        "coeff" | "coefficient" => Basis::Coefficient,
        _ => Basis::Evaluation,
    }
}

fn fe_hex(x: F) -> String {
    let mut v = Vec::new();
    x.serialize_compressed(&mut v).expect("field serialize");
    let mut s = String::with_capacity(2 + v.len() * 2);
    s.push_str("0x");
    s.push_str(&hex::encode(v));
    s
}

fn header_view(p: &Proof) -> ProofHeaderView {
    ProofHeaderView {
        n: p.header.domain_n as usize,
        omega_hex: fe_hex(p.header.domain_omega),
        zh_c_hex: fe_hex(p.header.zh_c),
        k: p.header.k as usize,
        basis_wires: match p.header.basis_wires {
            Basis::Coefficient => "Coefficient",
            Basis::Evaluation => "Evaluation",
        }
        .into(),
        srs_g1_digest_hex: hex_bytes(&p.header.srs_g1_digest),
        srs_g2_digest_hex: hex_bytes(&p.header.srs_g2_digest),
    }
}
fn hex_bytes(b: &[u8; 32]) -> String {
    let mut s = String::with_capacity(2 + 64);
    s.push_str("0x");
    for x in b {
        s.push_str(&format!("{:02x}", x));
    }
    s
}

fn rows_from_json(rows: &[Vec<u64>], k: usize) -> anyhow::Result<Vec<Row>> {
    let mut out = Vec::with_capacity(rows.len());
    for (i, r) in rows.iter().enumerate() {
        if r.len() != k {
            return Err(anyhow::anyhow!(
                "row {} has {} columns, expected {}",
                i,
                r.len(),
                k
            ));
        }
        let regs: Vec<F> = r.iter().copied().map(F::from).collect();
        out.push(Row {
            regs: regs.into_boxed_slice(),
        });
    }
    Ok(out)
}

fn plan_b_blk(rows: usize, provided: Option<usize>) -> usize {
    if let Some(b) = provided {
        return b.max(1);
    }
    let n = next_pow2(rows.max(1));
    let approx = (n as f64).sqrt().round() as usize;
    approx.clamp(8, 1 << 12)
}
fn next_pow2(n: usize) -> usize {
    if n <= 1 {
        return 1;
    }
    n.next_power_of_two()
}

fn max_rows_for_tier(st: &AppState, tier: Tier) -> usize {
    let tier_cap = match tier {
        Tier::Free => st.free_max_rows,
        Tier::Pro => st.pro_max_rows,
        Tier::Scale => st.scale_max_rows,
    };
    tier_cap.min(st.max_rows) // enforce global hard ceiling too
}

// ---------- NEW: Accounts helpers ----------

fn valid_email(e: &str) -> bool {
    // MVP sanity (keeps deps light): contains one '@', a '.', and reasonable length
    if e.len() < 3 || e.len() > 254 {
        return false;
    }
    let (has_at, has_dot) = (e.contains('@'), e.rsplit('.').next().map(|s| !s.is_empty()).unwrap_or(false));
    has_at && has_dot
}

fn random_user_id() -> String {
    let mut r = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut r);
    hex::encode(r)
}

async fn new_session(kvs: &Kvs, user_id: &str, email: &str) -> anyhow::Result<String> {
    let mut r = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut r);
    let token = hex::encode(blake3::hash(&r).as_bytes());
    let payload = serde_json::json!({ "user_id": user_id, "email": email }).to_string();
    kvs.set_ex(&format!("tinyzkp:sess:{token}"), &payload, 30 * 24 * 3600).await?;
    Ok(token)
}

async fn auth_session(kvs: &Kvs, headers: &HeaderMap) -> Result<(String, String), (StatusCode, String)> {
    let token = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ").map(|x| x.to_string()))
        .ok_or((StatusCode::UNAUTHORIZED, "missing Bearer token".into()))?;
    let v = kvs
        .get(&format!("tinyzkp:sess:{token}"))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "invalid session".into()))?;
    let obj: serde_json::Value = serde_json::from_str(&v).unwrap_or(serde_json::json!({}));
    let uid = obj.get("user_id").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let email = obj.get("email").and_then(|x| x.as_str()).unwrap_or("").to_string();
    if uid.is_empty() {
        return Err((StatusCode::UNAUTHORIZED, "invalid session".into()));
    }
    Ok((uid, email))
}

// --- auth/usage helpers ---

// DEPRECATED: Old subscription-based auth (keep for backward compat)
async fn check_and_count(
    st: &AppState,
    headers: &HeaderMap,
) -> Result<(String, Tier, i64, i64), (StatusCode, String)> {
    let api_key = headers
        .get("x-api-key")
        .and_then(|h| h.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "missing X-API-Key".into()))?
        .to_string();

    let tier_s = st
        .kvs
        .get(&format!("tinyzkp:key:tier:{api_key}"))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "unknown API key".into()))?;

    if tier_s.eq_ignore_ascii_case("disabled") {
        return Err((StatusCode::UNAUTHORIZED, "API key disabled".into()));
    }

    let tier = match tier_s.as_str() {
        "pro" | "Pro" | "PRO" => Tier::Pro,
        "scale" | "Scale" | "SCALE" => Tier::Scale,
        _ => Tier::Free,
    };

    let cap = match tier {
        Tier::Free => st.free_cap,
        Tier::Pro => st.pro_cap,
        Tier::Scale => st.scale_cap,
    };

    let used = st
        .kvs
        .incr(&monthly_usage_key(&api_key))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // On first hit this month, set TTL to end of month to auto-reset counters.
    if used == 1 {
        let _ = st
            .kvs
            .set_ex(&monthly_usage_key(&api_key), "1", end_of_month_ttl_secs())
            .await;
    }

    if used > cap {
        return Err((
            StatusCode::PAYMENT_REQUIRED,
            format!("monthly cap reached ({used}/{cap})"),
        ));
    }

    Ok((api_key, tier, used, cap))
}

// NEW: Usage-based pricing auth and deduction
async fn check_and_deduct(
    st: &AppState,
    headers: &HeaderMap,
    rows: usize,
    registers: usize,
) -> Result<(String, i64, i64), (StatusCode, String)> {
    let api_key = headers
        .get("x-api-key")
        .and_then(|h| h.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "missing X-API-Key".into()))?
        .to_string();

    // Load account
    let acc_json = st
        .kvs
        .get(&account_key(&api_key))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let mut account: Account = if let Some(j) = acc_json {
        serde_json::from_str(&j).unwrap_or_default()
    } else {
        // New account - initialize with default
        Account::default()
    };

    // Calculate cost
    let ops = (rows as i64).saturating_mul(registers as i64);
    let cost_cents = calculate_cost(rows, registers, &st.pricing);

    // Check free tier first
    let free_available = st.pricing.free_monthly_ops.saturating_sub(account.free_ops_used);
    
    let (ops_from_free, ops_from_paid, actual_cost) = if free_available >= ops {
        // Entirely covered by free tier
        (ops, 0, 0)
    } else if free_available > 0 {
        // Partially covered by free tier
        let ops_paid = ops - free_available;
        let paid_cost = calculate_cost(
            (ops_paid / registers as i64).max(1) as usize, 
            registers, 
            &st.pricing
        );
        (free_available, ops_paid, paid_cost)
    } else {
        // No free tier left - pay full price
        (0, ops, cost_cents)
    };

    // Check if user has enough balance for paid portion
    if actual_cost > account.balance_cents {
        return Err((
            StatusCode::PAYMENT_REQUIRED,
            format!(
                "insufficient balance: need {} cents, have {} cents (free ops remaining: {}/{})",
                actual_cost,
                account.balance_cents,
                free_available,
                st.pricing.free_monthly_ops
            ),
        ));
    }

    // Deduct cost
    account.free_ops_used = account.free_ops_used.saturating_add(ops_from_free);
    account.balance_cents = account.balance_cents.saturating_sub(actual_cost);
    account.total_spent_cents = account.total_spent_cents.saturating_add(actual_cost);

    // Save updated account
    let acc_str = serde_json::to_string(&account).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("serialize account: {}", e))
    })?;
    st.kvs
        .set_ex(&account_key(&api_key), &acc_str, 365 * 24 * 3600)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Track usage stats
    let usage_key = monthly_usage_key(&api_key);
    let _ = st.kvs.incr(&usage_key).await; // Best-effort stats tracking

    Ok((api_key, actual_cost, account.balance_cents))
}

// ------------------------------ Public Handlers ------------------------------

async fn health() -> impl IntoResponse {
    Json(Health { status: "ok" })
}

async fn version() -> impl IntoResponse {
    Json(Version {
        api: "tinyzkp-api/0.3",
        protocol: "sszkp-v2",
        curve: "bn254/kzg",
        features: VersionFeatures {
            dev_srs: cfg!(feature = "dev-srs"),
            zeta_shift: cfg!(feature = "zeta-shift"),
            lookups: cfg!(feature = "lookups"),
        },
    })
}

async fn domain_plan(
    Json(req): Json<DomainPlanReq>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let n = next_pow2(req.rows.max(1));
    let b_blk = plan_b_blk(req.rows, req.b_blk);
    let omega = F::get_root_of_unity(n as u64)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "no n-th root of unity".into()))?;
    let mut ok = true;
    let mut pow = F::from(1u64);
    for _ in 0..n {
        pow *= omega;
    }
    if pow != F::from(1u64) {
        ok = false;
    }
    if n >= 2 {
        let mut pow2 = F::from(1u64);
        for _ in 0..(n / 2) {
            pow2 *= omega;
        }
        if pow2 == F::from(1u64) {
            ok = false;
        }
    }
    let mem_hint_bytes = b_blk * 64;
    Ok(Json(DomainPlanRes {
        n,
        b_blk,
        omega_ok: ok,
        mem_hint_bytes,
    }))
}

// NEW: Cost estimation endpoint
async fn pricing_estimate(
    State(st): State<AppState>,
    Json(req): Json<CostEstimateReq>,
) -> Result<Json<CostEstimateRes>, (StatusCode, String)> {
    let ops = (req.rows as i64).saturating_mul(req.registers as i64);
    let cost_cents = calculate_cost(req.rows, req.registers, &st.pricing);
    
    // Provide helpful examples at different scales
    let examples = vec![
        CostExample {
            description: "Small demo (1K rows × 3 regs)".into(),
            rows: 1024,
            registers: 3,
            cost_cents: calculate_cost(1024, 3, &st.pricing),
            cost_usd: format!("${:.4}", calculate_cost(1024, 3, &st.pricing) as f64 / 100.0),
        },
        CostExample {
            description: "Medium computation (16K rows × 32 regs)".into(),
            rows: 16384,
            registers: 32,
            cost_cents: calculate_cost(16384, 32, &st.pricing),
            cost_usd: format!("${:.3}", calculate_cost(16384, 32, &st.pricing) as f64 / 100.0),
        },
        CostExample {
            description: "Large computation (1M rows × 32 regs)".into(),
            rows: 1_048_576,
            registers: 32,
            cost_cents: calculate_cost(1_048_576, 32, &st.pricing),
            cost_usd: format!("${:.2}", calculate_cost(1_048_576, 32, &st.pricing) as f64 / 100.0),
        },
        CostExample {
            description: "Production scale (16M rows × 32 regs)".into(),
            rows: 16_777_216,
            registers: 32,
            cost_cents: calculate_cost(16_777_216, 32, &st.pricing),
            cost_usd: format!("${:.2}", calculate_cost(16_777_216, 32, &st.pricing) as f64 / 100.0),
        },
    ];
    
    Ok(Json(CostEstimateRes {
        cost_cents,
        ops,
        pricing: PricingInfoView {
            base_cost_cents: st.pricing.base_cost_cents,
            cost_per_million_ops_cents: st.pricing.cost_per_million_ops_cents,
            free_monthly_ops: st.pricing.free_monthly_ops,
        },
        examples,
    }))
}

// ------------------------------ Accounts Handlers ------------------------------

async fn auth_signup(
    State(st): State<AppState>,
    Json(req): Json<SignupReq>,
) -> Result<Json<SignupRes>, (StatusCode, String)> {
    let email = req.email.trim().to_lowercase();
    if !valid_email(&email) {
        return Err((StatusCode::BAD_REQUEST, "invalid email".into()));
    }
    if req.password.len() < 8 {
        return Err((StatusCode::BAD_REQUEST, "password too short".into()));
    }

    if st
        .kvs
        .get(&format!("tinyzkp:user:by_email:{email}"))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .is_some()
    {
        return Err((StatusCode::CONFLICT, "email already registered".into()));
    }

    let api_key = random_key();
    let user_id = random_user_id();

    // Hash password (Argon2id default params)
    let salt = SaltString::generate(&mut OsRng);
    let argon = Argon2::default();
    let pw_hash = argon
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .to_string();

    let user_obj = serde_json::json!({
        "email": email,
        "pw_salt_hex": salt.as_str(),
        "pw_hash_hex": pw_hash,
        "api_key": api_key,
        "tier": "free",
        "created_at": Utc::now().timestamp(),
        "status": "active"
    })
    .to_string();

    // Store user + indexes + API key tier
    let year = 365 * 24 * 3600;
    st.kvs
        .set_ex(
            &format!("tinyzkp:user:by_email:{email}"),
            &serde_json::json!({ "user_id": user_id }).to_string(),
            year,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    st.kvs
        .set_ex(&format!("tinyzkp:user:{user_id}"), &user_obj, year)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    st.kvs
        .set_ex(
            &format!("tinyzkp:key:owner:{api_key}"),
            &serde_json::json!({ "user_id": user_id }).to_string(),
            year,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    st.kvs
        .set_ex(&format!("tinyzkp:key:tier:{api_key}"), "free", year)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let session = new_session(&st.kvs, &user_id, &email)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(SignupRes {
        user_id,
        api_key,
        tier: "free".into(),
        session_token: session,
    }))
}

async fn auth_login(
    State(st): State<AppState>,
    Json(req): Json<LoginReq>,
) -> Result<Json<LoginRes>, (StatusCode, String)> {
    let email = req.email.trim().to_lowercase();
    let uid_v = st
        .kvs
        .get(&format!("tinyzkp:user:by_email:{email}"))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "invalid credentials".into()))?;
    let uid_obj: serde_json::Value =
        serde_json::from_str(&uid_v).unwrap_or(serde_json::json!({}));
    let user_id = uid_obj
        .get("user_id")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    if user_id.is_empty() {
        return Err((StatusCode::UNAUTHORIZED, "invalid credentials".into()));
    }

    let user_v = st
        .kvs
        .get(&format!("tinyzkp:user:{user_id}"))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "invalid credentials".into()))?;
    let user: serde_json::Value = serde_json::from_str(&user_v).unwrap_or(serde_json::json!({}));

    let pw_hash = user
        .get("pw_hash_hex")
        .and_then(|x| x.as_str())
        .ok_or((StatusCode::UNAUTHORIZED, "invalid credentials".into()))?;
    let parsed =
        PasswordHash::new(pw_hash).map_err(|_| (StatusCode::UNAUTHORIZED, "invalid credentials".into()))?;
    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid credentials".into()))?;

    let api_key = user
        .get("api_key")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();

    // Prefer live tier on the key (webhook keeps it fresh)
    let tier_live = st
        .kvs
        .get(&format!("tinyzkp:key:tier:{api_key}"))
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "free".into());

    let session = new_session(&st.kvs, &user_id, &email)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(LoginRes {
        user_id,
        api_key,
        tier: tier_live,
        session_token: session,
    }))
}

async fn me(
    State(st): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<MeRes>, (StatusCode, String)> {
    let (user_id, email) = auth_session(&st.kvs, &headers).await?;
    let user_v = st
        .kvs
        .get(&format!("tinyzkp:user:{user_id}"))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "invalid session".into()))?;
    let user: serde_json::Value = serde_json::from_str(&user_v).unwrap_or(serde_json::json!({}));

    let api_key = user
        .get("api_key")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();

    // NEW: Load usage-based account
    let acc_json = st
        .kvs
        .get(&account_key(&api_key))
        .await
        .ok()
        .flatten();
    
    let account: Account = if let Some(j) = acc_json {
        serde_json::from_str(&j).unwrap_or_default()
    } else {
        Account::default()
    };

    let free_remaining = st.pricing.free_monthly_ops.saturating_sub(account.free_ops_used);

    // Usage stats (best-effort)
    let ops_used = account.free_ops_used;

    Ok(Json(MeRes {
        user_id,
        email,
        api_key,
        balance_cents: account.balance_cents,
        free_ops_remaining: free_remaining,
        current_month_ops_used: ops_used,
        total_spent_cents: account.total_spent_cents,
        pricing: PricingInfoView {
            base_cost_cents: st.pricing.base_cost_cents,
            cost_per_million_ops_cents: st.pricing.cost_per_million_ops_cents,
            free_monthly_ops: st.pricing.free_monthly_ops,
        },
        // Old fields for backward compat (all optional)
        tier: None,
        month: None,
        used: None,
        caps: None,
        limits: None,
    }))
}

async fn rotate_key(
    State(st): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<RotateRes>, (StatusCode, String)> {
    let (user_id, _email) = auth_session(&st.kvs, &headers).await?;

    // Load user
    let user_key = format!("tinyzkp:user:{user_id}");
    let user_v = st
        .kvs
        .get(&user_key)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "invalid session".into()))?;
    let mut user: serde_json::Value =
        serde_json::from_str(&user_v).unwrap_or(serde_json::json!({}));

    let old_api = user
        .get("api_key")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();

    // Read old tier (fallback to free)
    let tier_str = st
        .kvs
        .get(&format!("tinyzkp:key:tier:{old_api}"))
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "free".into());

    // Mint new key
    let new_api = random_key();
    let year = 365 * 24 * 3600;

    // Disable old key for safety
    st.kvs
        .set_ex(&format!("tinyzkp:key:tier:{old_api}"), "disabled", year)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Create owner/tier for new key
    st.kvs
        .set_ex(
            &format!("tinyzkp:key:owner:{new_api}"),
            &serde_json::json!({ "user_id": user_id }).to_string(),
            year,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    st.kvs
        .set_ex(&format!("tinyzkp:key:tier:{new_api}"), &tier_str, year)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Update user record
    user["api_key"] = serde_json::Value::String(new_api.clone());
    let user_str = user.to_string();
    st.kvs
        .set_ex(&user_key, &user_str, year)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(RotateRes { api_key: new_api }))
}

// ------------------------------ Paid Handlers ------------------------------

async fn prove(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ProveReq>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // NEW: Usage-based pricing - check and deduct cost upfront
    let (_api_key, cost_cents, balance_remaining) = 
        check_and_deduct(&st, &headers, req.domain.rows, req.air.k).await?;

    // Global safety cap (defense-in-depth)
    if req.domain.rows > st.max_rows {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "rows exceeds maximum limit ({}/{}). Contact support for enterprise limits.",
                req.domain.rows, st.max_rows
            ),
        ));
    }

    // Domain from request (zh_c as u64 string)
    let n_rows = req.domain.rows;
    let b_blk = req.domain.b_blk.max(1);
    let n_domain = next_pow2(n_rows);
    let omega = F::get_root_of_unity(n_domain as u64)
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                format!("no {}-th root of unity", n_domain),
            )
        })?;
    let zh_u = req
        .domain
        .zh_c
        .parse::<u64>()
        .map_err(|_| (StatusCode::BAD_REQUEST, "zh_c must be decimal u64".into()))?;
    let zh_c = F::from(zh_u);

    // AIR + selectors (only inline CSV small demo supported here)
    let selectors = if let Some(sel) = &req.air.selectors {
        let mut rows: Vec<Vec<F>> = Vec::new();
        for (lineno, line) in sel.csv.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let mut r = Vec::new();
            for tok in line.split(|c: char| c == ',' || c.is_whitespace()) {
                if tok.is_empty() {
                    continue;
                }
                let v = tok.parse::<u64>().map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        format!(
                            "selectors parse error at line {}: `{}` ({})",
                            lineno + 1,
                            tok,
                            e
                        ),
                    )
                })?;
                r.push(F::from(v));
            }
            if !r.is_empty() {
                rows.push(r);
            }
        }
        if !rows.is_empty() {
            let s_cols = rows[0].len();
            for (i, r) in rows.iter().enumerate() {
                if r.len() != s_cols {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        format!(
                            "selectors are ragged: row 0 has {} cols, row {} has {}",
                            s_cols,
                            i,
                            r.len()
                        ),
                    ));
                }
            }
            let mut cols: Vec<Vec<F>> = vec![Vec::with_capacity(rows.len()); s_cols];
            for r in rows {
                for (j, v) in r.into_iter().enumerate() {
                    cols[j].push(v);
                }
            }
            cols.into_iter().map(|v| v.into_boxed_slice()).collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let air = AirSpec {
        k: req.air.k,
        id_table: Vec::new(),
        sigma_table: Vec::new(),
        selectors,
    };
    let basis_wires = parse_basis(&req.pcs.basis_wires);
    let domain = myzkp::domain::Domain {
        n: n_domain,
        omega,
        zh_c,
    };

    // PCS params
    let pcs_wires = PcsParams {
        max_degree: n_domain - 1,
        basis: basis_wires,
        srs_placeholder: (),
    };
    let pcs_coeff = PcsParams {
        max_degree: n_domain - 1,
        basis: Basis::Coefficient,
        srs_placeholder: (),
    };
    let prove_params = ProveParams {
        domain: domain.clone(),
        pcs_wires,
        pcs_coeff,
        b_blk,
    };

    // Witness
    let witness_rows: Vec<Row> = match &req.witness {
        WitnessInput::JsonRows { rows } => {
            rows_from_json(rows, req.air.k).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
        }
    };

    // Prove
    let prover = Prover {
        air: &air,
        params: &prove_params,
    };
    let proof = prover.prove_with_restreamer(&witness_rows).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("prover failed: {e}"),
        )
    })?;

    // Header view
    let header_v = header_view(&proof);

    // Optional: return v2 container as base64
    let proof_b64 = if req.return_proof {
        let mut payload = Vec::new();
        proof
            .serialize_compressed(&mut payload)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("serialize proof: {e}")))?;
        let mut out = Vec::with_capacity(8 + 2 + payload.len());
        out.extend_from_slice(b"SSZKPv2\0");
        out.extend_from_slice(&2u16.to_be_bytes());
        out.extend_from_slice(&payload);
        Some(base64::engine::general_purpose::STANDARD.encode(out))
    } else {
        None
    };

    Ok(Json(ProveRes {
        header: header_v,
        proof_b64,
        cost_cents,
        balance_remaining_cents: balance_remaining,
    }))
}

async fn verify(
    State(st): State<AppState>,
    headers: HeaderMap,
    mut mp: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let _ = check_and_count(&st, &headers).await?;

    // Expect a single part named "proof"
    let mut proof_bytes: Option<Vec<u8>> = None;
    while let Some(field) = mp
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("multipart error: {e}")))?
    {
        if let Some(name) = field.name() {
            if name == "proof" {
                let data = field.bytes().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, format!("read multipart: {e}"))
                })?;
                proof_bytes = Some(data.to_vec());
            }
        }
    }
    let buf = proof_bytes
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "multipart field 'proof' is required".into()))?;

    // Parse v2 container
    if buf.len() < 10 || &buf[0..8] != b"SSZKPv2\0" {
        return Err((StatusCode::BAD_REQUEST, "bad proof file: missing magic".into()));
    }
    let ver = u16::from_be_bytes([buf[8], buf[9]]);
    if ver != 2 {
        return Err((StatusCode::BAD_REQUEST, format!("unsupported proof version {ver}")));
    }
    let payload = &buf[10..];
    let mut slice = payload;
    let proof: Proof = CanonicalDeserialize::deserialize_compressed(&mut slice)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("deserialize proof: {e}")))?;

    // Build env from header (authoritative)
    let domain = myzkp::domain::Domain {
        n: proof.header.domain_n as usize,
        omega: proof.header.domain_omega,
        zh_c: proof.header.zh_c,
    };
    let pcs_wires = PcsParams {
        max_degree: domain.n - 1,
        basis: proof.header.basis_wires,
        srs_placeholder: (),
    };
    let pcs_coeff = PcsParams {
        max_degree: domain.n - 1,
        basis: Basis::Coefficient,
        srs_placeholder: (),
    };
    let vp = VerifyParams {
        domain,
        pcs_wires,
        pcs_coeff,
    };
    let verifier = SchedVerifier { params: &vp };

    if let Err(e) = verifier.verify(&proof) {
        return Ok((
            StatusCode::OK,
            Json(VerifyRes {
                status: "failed",
                reason: Some(format!("{e}")),
            }),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(VerifyRes {
            status: "ok",
            reason: None,
        }),
    ))
}

async fn inspect(
    State(st): State<AppState>,
    headers: HeaderMap,
    mut mp: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let _ = check_and_count(&st, &headers).await?;

    let mut proof_bytes: Option<Vec<u8>> = None;
    while let Some(field) = mp
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("multipart error: {e}")))?
    {
        if let Some(name) = field.name() {
            if name == "proof" {
                let data = field.bytes().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, format!("read multipart: {e}"))
                })?;
                proof_bytes = Some(data.to_vec());
            }
        }
    }
    let buf = proof_bytes
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "multipart field 'proof' is required".into()))?;
    if buf.len() < 10 || &buf[0..8] != b"SSZKPv2\0" {
        return Err((StatusCode::BAD_REQUEST, "bad proof file: missing magic".into()));
    }
    let ver = u16::from_be_bytes([buf[8], buf[9]]);
    if ver != 2 {
        return Err((StatusCode::BAD_REQUEST, format!("unsupported proof version {ver}")));
    }
    let payload = &buf[10..];
    let mut slice = payload;
    let proof: Proof = CanonicalDeserialize::deserialize_compressed(&mut slice)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("deserialize proof: {e}")))?;

    Ok(Json(header_view(&proof)))
}

// ------------------------------ Billing Handlers (Stripe) ------------------------------

/// POST /v1/billing/topup (requires X-API-Key) — NEW usage-based pricing
async fn billing_topup(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<TopupReq>,
) -> Result<Json<TopupRes>, (StatusCode, String)> {
    // Validate API key exists (no usage deduction for checkout)
    let api_key = headers
        .get("x-api-key")
        .and_then(|h| h.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "missing X-API-Key".into()))?
        .to_string();

    let _ = st
        .kvs
        .get(&account_key(&api_key))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Determine pack and price
    let (price_id, amount_cents) = match req.pack.to_lowercase().as_str() {
        "small" => (st.price_pack_small.as_str(), 1000),  // $10
        "medium" => (st.price_pack_medium.as_str(), 5000), // $50 (20% bonus = 6000 cents)
        "large" => (st.price_pack_large.as_str(), 50000),  // $500 (50% bonus = 75000 cents)
        _ => return Err((StatusCode::BAD_REQUEST, "invalid pack: must be small/medium/large".into())),
    };

    // Create Stripe Checkout Session (one-time payment)
    let mut params = CreateCheckoutSession::new();
    params.mode = Some(CheckoutSessionMode::Payment); // One-time payment, not subscription

    params.success_url = Some(st.success_url.as_str());
    params.cancel_url = Some(st.cancel_url.as_str());

    // Line items (one-time price)
    params.line_items = Some(vec![CreateCheckoutSessionLineItems {
        price: Some(price_id.to_string()),
        quantity: Some(1),
        ..Default::default()
    }]);

    // Attach api_key + amount to session metadata for webhook
    let mut md = std::collections::HashMap::new();
    md.insert("api_key".to_string(), api_key.clone());
    md.insert("amount_cents".to_string(), amount_cents.to_string());
    md.insert("pack".to_string(), req.pack.clone());
    params.metadata = Some(md);

    // Optional customer email
    let customer_email_owned = req.customer_email;
    if let Some(ref email) = customer_email_owned {
        params.customer_email = Some(email.as_str());
    }

    let session: CheckoutSession = CheckoutSession::create(&st.stripe, params)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("stripe: {e}")))?;

    let url = session
        .url
        .ok_or((StatusCode::BAD_GATEWAY, "stripe: missing checkout URL".into()))?;
    
    Ok(Json(TopupRes { url, amount_cents }))
}

/// POST /v1/billing/checkout (requires X-API-Key) — DEPRECATED subscription model
async fn billing_checkout(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CheckoutReq>,
) -> Result<Json<CheckoutRes>, (StatusCode, String)> {
    // Validate API key exists (no usage increment)
    let api_key = headers
        .get("x-api-key")
        .and_then(|h| h.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "missing X-API-Key".into()))?
        .to_string();

    let _tier = st
        .kvs
        .get(&format!("tinyzkp:key:tier:{api_key}"))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "unknown API key".into()))?;

    // Which plan?
    let plan = req.plan.as_deref().unwrap_or("pro");
    let (price_id, tier_str) = if plan.eq_ignore_ascii_case("scale") {
        (st.price_scale.as_str(), "scale")
    } else {
        (st.price_pro.as_str(), "pro")
    };

    // Create Stripe Checkout Session (subscription)
    let mut params = CreateCheckoutSession::new();
    params.mode = Some(CheckoutSessionMode::Subscription);

    // async-stripe 0.37 expects &str for these fields
    params.success_url = Some(st.success_url.as_str());
    params.cancel_url = Some(st.cancel_url.as_str());

    // Line items (subscription price)
    params.line_items = Some(vec![CreateCheckoutSessionLineItems {
        price: Some(price_id.to_string()),
        quantity: Some(1),
        ..Default::default()
    }]);

    // Attach api_key + tier to both session and resulting subscription metadata
    let mut md = std::collections::HashMap::new();
    md.insert("api_key".to_string(), api_key.clone());
    md.insert("tier".to_string(), tier_str.to_string());
    params.metadata = Some(md);

    let mut sub_md = std::collections::HashMap::new();
    sub_md.insert("api_key".to_string(), api_key.clone());
    sub_md.insert("tier".to_string(), tier_str.to_string());
    params.subscription_data =
        Some(stripe::CreateCheckoutSessionSubscriptionData { metadata: Some(sub_md), ..Default::default() });

    // Keep an owned email alive across the create() call
    let customer_email_owned = req.customer_email;
    if let Some(ref email) = customer_email_owned {
        params.customer_email = Some(email.as_str());
    }

    let session: CheckoutSession = CheckoutSession::create(&st.stripe, params)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("stripe: {e}")))?;

    let url = session
        .url
        .ok_or((StatusCode::BAD_GATEWAY, "stripe: missing checkout URL".into()))?;
    Ok(Json(CheckoutRes { url }))
}

/// POST /v1/stripe/webhook (Stripe calls this)
///
/// Minimal handler compatible with async-stripe 0.37 without the webhook helper:
/// - Requires the `stripe-signature` header to be present (defense-in-depth).
/// - Parses the JSON payload and matches on `type`.
/// - Uses `metadata.api_key` and `metadata.tier` to set the plan.
///   Falls back to Free if unclear.
async fn stripe_webhook(
    State(st): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<HookAck>), (StatusCode, String)> {
    // Require header (even if we don’t verify cryptographically here)
    let _sig = headers
        .get("stripe-signature")
        .and_then(|h| h.to_str().ok())
        .ok_or((StatusCode::BAD_REQUEST, "missing stripe-signature".into()))?;

    // Parse JSON payload directly
    let payload = std::str::from_utf8(&body)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid utf-8 payload".to_string()))?;
    let v: serde_json::Value =
        serde_json::from_str(payload).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let typ = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let obj = v
        .get("data")
        .and_then(|d| d.get("object"))
        .and_then(|o| o.as_object())
        .ok_or((StatusCode::BAD_REQUEST, "missing data.object".to_string()))?;

    // Helper: read tier string safely
    let read_tier = |o: &serde_json::Map<String, serde_json::Value>| -> Option<&'static str> {
        if let Some(t) = o
            .get("metadata")
            .and_then(|m| m.get("tier"))
            .and_then(|x| x.as_str())
        {
            if t.eq_ignore_ascii_case("scale") {
                return Some("scale");
            } else if t.eq_ignore_ascii_case("pro") {
                return Some("pro");
            } else if t.eq_ignore_ascii_case("free") {
                return Some("free");
            }
        }
        None
    };

    match typ {
        "checkout.session.completed" => {
            // Check if this is a topup (new usage-based) or subscription (old model)
            let metadata = obj.get("metadata").and_then(|m| m.as_object());
            
            if let Some(md) = metadata {
                if let Some(api_key) = md.get("api_key").and_then(|x| x.as_str()) {
                    // Check if this is a credit topup (has amount_cents in metadata)
                    if let Some(amt_str) = md.get("amount_cents").and_then(|x| x.as_str()) {
                        // NEW: Credit topup flow
                        if let Ok(amount_cents) = amt_str.parse::<i64>() {
                            // Load account
                            let acc_json = st.kvs.get(&account_key(api_key)).await.ok().flatten();
                            let mut account: Account = if let Some(j) = acc_json {
                                serde_json::from_str(&j).unwrap_or_default()
                            } else {
                                Account::default()
                            };
                            
                            // Add credits (with bonus based on pack size)
                            let pack = md.get("pack").and_then(|x| x.as_str()).unwrap_or("small");
                            let credits = match pack {
                                "small" => 1000,       // $10 → 1000 cents
                                "medium" => 6000,      // $50 → 6000 cents (20% bonus)
                                "large" => 75000,      // $500 → 75000 cents (50% bonus)
                                _ => amount_cents,
                            };
                            
                            account.balance_cents = account.balance_cents.saturating_add(credits);
                            
                            // Save account
                            if let Ok(acc_str) = serde_json::to_string(&account) {
                                let _ = st.kvs.set_ex(&account_key(api_key), &acc_str, 365 * 24 * 3600).await;
                            }
                        }
                    } else {
                        // OLD: Subscription activation flow (deprecated but kept for backward compat)
                        let tier = read_tier(obj).unwrap_or("pro");
                        let store = match tier {
                            "scale" => "scale",
                            "pro" => "pro",
                            _ => "free",
                        };
                        st.kvs
                            .set_ex(&format!("tinyzkp:key:tier:{api_key}"), store, 365 * 24 * 3600)
                            .await
                            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                    }
                }
            }
        }
        "customer.subscription.deleted" => {
            if let Some(api_key) = obj
                .get("metadata")
                .and_then(|m| m.get("api_key"))
                .and_then(|x| x.as_str())
            {
                st.kvs
                    .set_ex(&format!("tinyzkp:key:tier:{api_key}"), "free", 365 * 24 * 3600)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            }
        }
        "customer.subscription.updated" => {
            let status = obj.get("status").and_then(|s| s.as_str()).unwrap_or("");
            let tier = read_tier(obj);
            if let Some(api_key) = obj
                .get("metadata")
                .and_then(|m| m.get("api_key"))
                .and_then(|x| x.as_str())
            {
                // If Stripe tells us it's inactive, downgrade to Free regardless of metadata.
                let store = if status == "active" || status == "trialing" {
                    match tier {
                        Some("scale") => "scale",
                        Some("pro") => "pro",
                        Some("free") => "free",
                        _ => "pro",
                    }
                } else {
                    "free"
                };
                st.kvs
                    .set_ex(&format!("tinyzkp:key:tier:{api_key}"), store, 365 * 24 * 3600)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            }
        }
        _ => { /* ignore others */ }
    }

    Ok((StatusCode::OK, Json(HookAck { ok: true })))
}

// ------------------------------ Admin Handlers ------------------------------

fn random_key() -> String {
    let mut r = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut r);
    let h = blake3::hash(&r).to_hex();
    format!("tz_{h}")
}

async fn admin_new_key(
    State(st): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiKeyInfo>, (StatusCode, String)> {
    let auth = headers.get("x-admin-token").and_then(|v| v.to_str().ok());
    if auth != Some(st.admin_token.as_str()) {
        return Err((StatusCode::UNAUTHORIZED, "bad admin token".into()));
    }
    let key = random_key();
    st.kvs
        .set_ex(&format!("tinyzkp:key:tier:{key}"), "free", 365 * 24 * 3600)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(ApiKeyInfo {
        key,
        tier: Tier::Free,
    }))
}

async fn admin_set_tier(
    State(st): State<AppState>,
    axum::extract::Path(key): axum::extract::Path<String>,
    headers: HeaderMap,
    Json(req): Json<SetTierReq>,
) -> Result<Json<ApiKeyInfo>, (StatusCode, String)> {
    let auth = headers.get("x-admin-token").and_then(|v| v.to_str().ok());
    if auth != Some(st.admin_token.as_str()) {
        return Err((StatusCode::UNAUTHORIZED, "bad admin token".into()));
    }
    let tier_s = match req.tier.as_str() {
        "scale" | "Scale" | "SCALE" => "scale",
        "pro" | "Pro" | "PRO" => "pro",
        _ => "free",
    };
    st.kvs
        .set_ex(&format!("tinyzkp:key:tier:{key}"), tier_s, 365 * 24 * 3600)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let tier = match tier_s {
        "scale" => Tier::Scale,
        "pro" => Tier::Pro,
        _ => Tier::Free,
    };
    Ok(Json(ApiKeyInfo { key, tier }))
}

async fn admin_usage(
    State(st): State<AppState>,
    axum::extract::Path(key): axum::extract::Path<String>,
    headers: HeaderMap,
) -> Result<Json<UsageRes>, (StatusCode, String)> {
    let auth = headers.get("x-admin-token").and_then(|v| v.to_str().ok());
    if auth != Some(st.admin_token.as_str()) {
        return Err((StatusCode::UNAUTHORIZED, "bad admin token".into()));
    }
    let tier_s = st
        .kvs
        .get(&format!("tinyzkp:key:tier:{key}"))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .unwrap_or_else(|| "free".into());
    let tier = match tier_s.as_str() {
        "scale" => Tier::Scale,
        "pro" => Tier::Pro,
        _ => Tier::Free,
    };
    let used = st
        .kvs
        .get(&monthly_usage_key(&key))
        .await
        .ok()
        .flatten()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    let cap = match tier {
        Tier::Free => st.free_cap,
        Tier::Pro => st.pro_cap,
        Tier::Scale => st.scale_cap,
    };
    Ok(Json(UsageRes {
        key,
        month: month_bucket(),
        used,
        cap,
        tier,
    }))
}

// ------------------------------ Main ------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Bind address from env or default
    let addr: SocketAddr = std::env::var("TINYZKP_ADDR")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 8080)));

    // Service config / policy
    let kvs = Kvs::from_env()?;
    let admin_token =
        std::env::var("TINYZKP_ADMIN_TOKEN").unwrap_or_else(|_| "changeme-admin".into());

    // Per-tier caps (defaults match site)
    let free_cap = std::env::var("TINYZKP_FREE_MONTHLY_CAP")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(500);
    let pro_cap = std::env::var("TINYZKP_PRO_MONTHLY_CAP")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5000);
    let scale_cap = std::env::var("TINYZKP_SCALE_MONTHLY_CAP")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50_000);

    // Row ceilings (defaults: Free 4,096 · Pro 16,384 · Scale 65,536)
    let max_rows = std::env::var("TINYZKP_MAX_ROWS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(131_072); // global safety cap
    let free_max_rows = std::env::var("TINYZKP_FREE_MAX_ROWS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4_096);
    let pro_max_rows = std::env::var("TINYZKP_PRO_MAX_ROWS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(16_384);
    let scale_max_rows = std::env::var("TINYZKP_SCALE_MAX_ROWS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(65_536);

    let allow_dev_srs = std::env::var("TINYZKP_ALLOW_DEV_SRS")
        .map(|s| s == "true")
        .unwrap_or(true);

    // NEW: Usage-based pricing config
    let pricing = PricingConfig {
        base_cost_cents: std::env::var("TINYZKP_BASE_COST_CENTS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1), // $0.01 per proof
        cost_per_million_ops_cents: std::env::var("TINYZKP_COST_PER_MILLION_OPS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10), // $0.10 per million ops
        free_monthly_ops: std::env::var("TINYZKP_FREE_MONTHLY_OPS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10_000_000), // 10M ops free/month
    };

    // Stripe config
    let stripe = StripeClient::new(std::env::var("STRIPE_SECRET_KEY")?);
    
    // OLD: Subscription price IDs (deprecated)
    let price_pro = std::env::var("STRIPE_PRICE_PRO").unwrap_or_else(|_| "price_pro_deprecated".into());
    let price_scale = std::env::var("STRIPE_PRICE_SCALE")
        .unwrap_or_else(|_| "price_scale_deprecated".into());
    
    // NEW: Credit pack price IDs
    let price_pack_small = std::env::var("STRIPE_PRICE_PACK_SMALL")
        .unwrap_or_else(|_| "price_pack_small_not_set".into()); // $10 pack
    let price_pack_medium = std::env::var("STRIPE_PRICE_PACK_MEDIUM")
        .unwrap_or_else(|_| "price_pack_medium_not_set".into()); // $50 pack
    let price_pack_large = std::env::var("STRIPE_PRICE_PACK_LARGE")
        .unwrap_or_else(|_| "price_pack_large_not_set".into()); // $500 pack
    
    let success_url =
        std::env::var("BILLING_SUCCESS_URL").unwrap_or_else(|_| "https://tinyzkp.com/success".into());
    let cancel_url =
        std::env::var("BILLING_CANCEL_URL").unwrap_or_else(|_| "https://tinyzkp.com/cancel".into());
    let portal_return_url = std::env::var("BILLING_PORTAL_RETURN_URL")
        .unwrap_or_else(|_| "https://tinyzkp.com/account".into());

    let app = Router::new()
        // public
        .route("/v1/health", get(health))
        .route("/v1/version", get(version))
        .route("/v1/domain/plan", post(domain_plan))
        .route("/v1/pricing/estimate", post(pricing_estimate)) // NEW: Cost calculator
        // accounts
        .route("/v1/auth/signup", post(auth_signup))
        .route("/v1/auth/login", post(auth_login))
        .route("/v1/me", get(me))
        .route("/v1/keys/rotate", post(rotate_key))
        // paid
        .route("/v1/prove", post(prove))
        .route("/v1/verify", post(verify))
        .route("/v1/proof/inspect", post(inspect))
        // billing (NEW: usage-based)
        .route("/v1/billing/topup", post(billing_topup))
        // billing (OLD: subscription - deprecated)
        .route("/v1/billing/checkout", post(billing_checkout))
        .route("/v1/stripe/webhook", post(stripe_webhook))
        // admin
        .route("/v1/admin/keys", post(admin_new_key))
        .route("/v1/admin/keys/:key/tier", post(admin_set_tier))
        .route("/v1/admin/keys/:key/usage", get(admin_usage))
        .layer(DefaultBodyLimit::max(32 * 1024 * 1024)) // 32MB
        .with_state(AppState {
            addr,
            kvs,
            admin_token,
            pricing, // NEW: Usage-based pricing config
            free_cap,
            pro_cap,
            scale_cap,
            max_rows,
            free_max_rows,
            pro_max_rows,
            scale_max_rows,
            allow_dev_srs,
            stripe,
            price_pro,
            price_scale,
            price_pack_small,  // NEW
            price_pack_medium, // NEW
            price_pack_large,  // NEW
            success_url,
            cancel_url,
            portal_return_url,
        })
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    println!("tinyzkp API listening on http://{addr}");
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
