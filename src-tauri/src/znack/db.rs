//! SQLite storage for the Znack integration (per-store database, so no
//! shop scoping is needed — the active store's DB holds its own rows).

use crate::error::{AppError, AppResult};
use crate::znack::models::*;
use rusqlite::{params, Connection, OptionalExtension};

pub const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS znack_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  true_api_base_url TEXT NOT NULL DEFAULT '',
  suz_base_url TEXT NOT NULL DEFAULT '',
  oms_id TEXT NOT NULL DEFAULT '',
  oms_connection TEXT NOT NULL DEFAULT '',
  participant_inn TEXT NOT NULL DEFAULT '',
  producer_inn TEXT NOT NULL DEFAULT '',
  owner_inn TEXT NOT NULL DEFAULT '',
  cryptcp_path TEXT NOT NULL DEFAULT '',
  cert_thumbprint TEXT NOT NULL DEFAULT '',
  cert_label TEXT NOT NULL DEFAULT '',
  cryptopro_timeout_seconds INTEGER NOT NULL DEFAULT 60,
  document_type TEXT NOT NULL DEFAULT '',
  document_number TEXT NOT NULL DEFAULT '',
  document_date TEXT NOT NULL DEFAULT '',
  auto_introduction INTEGER NOT NULL DEFAULT 0,
  updated_at TEXT
);
CREATE TABLE IF NOT EXISTS znack_products (
  gtin TEXT PRIMARY KEY,
  product_name TEXT NOT NULL DEFAULT '',
  tn_ved TEXT NOT NULL DEFAULT '',
  good_mark_flag INTEGER,
  good_turn_flag INTEGER,
  card_status TEXT NOT NULL DEFAULT '',
  card_detailed_status TEXT NOT NULL DEFAULT '',
  synced_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS kiz_orders (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  external_order_id TEXT UNIQUE,
  gtin TEXT NOT NULL,
  quantity INTEGER NOT NULL,
  remote_status TEXT,
  local_status TEXT NOT NULL,
  error_message TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS kiz_codes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  order_id INTEGER NOT NULL,
  raw_code TEXT NOT NULL UNIQUE,
  gtin TEXT NOT NULL,
  block_id TEXT,
  status TEXT NOT NULL DEFAULT 'AVAILABLE',
  legal_status TEXT NOT NULL DEFAULT 'RECEIVED',
  reservation_token TEXT,
  reserved_at TEXT,
  consumed_at TEXT,
  assigned_order INTEGER,
  document_id INTEGER,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_kiz_codes_gtin_status ON kiz_codes(gtin, status, id);
CREATE INDEX IF NOT EXISTS idx_kiz_codes_order ON kiz_codes(order_id);
CREATE TABLE IF NOT EXISTS znack_documents (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  order_id INTEGER NOT NULL,
  payload_json TEXT NOT NULL,
  external_document_id TEXT,
  status TEXT NOT NULL,
  error_message TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_znack_documents_order ON znack_documents(order_id, id DESC);
CREATE TABLE IF NOT EXISTS znack_gtin_mapping_rules (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  gtin TEXT NOT NULL,
  subject_name TEXT NOT NULL,
  gender_value TEXT NOT NULL DEFAULT '',
  wildcard_gender INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(subject_name, gender_value)
);
CREATE TABLE IF NOT EXISTS znack_purchase_pipelines (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  gtin TEXT NOT NULL,
  quantity INTEGER NOT NULL,
  order_id INTEGER,
  stage TEXT NOT NULL,
  error_message TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_znack_pipelines_stage ON znack_purchase_pipelines(stage, updated_at DESC);
"#;

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

// --- settings ----------------------------------------------------------------

pub fn get_settings(conn: &Connection) -> AppResult<ZnackSettings> {
    let row = conn
        .query_row(
            "SELECT true_api_base_url, suz_base_url, oms_id, oms_connection, participant_inn,
                    producer_inn, owner_inn, cryptcp_path, cert_thumbprint,
                    cryptopro_timeout_seconds, document_type, document_number, document_date,
                    auto_introduction, COALESCE(cert_label,'')
             FROM znack_settings WHERE id = 1",
            [],
            |r| {
                Ok(ZnackSettings {
                    true_api_base_url: r.get(0)?,
                    suz_base_url: r.get(1)?,
                    oms_id: r.get(2)?,
                    oms_connection: r.get(3)?,
                    participant_inn: r.get(4)?,
                    producer_inn: r.get(5)?,
                    owner_inn: r.get(6)?,
                    cryptcp_path: r.get(7)?,
                    cert_thumbprint: r.get(8)?,
                    cryptopro_timeout_seconds: r.get(9)?,
                    document_type: r.get(10)?,
                    document_number: r.get(11)?,
                    document_date: r.get(12)?,
                    auto_introduction: r.get::<_, i64>(13)? != 0,
                    cert_label: r.get(14)?,
                })
            },
        )
        .optional()?;
    Ok(row.unwrap_or_default())
}

pub fn save_settings(conn: &Connection, s: &ZnackSettings) -> AppResult<()> {
    conn.execute(
        "INSERT INTO znack_settings(id, true_api_base_url, suz_base_url, oms_id, oms_connection,
            participant_inn, producer_inn, owner_inn, cryptcp_path, cert_thumbprint, cert_label,
            cryptopro_timeout_seconds, document_type, document_number, document_date,
            auto_introduction, updated_at)
         VALUES(1,?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)
         ON CONFLICT(id) DO UPDATE SET
           true_api_base_url=excluded.true_api_base_url, suz_base_url=excluded.suz_base_url,
           oms_id=excluded.oms_id, oms_connection=excluded.oms_connection,
           participant_inn=excluded.participant_inn, producer_inn=excluded.producer_inn,
           owner_inn=excluded.owner_inn, cryptcp_path=excluded.cryptcp_path,
           cert_thumbprint=excluded.cert_thumbprint, cert_label=excluded.cert_label,
           cryptopro_timeout_seconds=excluded.cryptopro_timeout_seconds,
           document_type=excluded.document_type, document_number=excluded.document_number,
           document_date=excluded.document_date, auto_introduction=excluded.auto_introduction,
           updated_at=excluded.updated_at",
        params![
            s.true_api_base_url.trim(),
            s.suz_base_url.trim(),
            s.oms_id.trim(),
            s.oms_connection.trim(),
            s.participant_inn.trim(),
            s.producer_inn.trim(),
            s.owner_inn.trim(),
            s.cryptcp_path.trim(),
            s.cert_thumbprint.trim(),
            s.cert_label.trim(),
            s.cryptopro_timeout_seconds,
            s.document_type.trim(),
            s.document_number.trim(),
            s.document_date.trim(),
            s.auto_introduction as i64,
            now_iso()
        ],
    )?;
    Ok(())
}

// --- products ------------------------------------------------------------------

pub struct ProductUpsert {
    pub gtin: String,
    pub product_name: String,
    pub tn_ved: String,
    pub good_mark_flag: Option<bool>,
    pub good_turn_flag: Option<bool>,
    pub card_status: String,
    pub card_detailed_status: String,
}

pub fn upsert_product(conn: &Connection, p: &ProductUpsert) -> AppResult<()> {
    conn.execute(
        "INSERT INTO znack_products(gtin, product_name, tn_ved, good_mark_flag, good_turn_flag,
            card_status, card_detailed_status, synced_at)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8)
         ON CONFLICT(gtin) DO UPDATE SET
           product_name=CASE WHEN excluded.product_name<>'' THEN excluded.product_name ELSE znack_products.product_name END,
           tn_ved=CASE WHEN excluded.tn_ved<>'' THEN excluded.tn_ved ELSE znack_products.tn_ved END,
           good_mark_flag=COALESCE(excluded.good_mark_flag, znack_products.good_mark_flag),
           good_turn_flag=COALESCE(excluded.good_turn_flag, znack_products.good_turn_flag),
           card_status=CASE WHEN excluded.card_status<>'' THEN excluded.card_status ELSE znack_products.card_status END,
           card_detailed_status=CASE WHEN excluded.card_detailed_status<>'' THEN excluded.card_detailed_status ELSE znack_products.card_detailed_status END,
           synced_at=excluded.synced_at",
        params![
            p.gtin,
            p.product_name,
            p.tn_ved,
            p.good_mark_flag.map(|b| b as i64),
            p.good_turn_flag.map(|b| b as i64),
            p.card_status,
            p.card_detailed_status,
            now_iso()
        ],
    )?;
    Ok(())
}

pub fn list_products(conn: &Connection) -> AppResult<Vec<ZnackProduct>> {
    let mut stmt = conn.prepare(
        "SELECT p.gtin, p.product_name, p.tn_ved, p.good_mark_flag, p.good_turn_flag,
                p.card_status, p.card_detailed_status, p.synced_at,
                COALESCE((SELECT COUNT(*) FROM kiz_codes c WHERE c.gtin=p.gtin AND c.status='AVAILABLE'),0),
                COALESCE((SELECT COUNT(*) FROM kiz_codes c WHERE c.gtin=p.gtin AND c.status='RESERVED'),0),
                COALESCE((SELECT COUNT(*) FROM kiz_codes c WHERE c.gtin=p.gtin AND c.status='CONSUMED'),0)
         FROM znack_products p ORDER BY p.product_name, p.gtin",
    )?;
    let mut products: Vec<ZnackProduct> = stmt
        .query_map([], |r| {
            Ok(ZnackProduct {
                gtin: r.get(0)?,
                product_name: r.get(1)?,
                tn_ved: r.get(2)?,
                good_mark_flag: r.get::<_, Option<i64>>(3)?.map(|v| v != 0),
                good_turn_flag: r.get::<_, Option<i64>>(4)?.map(|v| v != 0),
                card_status: r.get(5)?,
                card_detailed_status: r.get(6)?,
                synced_at: r.get(7)?,
                available: r.get(8)?,
                reserved: r.get(9)?,
                consumed: r.get(10)?,
                pipeline: None,
            })
        })?
        .collect::<Result<_, _>>()?;
    for p in &mut products {
        p.pipeline = find_active_pipeline(conn, &p.gtin)?;
    }
    Ok(products)
}

pub fn product_exists(conn: &Connection, gtin: &str) -> AppResult<bool> {
    Ok(conn
        .query_row("SELECT 1 FROM znack_products WHERE gtin=?1", params![gtin], |_| Ok(()))
        .optional()?
        .is_some())
}

pub fn product_tnved(conn: &Connection, gtin: &str) -> AppResult<String> {
    Ok(conn
        .query_row("SELECT tn_ved FROM znack_products WHERE gtin=?1", params![gtin], |r| r.get(0))
        .optional()?
        .unwrap_or_default())
}

pub fn product_flags(conn: &Connection, gtin: &str) -> AppResult<(Option<bool>, Option<bool>)> {
    Ok(conn
        .query_row(
            "SELECT good_mark_flag, good_turn_flag FROM znack_products WHERE gtin=?1",
            params![gtin],
            |r| {
                Ok((
                    r.get::<_, Option<i64>>(0)?.map(|v| v != 0),
                    r.get::<_, Option<i64>>(1)?.map(|v| v != 0),
                ))
            },
        )
        .optional()?
        .unwrap_or((None, None)))
}

// --- mapping rules -------------------------------------------------------------

pub fn list_rules(conn: &Connection) -> AppResult<Vec<ZnackMappingRule>> {
    let mut stmt = conn.prepare(
        "SELECT id, gtin, subject_name, gender_value, wildcard_gender
         FROM znack_gtin_mapping_rules ORDER BY subject_name, gender_value",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(ZnackMappingRule {
                id: r.get(0)?,
                gtin: r.get(1)?,
                subject_name: r.get(2)?,
                gender_value: r.get(3)?,
                wildcard_gender: r.get::<_, i64>(4)? != 0,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

pub fn save_rule(
    conn: &Connection,
    gtin: &str,
    subject_name: &str,
    gender_value: &str,
    wildcard_gender: bool,
) -> AppResult<()> {
    let subject = subject_name.trim();
    if subject.is_empty() {
        return Err(AppError::Msg("Danh mục không được để trống.".into()));
    }
    let gender = if wildcard_gender { "" } else { gender_value.trim() };
    conn.execute(
        "INSERT INTO znack_gtin_mapping_rules(gtin, subject_name, gender_value, wildcard_gender, created_at, updated_at)
         VALUES(?1,?2,?3,?4,?5,?5)
         ON CONFLICT(subject_name, gender_value) DO UPDATE SET
           gtin=excluded.gtin, wildcard_gender=excluded.wildcard_gender, updated_at=excluded.updated_at",
        params![gtin, subject, gender, wildcard_gender as i64, now_iso()],
    )?;
    Ok(())
}

pub fn delete_rule(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM znack_gtin_mapping_rules WHERE id=?1", params![id])?;
    Ok(())
}

/// Resolve the GTIN for a WB order by its category and gender:
/// exact (subject, gender) rule → wildcard-gender rule → single rule fallback.
pub fn resolve_gtin(conn: &Connection, subject_name: &str, gender: &str) -> AppResult<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT gtin, gender_value, wildcard_gender FROM znack_gtin_mapping_rules WHERE subject_name=?1",
    )?;
    let rules: Vec<(String, String, bool)> = stmt
        .query_map(params![subject_name], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get::<_, i64>(2)? != 0))
        })?
        .collect::<Result<_, _>>()?;
    if rules.is_empty() {
        return Ok(None);
    }
    let gender = gender.trim();
    // Exact gender match first — including the "" gender for products
    // without a Пол characteristic.
    if let Some((g, _, _)) = rules
        .iter()
        .find(|(_, gv, w)| !w && gv.trim().eq_ignore_ascii_case(gender))
    {
        return Ok(Some(g.clone()));
    }
    if let Some((g, _, _)) = rules.iter().find(|(_, _, w)| *w) {
        return Ok(Some(g.clone()));
    }
    if rules.len() == 1 && gender.is_empty() {
        return Ok(Some(rules[0].0.clone()));
    }
    Ok(None)
}

