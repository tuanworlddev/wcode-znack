//! Data types for the Честный ЗНАК (Znack) integration.
//! Mirrors the reference implementation in docs/znack_api/znack/ZnackModels.java.

use serde::{Deserialize, Serialize};

pub const PRODUCTION_TRUE_API: &str = "https://markirovka.crpt.ru/api/v3/true-api";
pub const PRODUCTION_SUZ: &str = "https://suzgrid.crpt.ru";

/// Per-store integration settings (single row in znack_settings).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ZnackSettings {
    pub true_api_base_url: String,
    pub suz_base_url: String,
    pub oms_id: String,
    pub oms_connection: String,
    pub participant_inn: String,
    pub producer_inn: String,
    pub owner_inn: String,
    pub cryptcp_path: String,
    pub cert_thumbprint: String,
    pub cert_label: String,
    pub cryptopro_timeout_seconds: i64,
    pub document_type: String,
    pub document_number: String,
    pub document_date: String,
    pub auto_introduction: bool,
}

impl ZnackSettings {
    pub fn resolved_true_api_base(&self) -> String {
        let v = self.true_api_base_url.trim();
        if v.is_empty() { PRODUCTION_TRUE_API.to_string() } else { v.trim_end_matches('/').to_string() }
    }
    pub fn resolved_suz_base(&self) -> String {
        let v = self.suz_base_url.trim();
        if v.is_empty() { PRODUCTION_SUZ.to_string() } else { v.trim_end_matches('/').to_string() }
    }
    pub fn resolved_timeout_secs(&self) -> u64 {
        if self.cryptopro_timeout_seconds <= 0 { 60 } else { self.cryptopro_timeout_seconds.min(600) as u64 }
    }
    pub fn resolved_document_type(&self) -> String {
        let v = self.document_type.trim();
        if v.is_empty() { "CONFORMITY_DECLARATION".to_string() } else { v.to_string() }
    }
    /// Introduction runs automatically iff the conformity document is filled in.
    pub fn has_goods_document(&self) -> bool {
        !self.document_number.trim().is_empty() && !self.document_date.trim().is_empty()
    }
    pub fn require_signer(&self) -> crate::error::AppResult<()> {
        if self.cert_thumbprint.trim().is_empty() {
            return Err(crate::error::AppError::Msg(
                "Chưa cấu hình chứng thư CryptoPro (thumbprint) trong cài đặt Честный ЗНАК.".into(),
            ));
        }
        Ok(())
    }
    pub fn require_oms(&self) -> crate::error::AppResult<()> {
        self.require_signer()?;
        if self.oms_id.trim().is_empty() || self.oms_connection.trim().is_empty() {
            return Err(crate::error::AppError::Msg(
                "Cần omsId và omsConnection trước khi làm việc với СУЗ.".into(),
            ));
        }
        Ok(())
    }
}