/// Replace all links between `gtin` and `subject_name`: delete this GTIN's
/// rules for the category, then insert either one wildcard rule (all genders)
/// or one rule per selected gender. Upserts steal (subject, gender) slots from
/// other GTINs, keeping resolution unambiguous.
pub fn apply_mapping(
    conn: &Connection,
    gtin: &str,
    subject_name: &str,
    genders: &[String],
    all_genders: bool,
) -> AppResult<()> {
    let subject = subject_name.trim();
    if subject.is_empty() {
        return Err(AppError::Msg("Danh mục không được để trống.".into()));
    }
    let now = now_iso();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "DELETE FROM znack_gtin_mapping_rules WHERE subject_name=?1 AND gtin=?2",
        params![subject, gtin],
    )?;
    if all_genders {
        tx.execute(
            "INSERT INTO znack_gtin_mapping_rules(gtin, subject_name, gender_value, wildcard_gender, created_at, updated_at)
             VALUES(?1,?2,'',1,?3,?3)
             ON CONFLICT(subject_name, gender_value) DO UPDATE SET
               gtin=excluded.gtin, wildcard_gender=1, updated_at=excluded.updated_at",
            params![gtin, subject, now],
        )?;
    } else {
        for gender in genders {
            tx.execute(
                "INSERT INTO znack_gtin_mapping_rules(gtin, subject_name, gender_value, wildcard_gender, created_at, updated_at)
                 VALUES(?1,?2,?3,0,?4,?4)
                 ON CONFLICT(subject_name, gender_value) DO UPDATE SET
                   gtin=excluded.gtin, wildcard_gender=0, updated_at=excluded.updated_at",
                params![gtin, subject, gender.trim(), now],
            )?;
        }
    }
    tx.commit()?;
    Ok(())
}

// --- kiz orders ----------------------------------------------------------------

pub fn create_order_draft(conn: &Connection, gtin: &str, quantity: i64) -> AppResult<i64> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO kiz_orders(gtin, quantity, local_status, created_at, updated_at)
         VALUES(?1,?2,'DRAFT',?3,?3)",
        params![gtin, quantity, now],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_order(
    conn: &Connection,
    id: i64,
    external_order_id: Option<&str>,
    remote_status: Option<&str>,
    local_status: Option<&str>,
    error_message: Option<&str>,
) -> AppResult<()> {
    conn.execute(
        "UPDATE kiz_orders SET
           external_order_id=COALESCE(?2, external_order_id),
           remote_status=COALESCE(?3, remote_status),
           local_status=COALESCE(?4, local_status),
           error_message=?5,
           updated_at=?6
         WHERE id=?1",
        params![id, external_order_id, remote_status, local_status, error_message, now_iso()],
    )?;
    Ok(())
}

pub fn find_order(conn: &Connection, id: i64) -> AppResult<ZnackKizOrder> {
    conn.query_row(
        "SELECT id, external_order_id, gtin, quantity, remote_status, local_status, error_message, created_at
         FROM kiz_orders WHERE id=?1",
        params![id],
        |r| {
            Ok(ZnackKizOrder {
                id: r.get(0)?,
                external_order_id: r.get(1)?,
                gtin: r.get(2)?,
                quantity: r.get(3)?,
                remote_status: r.get(4)?,
                local_status: r.get(5)?,
                error_message: r.get(6)?,
                created_at: r.get(7)?,
            })
        },
    )
    .map_err(AppError::from)
}