/// A goods card synced from Znack (National Catalog via True API).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZnackProduct {
    pub gtin: String,
    pub product_name: String,
    pub tn_ved: String,
    pub good_mark_flag: Option<bool>,
    pub good_turn_flag: Option<bool>,
    pub card_status: String,
    pub card_detailed_status: String,
    pub synced_at: String,
    // Inventory counts (filled by list command).
    pub available: i64,
    pub reserved: i64,
    pub consumed: i64,
    // Active purchase pipeline summary, if any.
    pub pipeline: Option<ZnackPipeline>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZnackMappingRule {
    pub id: i64,
    pub gtin: String,
    pub subject_name: String,
    pub gender_value: String,
    pub wildcard_gender: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZnackKizOrder {
    pub id: i64,
    pub external_order_id: Option<String>,
    pub gtin: String,
    pub quantity: i64,
    pub remote_status: Option<String>,
    pub local_status: String,
    pub error_message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZnackPipeline {
    pub id: i64,
    pub gtin: String,
    pub quantity: i64,
    pub order_id: Option<i64>,
    pub stage: String,
    pub error_message: Option<String>,
    pub updated_at: String,
}

/// Purchase pipeline stages (stored as TEXT).
pub mod stage {
    pub const VALIDATING: &str = "VALIDATING";
    pub const CREATING_ORDER: &str = "CREATING_ORDER";
    pub const POLLING_ORDER: &str = "POLLING_ORDER";
    pub const DOWNLOADING_CODES: &str = "DOWNLOADING_CODES";
    pub const WAITING_INTRODUCTION_READINESS: &str = "WAITING_INTRODUCTION_READINESS";
    pub const SUBMITTING_INTRODUCTION: &str = "SUBMITTING_INTRODUCTION";
    pub const POLLING_INTRODUCTION: &str = "POLLING_INTRODUCTION";
    pub const INTRODUCED: &str = "INTRODUCED";
    pub const COMPLETED: &str = "COMPLETED";
    pub const FAILED: &str = "FAILED";

    pub fn is_active(stage: &str) -> bool {
        !matches!(stage, INTRODUCED | COMPLETED | FAILED)
    }
}

/// kiz_codes.legal_status — Честный ЗНАК lifecycle. (Inventory lifecycle
/// AVAILABLE/RESERVED/CONSUMED and legal RECEIVED/PRINTED live in SQL only.)
pub mod legal_status {
    pub const INTRO_SENT: &str = "INTRO_SENT";
    pub const IN_CIRCULATION: &str = "IN_CIRCULATION";
}

/// Normalize a GTIN to 14 digits (docs/znack_api/znack/GtinNormalizer.java).
pub fn normalize_gtin(value: &str) -> crate::error::AppResult<String> {
    let gtin = value.trim();
    if gtin.is_empty() || gtin.len() > 14 || !gtin.bytes().all(|b| b.is_ascii_digit()) {
        return Err(crate::error::AppError::Msg("GTIN phải là tối đa 14 chữ số.".into()));
    }
    Ok(format!("{:0>14}", gtin))
}

pub fn require_production_orderable(value: &str) -> crate::error::AppResult<String> {
    let gtin = normalize_gtin(value)?;
    if gtin.starts_with("029") {
        return Err(crate::error::AppError::Msg(
            "GTIN kỹ thuật (dải 0290–0299) không thể dùng để mua KIZ PRODUCTION.".into(),
        ));
    }
    Ok(gtin)
}

/// Convert a downloaded GS1 DataMatrix payload into the normalized CIS form
/// required by True API (docs/znack_api/znack/ZnackCisNormalizer.java).
/// The original payload must still be retained for printing.
pub fn cis_for_true_api(raw_code: &str) -> String {
    const GS: char = '\u{1d}';
    const SERIAL_OFFSET: usize = 2 + 14 + 2; // 01 + gtin + 21
    const LP_NORMALIZED_LEN: usize = SERIAL_OFFSET + 13; // 13-char lp serial

    let mut code = raw_code.trim();
    if let Some(stripped) = code.strip_prefix("]d2") {
        code = stripped;
    }
    let code = code.trim_start_matches(GS);

    let is_unit = code.len() >= SERIAL_OFFSET
        && code.starts_with("01")
        && code.get(2..16).map(|g| g.bytes().all(|b| b.is_ascii_digit())) == Some(true)
        && code.get(16..18) == Some("21");
    if !is_unit {
        return code.to_string();
    }
    if let Some(pos) = code[SERIAL_OFFSET..].find(GS) {
        return code[..SERIAL_OFFSET + pos].to_string();
    }
    if code.len() > LP_NORMALIZED_LEN + 6
        && code.get(LP_NORMALIZED_LEN..LP_NORMALIZED_LEN + 2) == Some("91")
        && (code[LP_NORMALIZED_LEN..].contains(GS)
            || code.get(LP_NORMALIZED_LEN + 6..LP_NORMALIZED_LEN + 8) == Some("92"))
    {
        return code[..LP_NORMALIZED_LEN].to_string();
    }
    code.to_string()
}

/// SGTIN display form ("gtin serial") extracted from a raw code, for UI/labels.
pub fn sgtin_display(raw_code: &str) -> String {
    let cis = cis_for_true_api(raw_code);
    if cis.len() >= 18 && cis.starts_with("01") {
        format!("{} {}", &cis[2..16], &cis[18..])
    } else {
        cis
    }
}