// --- kiz codes -------------------------------------------------------------------

pub fn insert_codes(
    conn: &Connection,
    order_id: i64,
    gtin: &str,
    codes: &[String],
    block_id: Option<&str>,
) -> AppResult<i64> {
    let now = now_iso();
    let tx = conn.unchecked_transaction()?;
    let mut inserted = 0i64;
    for code in codes {
        let n = tx.execute(
            "INSERT OR IGNORE INTO kiz_codes(order_id, raw_code, gtin, block_id, status, legal_status, created_at, updated_at)
             VALUES(?1,?2,?3,?4,'AVAILABLE','RECEIVED',?5,?5)",
            params![order_id, code, gtin, block_id, now],
        )?;
        inserted += n as i64;
    }
    tx.commit()?;
    Ok(inserted)
}

pub fn count_codes(conn: &Connection, order_id: i64) -> AppResult<i64> {
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM kiz_codes WHERE order_id=?1",
        params![order_id],
        |r| r.get(0),
    )?)
}

pub fn codes_for_order(conn: &Connection, order_id: i64) -> AppResult<Vec<(i64, String)>> {
    let mut stmt =
        conn.prepare("SELECT id, raw_code FROM kiz_codes WHERE order_id=?1 ORDER BY id")?;
    let rows = stmt
        .query_map(params![order_id], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

pub fn mark_codes_legal(
    conn: &Connection,
    order_id: i64,
    legal: &str,
    document_id: Option<i64>,
) -> AppResult<()> {
    conn.execute(
        "UPDATE kiz_codes SET legal_status=?2, document_id=COALESCE(?3, document_id), updated_at=?4
         WHERE order_id=?1",
        params![order_id, legal, document_id, now_iso()],
    )?;
    Ok(())
}

/// Get a code for one WB order. Reuses the code already assigned to this
/// order on a previous print (so reprints never consume a second paid code);
/// otherwise reserves the oldest AVAILABLE code under the shared print token.
/// Returns (raw_code, newly_reserved).
pub fn code_for_order(
    conn: &Connection,
    gtin: &str,
    order_id: i64,
    token: &str,
) -> AppResult<(String, bool)> {
    // Reuse a code previously printed/reserved for this very order.
    if let Some(raw) = conn
        .query_row(
            "SELECT raw_code FROM kiz_codes
             WHERE assigned_order=?1 AND gtin=?2 AND status IN ('CONSUMED','RESERVED')
             ORDER BY id LIMIT 1",
            params![order_id, gtin],
            |r| r.get::<_, String>(0),
        )
        .optional()?
    {
        return Ok((raw, false));
    }
    let now = now_iso();
    let picked = conn
        .query_row(
            "SELECT id, raw_code FROM kiz_codes
             WHERE gtin=?1 AND status='AVAILABLE' ORDER BY id LIMIT 1",
            params![gtin],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
        )
        .optional()?;
    let Some((id, raw)) = picked else {
        return Err(AppError::Msg(format!("Hết mã KIZ cho GTIN {gtin}.")));
    };
    let updated = conn.execute(
        "UPDATE kiz_codes SET status='RESERVED', reservation_token=?2, reserved_at=?3,
                assigned_order=?4, updated_at=?3
         WHERE id=?1 AND status='AVAILABLE'",
        params![id, token, now, order_id],
    )?;
    if updated != 1 {
        return Err(AppError::Msg("Kho KIZ thay đổi trong lúc giữ mã, thử lại.".into()));
    }
    Ok((raw, true))
}

/// Reserve `quantity` AVAILABLE codes for a GTIN under a shared token, without
/// an order link (FBO printing). Fails if there are not enough codes.
pub fn reserve_bulk(
    conn: &Connection,
    gtin: &str,
    quantity: i64,
    token: &str,
) -> AppResult<Vec<String>> {
    let tx = conn.unchecked_transaction()?;
    let selected: Vec<(i64, String)> = {
        let mut stmt = tx.prepare(
            "SELECT id, raw_code FROM kiz_codes
             WHERE gtin=?1 AND status='AVAILABLE' ORDER BY id LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(params![gtin, quantity], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };
    if (selected.len() as i64) < quantity {
        return Err(AppError::Msg(format!(
            "Không đủ mã KIZ cho GTIN {gtin}: cần {quantity}, còn {}.",
            selected.len()
        )));
    }
    let now = now_iso();
    for (id, _) in &selected {
        let updated = tx.execute(
            "UPDATE kiz_codes SET status='RESERVED', reservation_token=?2, reserved_at=?3, updated_at=?3
             WHERE id=?1 AND status='AVAILABLE'",
            params![id, token, now],
        )?;
        if updated != 1 {
            return Err(AppError::Msg("Kho KIZ thay đổi trong lúc giữ mã, thử lại.".into()));
        }
    }
    tx.commit()?;
    Ok(selected.into_iter().map(|(_, raw)| raw).collect())
}

/// Commit (CONSUMED + PRINTED, keeping the order link) or release (AVAILABLE,
/// clearing the order link) all codes under a token.
pub fn finish_reservation(conn: &Connection, token: &str, consume: bool) -> AppResult<i64> {
    let now = now_iso();
    let n = if consume {
        conn.execute(
            "UPDATE kiz_codes SET status='CONSUMED', legal_status=CASE WHEN legal_status='RECEIVED' THEN 'PRINTED' ELSE legal_status END,
                    consumed_at=?2, reservation_token=NULL, reserved_at=NULL, updated_at=?2
             WHERE reservation_token=?1 AND status='RESERVED'",
            params![token, now],
        )?
    } else {
        conn.execute(
            "UPDATE kiz_codes SET status='AVAILABLE', reservation_token=NULL, reserved_at=NULL,
                    assigned_order=NULL, updated_at=?2
             WHERE reservation_token=?1 AND status='RESERVED'",
            params![token, now],
        )?
    };
    Ok(n as i64)
}

/// Startup recovery: release reservations left behind by an interrupted print.
pub fn release_stale_reservations(conn: &Connection) -> AppResult<i64> {
    let n = conn.execute(
        "UPDATE kiz_codes SET status='AVAILABLE', reservation_token=NULL, reserved_at=NULL,
                assigned_order=NULL, updated_at=?1
         WHERE status='RESERVED'",
        params![now_iso()],
    )?;
    Ok(n as i64)
}

// --- documents -------------------------------------------------------------------

pub fn create_document(conn: &Connection, order_id: i64, payload_json: &str) -> AppResult<i64> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO znack_documents(order_id, payload_json, status, created_at, updated_at)
         VALUES(?1,?2,'DRAFT',?3,?3)",
        params![order_id, payload_json, now],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_document(
    conn: &Connection,
    id: i64,
    external_document_id: Option<&str>,
    status: &str,
    error_message: Option<&str>,
) -> AppResult<()> {
    conn.execute(
        "UPDATE znack_documents SET
           external_document_id=COALESCE(?2, external_document_id),
           status=?3, error_message=?4, updated_at=?5
         WHERE id=?1",
        params![id, external_document_id, status, error_message, now_iso()],
    )?;
    Ok(())
}

pub fn latest_document(
    conn: &Connection,
    order_id: i64,
) -> AppResult<Option<(i64, Option<String>, String)>> {
    Ok(conn
        .query_row(
            "SELECT id, external_document_id, status FROM znack_documents
             WHERE order_id=?1 ORDER BY id DESC LIMIT 1",
            params![order_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?)
}

// --- pipelines --------------------------------------------------------------------

pub fn create_pipeline(conn: &Connection, gtin: &str, quantity: i64) -> AppResult<i64> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO znack_purchase_pipelines(gtin, quantity, stage, created_at, updated_at)
         VALUES(?1,?2,'VALIDATING',?3,?3)",
        params![gtin, quantity, now],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_pipeline(
    conn: &Connection,
    id: i64,
    order_id: Option<i64>,
    stage: &str,
    error_message: Option<&str>,
) -> AppResult<()> {
    conn.execute(
        "UPDATE znack_purchase_pipelines SET
           order_id=COALESCE(?2, order_id), stage=?3, error_message=?4, updated_at=?5
         WHERE id=?1",
        params![id, order_id, stage, error_message, now_iso()],
    )?;
    Ok(())
}

fn map_pipeline(r: &rusqlite::Row) -> rusqlite::Result<ZnackPipeline> {
    Ok(ZnackPipeline {
        id: r.get(0)?,
        gtin: r.get(1)?,
        quantity: r.get(2)?,
        order_id: r.get(3)?,
        stage: r.get(4)?,
        error_message: r.get(5)?,
        updated_at: r.get(6)?,
    })
}

const PIPELINE_COLS: &str = "id, gtin, quantity, order_id, stage, error_message, updated_at";
const ACTIVE_STAGES: &str = "('VALIDATING','CREATING_ORDER','POLLING_ORDER','DOWNLOADING_CODES',\
     'WAITING_INTRODUCTION_READINESS','SUBMITTING_INTRODUCTION','POLLING_INTRODUCTION')";

pub fn find_pipeline(conn: &Connection, id: i64) -> AppResult<Option<ZnackPipeline>> {
    Ok(conn
        .query_row(
            &format!("SELECT {PIPELINE_COLS} FROM znack_purchase_pipelines WHERE id=?1"),
            params![id],
            map_pipeline,
        )
        .optional()?)
}

pub fn find_active_pipeline(conn: &Connection, gtin: &str) -> AppResult<Option<ZnackPipeline>> {
    Ok(conn
        .query_row(
            &format!(
                "SELECT {PIPELINE_COLS} FROM znack_purchase_pipelines
                 WHERE gtin=?1 AND stage IN {ACTIVE_STAGES} ORDER BY id DESC LIMIT 1"
            ),
            params![gtin],
            map_pipeline,
        )
        .optional()?)
}

pub fn active_pipelines(conn: &Connection) -> AppResult<Vec<ZnackPipeline>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {PIPELINE_COLS} FROM znack_purchase_pipelines WHERE stage IN {ACTIVE_STAGES} ORDER BY id"
    ))?;
    let rows = stmt.query_map([], map_pipeline)?.collect::<Result<_, _>>()?;
    Ok(rows)
}

pub fn list_pipelines(conn: &Connection, limit: i64) -> AppResult<Vec<ZnackPipeline>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {PIPELINE_COLS} FROM znack_purchase_pipelines ORDER BY id DESC LIMIT ?1"
    ))?;
    let rows = stmt.query_map(params![limit], map_pipeline)?.collect::<Result<_, _>>()?;
    Ok(rows)
}
