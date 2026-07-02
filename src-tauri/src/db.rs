//! SQLite storage: schema, upserts, and query helpers.

use crate::error::AppResult;
use crate::models::*;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS cards (
  nm_id INTEGER PRIMARY KEY,
  imt_id INTEGER,
  vendor_code TEXT,
  title TEXT,
  brand TEXT,
  subject_name TEXT,
  color TEXT,
  gender TEXT,
  photo TEXT,
  updated_at TEXT,
  raw_json TEXT
);
CREATE TABLE IF NOT EXISTS card_sizes (
  chrt_id INTEGER PRIMARY KEY,
  nm_id INTEGER,
  tech_size TEXT,
  wb_size TEXT
);
CREATE INDEX IF NOT EXISTS idx_card_sizes_nm ON card_sizes(nm_id);
CREATE TABLE IF NOT EXISTS skus (
  barcode TEXT PRIMARY KEY,
  chrt_id INTEGER,
  nm_id INTEGER
);
CREATE INDEX IF NOT EXISTS idx_skus_chrt ON skus(chrt_id);
CREATE TABLE IF NOT EXISTS supplies (
  id TEXT PRIMARY KEY,
  name TEXT,
  done INTEGER,
  cargo_type INTEGER,
  pickup INTEGER,
  created_at TEXT,
  closed_at TEXT,
  scan_dt TEXT,
  raw_json TEXT,
  synced_at TEXT
);
CREATE TABLE IF NOT EXISTS orders (
  id INTEGER PRIMARY KEY,
  rid TEXT,
  created_at TEXT,
  nm_id INTEGER,
  chrt_id INTEGER,
  article TEXT,
  color_code TEXT,
  sku TEXT,
  price INTEGER,
  cargo_type INTEGER,
  ddate TEXT,
  supply_id TEXT,
  status TEXT,
  offices TEXT,
  warehouse TEXT,
  delivery_type TEXT,
  pickup INTEGER,
  required_meta TEXT,
  raw_json TEXT,
  synced_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_orders_supply ON orders(supply_id);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
CREATE TABLE IF NOT EXISTS sync_state (
  key TEXT PRIMARY KEY,
  updated_at TEXT,
  nm_id INTEGER
);
CREATE TABLE IF NOT EXISTS app_settings (
  key TEXT PRIMARY KEY,
  value TEXT
);
"#;

pub fn open(path: &Path) -> AppResult<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA foreign_keys=ON;",
    )?;
    conn.execute_batch(SCHEMA)?;
    conn.execute_batch(crate::znack::db::SCHEMA)?;
    // Best-effort migrations for columns added after initial release.
    for stmt in [
        "ALTER TABLE cards ADD COLUMN photo TEXT",
        "ALTER TABLE cards ADD COLUMN gender TEXT",
        "ALTER TABLE znack_settings ADD COLUMN cert_label TEXT",
        "ALTER TABLE kiz_codes ADD COLUMN assigned_order INTEGER",
        "ALTER TABLE orders ADD COLUMN offices TEXT",
        "ALTER TABLE orders ADD COLUMN warehouse TEXT",
        "ALTER TABLE orders ADD COLUMN delivery_type TEXT",
        "ALTER TABLE orders ADD COLUMN pickup INTEGER",
        "ALTER TABLE orders ADD COLUMN required_meta TEXT",
        "ALTER TABLE supplies ADD COLUMN pickup INTEGER",
    ] {
        let _ = conn.execute(stmt, []);
    }
    Ok(conn)
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

// --- cards ------------------------------------------------------------------

fn extract_color(card: &Card) -> String {
    for ch in &card.characteristics {
        let n = ch.name.to_lowercase();
        if n.contains("цвет") || n.contains("color") {
            match &ch.value {
                serde_json::Value::Array(arr) => {
                    let parts: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    if !parts.is_empty() {
                        return parts.join(", ");
                    }
                }
                serde_json::Value::String(s) if !s.is_empty() => return s.clone(),
                _ => {}
            }
        }
    }
    String::new()
}

/// Target gender from the WB card's "Пол" characteristic.
fn extract_gender(card: &Card) -> String {
    for ch in &card.characteristics {
        let n = ch.name.trim().to_lowercase();
        if n == "пол" || n == "gender" {
            match &ch.value {
                serde_json::Value::Array(arr) => {
                    let parts: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                        .filter(|s| !s.is_empty())
                        .collect();
                    if !parts.is_empty() {
                        return parts.join(", ");
                    }
                }
                serde_json::Value::String(s) if !s.trim().is_empty() => {
                    return s.trim().to_string()
                }
                _ => {}
            }
        }
    }
    String::new()
}

fn extract_photo(card: &Card) -> String {
    for p in &card.photos {
        for candidate in [&p.c246x328, &p.big, &p.square] {
            if !candidate.is_empty() {
                return candidate.clone();
            }
        }
    }
    String::new()
}

pub fn upsert_card(conn: &Connection, card: &Card) -> AppResult<()> {
    let color = extract_color(card);
    let photo = extract_photo(card);
    let gender = extract_gender(card);
    let raw = serde_json::to_string(&serde_json::json!({
        "nmID": card.nm_id, "vendorCode": card.vendor_code
    }))
    .unwrap_or_default();
    conn.execute(
        "INSERT INTO cards(nm_id, imt_id, vendor_code, title, brand, subject_name, color, gender, photo, updated_at, raw_json)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)
         ON CONFLICT(nm_id) DO UPDATE SET
           imt_id=excluded.imt_id, vendor_code=excluded.vendor_code, title=excluded.title,
           brand=excluded.brand, subject_name=excluded.subject_name, color=excluded.color,
           gender=excluded.gender, photo=excluded.photo, updated_at=excluded.updated_at,
           raw_json=excluded.raw_json",
        params![
            card.nm_id,
            card.imt_id,
            card.vendor_code,
            card.title,
            card.brand,
            card.subject_name,
            color,
            gender,
            photo,
            card.updated_at,
            raw
        ],
    )?;

    for size in &card.sizes {
        conn.execute(
            "INSERT INTO card_sizes(chrt_id, nm_id, tech_size, wb_size) VALUES(?1,?2,?3,?4)
             ON CONFLICT(chrt_id) DO UPDATE SET
               nm_id=excluded.nm_id, tech_size=excluded.tech_size, wb_size=excluded.wb_size",
            params![size.chrt_id, card.nm_id, size.tech_size, size.wb_size],
        )?;
        for sku in &size.skus {
            conn.execute(
                "INSERT INTO skus(barcode, chrt_id, nm_id) VALUES(?1,?2,?3)
                 ON CONFLICT(barcode) DO UPDATE SET chrt_id=excluded.chrt_id, nm_id=excluded.nm_id",
                params![sku, size.chrt_id, card.nm_id],
            )?;
        }
    }
    Ok(())
}

pub fn count_products(conn: &Connection) -> AppResult<i64> {
    Ok(conn.query_row("SELECT COUNT(*) FROM cards", [], |r| r.get(0))?)
}

pub fn list_categories(conn: &Connection) -> AppResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT subject_name FROM cards
         WHERE subject_name IS NOT NULL AND subject_name <> ''
         ORDER BY subject_name",
    )?;
    let rows = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn list_products(
    conn: &Connection,
    search: &str,
    categories: &[String],
    limit: i64,
) -> AppResult<Vec<ProductRow>> {
    use rusqlite::types::Value;
    let mut sql = String::from(
        "SELECT c.nm_id, c.vendor_code, c.title, c.brand, c.subject_name, c.color,
                COALESCE((SELECT GROUP_CONCAT(tech_size, ', ') FROM
                    (SELECT tech_size FROM card_sizes s WHERE s.nm_id=c.nm_id ORDER BY s.chrt_id)
                 ), '') AS sizes,
                COALESCE(c.photo, '') AS photo,
                c.updated_at
         FROM cards c
         WHERE 1=1",
    );
    let mut args: Vec<Value> = Vec::new();
    if !search.is_empty() {
        sql.push_str(
            " AND (c.vendor_code LIKE ? OR CAST(c.nm_id AS TEXT) LIKE ? OR c.title LIKE ?)",
        );
        let like = format!("%{}%", search);
        args.push(Value::Text(like.clone()));
        args.push(Value::Text(like.clone()));
        args.push(Value::Text(like));
    }
    if !categories.is_empty() {
        let ph = categories.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        sql.push_str(&format!(" AND c.subject_name IN ({})", ph));
        for c in categories {
            args.push(Value::Text(c.clone()));
        }
    }
    sql.push_str(" ORDER BY c.updated_at DESC LIMIT ?");
    args.push(Value::Integer(limit));

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(args.iter()), |r| {
            Ok(ProductRow {
                nm_id: r.get(0)?,
                vendor_code: r.get(1)?,
                title: r.get(2)?,
                brand: r.get(3)?,
                subject_name: r.get(4)?,
                color: r.get(5)?,
                sizes: r.get(6)?,
                photo: r.get(7)?,
                updated_at: r.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

// --- orders -----------------------------------------------------------------

pub fn upsert_order(conn: &Connection, o: &WbOrder, status: &str, warehouse: &str) -> AppResult<()> {
    let sku = o.skus.first().cloned().unwrap_or_default();
    let offices = o.offices.join(", ");
    let raw = serde_json::to_string(&serde_json::json!({ "id": o.id, "rid": o.rid })).unwrap_or_default();
    // Preserve an already-known supply_id / more-advanced status if present.
    conn.execute(
        "INSERT INTO orders(id, rid, created_at, nm_id, chrt_id, article, color_code, sku,
                            price, cargo_type, ddate, supply_id, status,
                            offices, warehouse, delivery_type, pickup, required_meta, raw_json, synced_at)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20)
         ON CONFLICT(id) DO UPDATE SET
           rid=excluded.rid, created_at=excluded.created_at, nm_id=excluded.nm_id,
           chrt_id=excluded.chrt_id, article=excluded.article, color_code=excluded.color_code,
           sku=excluded.sku, price=excluded.price, cargo_type=excluded.cargo_type,
           ddate=excluded.ddate,
           supply_id=COALESCE(excluded.supply_id, orders.supply_id),
           status=CASE WHEN orders.status IN ('confirm','complete') THEN orders.status ELSE excluded.status END,
           offices=excluded.offices,
           warehouse=CASE WHEN excluded.warehouse<>'' THEN excluded.warehouse ELSE orders.warehouse END,
           delivery_type=excluded.delivery_type, pickup=excluded.pickup,
           required_meta=excluded.required_meta,
           raw_json=excluded.raw_json, synced_at=excluded.synced_at",
        params![
            o.id,
            o.rid,
            o.created_at,
            o.nm_id,
            o.chrt_id,
            o.article,
            o.color_code,
            sku,
            o.price,
            o.cargo_type,
            o.ddate,
            o.supply_id,
            status,
            offices,
            warehouse,
            o.delivery_type,
            o.is_pickup as i64,
            o.required_meta.join(","),
            raw,
            now_iso()
        ],
    )?;
    Ok(())
}

/// Does WB require a Честный ЗНАК code for this order?
/// None = unknown (order was synced before required_meta existed).
pub fn order_requires_sgtin(conn: &Connection, id: i64) -> Option<bool> {
    conn.query_row(
        "SELECT required_meta FROM orders WHERE id=?1",
        params![id],
        |r| r.get::<_, Option<String>>(0),
    )
    .ok()
    .flatten()
    .map(|meta| meta.split(',').any(|m| m.trim().eq_ignore_ascii_case("sgtin")))
}

pub fn set_orders_supply(
    conn: &Connection,
    ids: &[i64],
    supply_id: &str,
    status: &str,
) -> AppResult<()> {
    let tx = conn.unchecked_transaction()?;
    for id in ids {
        tx.execute(
            "UPDATE orders SET supply_id=?1, status=?2 WHERE id=?3",
            params![supply_id, status, id],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Core fields for building a label: (chrt_id, sku, article, nm_id).
pub fn get_order_core(conn: &Connection, id: i64) -> Option<(i64, String, String, i64)> {
    conn.query_row(
        "SELECT chrt_id, sku, article, nm_id FROM orders WHERE id=?1",
        params![id],
        |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
            ))
        },
    )
    .optional()
    .ok()
    .flatten()
}

pub fn count_new_orders(conn: &Connection) -> AppResult<i64> {
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM orders WHERE status='new'",
        [],
        |r| r.get(0),
    )?)
}

pub fn count_orders_by_status(conn: &Connection, status: &str) -> AppResult<i64> {
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM orders WHERE status=?1",
        params![status],
        |r| r.get(0),
    )?)
}

/// List orders, optionally filtered by status ("" = all) or supply_id (None = any).
pub fn list_orders(
    conn: &Connection,
    status: &str,
    supply_id: Option<&str>,
) -> AppResult<Vec<OrderRow>> {
    let mut sql = String::from(
        "SELECT o.id, o.rid, o.article, o.nm_id, o.chrt_id, o.sku,
                c.title, cs.tech_size, c.color, c.brand, c.photo,
                o.status, o.supply_id, o.cargo_type,
                COALESCE(o.price,0), COALESCE(o.warehouse,''), COALESCE(o.offices,''),
                COALESCE(o.pickup,0), o.created_at, c.subject_name, c.gender, o.required_meta
         FROM orders o
         LEFT JOIN card_sizes cs ON cs.chrt_id = o.chrt_id
         LEFT JOIN cards c ON c.nm_id = cs.nm_id
         WHERE 1=1",
    );
    if !status.is_empty() {
        sql.push_str(" AND o.status = :status");
    }
    match supply_id {
        Some(_) => sql.push_str(" AND o.supply_id = :supply"),
        None => {}
    }
    sql.push_str(" ORDER BY o.created_at DESC LIMIT 2000");

    let mut stmt = conn.prepare(&sql)?;
    let map = |r: &rusqlite::Row| -> rusqlite::Result<OrderRow> {
        Ok(OrderRow {
            id: r.get(0)?,
            rid: r.get(1)?,
            article: r.get(2)?,
            nm_id: r.get(3)?,
            chrt_id: r.get(4)?,
            sku: r.get(5)?,
            title: r.get(6)?,
            tech_size: r.get(7)?,
            color: r.get(8)?,
            brand: r.get(9)?,
            photo: r.get(10)?,
            status: r.get(11)?,
            supply_id: r.get(12)?,
            cargo_type: r.get(13)?,
            price: r.get(14)?,
            warehouse: r.get(15)?,
            offices: r.get(16)?,
            pickup: r.get::<_, i64>(17)? != 0,
            created_at: r.get(18)?,
            subject_name: r.get(19)?,
            gender: r.get(20)?,
            sgtin_required: r
                .get::<_, Option<String>>(21)?
                .map(|m| m.split(',').any(|x| x.trim().eq_ignore_ascii_case("sgtin"))),
        })
    };
    let rows: Vec<OrderRow> = match (status.is_empty(), supply_id) {
        (true, None) => stmt.query_map([], map)?.collect::<Result<_, _>>()?,
        (false, None) => stmt
            .query_map(rusqlite::named_params! {":status": status}, map)?
            .collect::<Result<_, _>>()?,
        (true, Some(s)) => stmt
            .query_map(rusqlite::named_params! {":supply": s}, map)?
            .collect::<Result<_, _>>()?,
        (false, Some(s)) => stmt
            .query_map(
                rusqlite::named_params! {":status": status, ":supply": s},
                map,
            )?
            .collect::<Result<_, _>>()?,
    };
    Ok(rows)
}

/// Product info the label printer needs for one order.
#[derive(Debug, Default)]
pub struct OrderProductInfo {
    pub title: String,
    pub vendor_code: String,
    pub tech_size: String,
    pub color: String,
    pub brand: String,
    pub subject_name: String,
    pub gender: String,
    pub photo: String,
}

pub fn product_for_order(conn: &Connection, chrt_id: i64, sku: &str) -> OrderProductInfo {
    let map = |r: &rusqlite::Row| -> rusqlite::Result<OrderProductInfo> {
        Ok(OrderProductInfo {
            title: r.get(0)?,
            vendor_code: r.get(1)?,
            tech_size: r.get(2)?,
            color: r.get(3)?,
            brand: r.get(4)?,
            subject_name: r.get::<_, Option<String>>(5)?.unwrap_or_default(),
            gender: r.get::<_, Option<String>>(6)?.unwrap_or_default(),
            photo: r.get::<_, Option<String>>(7)?.unwrap_or_default(),
        })
    };
    // Prefer chrt_id join; fall back to sku -> chrt_id.
    let by_chrt = conn
        .query_row(
            "SELECT c.title, c.vendor_code, cs.tech_size, c.color, c.brand,
                    c.subject_name, c.gender, c.photo
             FROM card_sizes cs JOIN cards c ON c.nm_id = cs.nm_id
             WHERE cs.chrt_id = ?1",
            params![chrt_id],
            map,
        )
        .optional()
        .ok()
        .flatten();
    if let Some(v) = by_chrt {
        return v;
    }
    conn.query_row(
        "SELECT c.title, c.vendor_code, cs.tech_size, c.color, c.brand,
                c.subject_name, c.gender, c.photo
         FROM skus k JOIN card_sizes cs ON cs.chrt_id = k.chrt_id
         JOIN cards c ON c.nm_id = cs.nm_id
         WHERE k.barcode = ?1",
        params![sku],
        map,
    )
    .optional()
    .ok()
    .flatten()
    .unwrap_or_default()
}

/// Product size (SKU) rows for FBO barcode printing, with search + category filters.
pub fn list_sku_items(
    conn: &Connection,
    search: &str,
    categories: &[String],
    limit: i64,
) -> AppResult<Vec<SkuItem>> {
    let mut sql = String::from(
        "SELECT c.nm_id, k.barcode, COALESCE(cs.tech_size,''), COALESCE(cs.wb_size,''),
                COALESCE(c.title,''),
                COALESCE(c.brand,''), COALESCE(c.subject_name,''), COALESCE(c.gender,''),
                COALESCE(c.color,''), COALESCE(c.vendor_code,''), COALESCE(c.photo,'')
         FROM skus k
         JOIN card_sizes cs ON cs.chrt_id = k.chrt_id
         JOIN cards c ON c.nm_id = cs.nm_id
         WHERE 1=1",
    );
    let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    let term = search.trim();
    if !term.is_empty() {
        sql.push_str(
            " AND (c.title LIKE ?1 OR c.vendor_code LIKE ?1 OR k.barcode LIKE ?1
                   OR CAST(c.nm_id AS TEXT) LIKE ?1 OR c.brand LIKE ?1)",
        );
        params_vec.push(Box::new(format!("%{term}%")));
    }
    if !categories.is_empty() {
        let start = params_vec.len();
        let ph: Vec<String> = (0..categories.len())
            .map(|i| format!("?{}", start + i + 1))
            .collect();
        sql.push_str(&format!(" AND c.subject_name IN ({})", ph.join(",")));
        for c in categories {
            params_vec.push(Box::new(c.clone()));
        }
    }
    // Rows of the same vendor code (one product, many sizes) stay together;
    // numeric sizes sort numerically.
    sql.push_str(&format!(
        " ORDER BY c.vendor_code, c.nm_id, CAST(cs.tech_size AS INTEGER), cs.tech_size LIMIT {limit}"
    ));
    let mut stmt = conn.prepare(&sql)?;
    let refs: Vec<&dyn rusqlite::types::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
    let rows = stmt
        .query_map(refs.as_slice(), |r| {
            Ok(SkuItem {
                nm_id: r.get(0)?,
                barcode: r.get(1)?,
                tech_size: r.get(2)?,
                wb_size: r.get(3)?,
                title: r.get(4)?,
                brand: r.get(5)?,
                subject_name: r.get(6)?,
                gender: r.get(7)?,
                color: r.get(8)?,
                vendor_code: r.get(9)?,
                photo: r.get(10)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

/// Shop categories with the genders present in the catalog, for KIZ mapping.
pub fn list_category_genders(conn: &Connection) -> AppResult<Vec<(String, String, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT subject_name, COALESCE(gender,''), COUNT(*)
         FROM cards
         WHERE subject_name IS NOT NULL AND subject_name <> ''
         GROUP BY subject_name, COALESCE(gender,'')
         ORDER BY subject_name, gender",
    )?;
    let rows = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

// --- supplies ---------------------------------------------------------------

pub fn upsert_supply(conn: &Connection, s: &WbSupply) -> AppResult<()> {
    conn.execute(
        "INSERT INTO supplies(id, name, done, cargo_type, pickup, created_at, closed_at, scan_dt, raw_json, synced_at)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
         ON CONFLICT(id) DO UPDATE SET
           name=excluded.name, done=excluded.done, cargo_type=excluded.cargo_type,
           pickup=excluded.pickup, created_at=excluded.created_at, closed_at=excluded.closed_at,
           scan_dt=excluded.scan_dt, synced_at=excluded.synced_at",
        params![
            s.id,
            s.name,
            s.done as i64,
            s.cargo_type,
            s.is_pickup as i64,
            s.created_at,
            s.closed_at,
            s.scan_dt,
            "{}",
            now_iso()
        ],
    )?;
    Ok(())
}

pub fn delete_supply(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM supplies WHERE id=?1", params![id])?;
    Ok(())
}

pub fn insert_local_supply(conn: &Connection, id: &str, name: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO supplies(id, name, done, cargo_type, created_at, closed_at, scan_dt, raw_json, synced_at)
         VALUES(?1,?2,0,0,?3,NULL,NULL,'{}',?3)
         ON CONFLICT(id) DO NOTHING",
        params![id, name, now_iso()],
    )?;
    Ok(())
}

pub fn list_supplies(conn: &Connection) -> AppResult<Vec<SupplyRow>> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.name, s.done, s.cargo_type, COALESCE(s.pickup,0), s.created_at, s.closed_at,
                (SELECT COUNT(*) FROM orders o WHERE o.supply_id = s.id) AS order_count
         FROM supplies s
         ORDER BY s.created_at DESC",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(SupplyRow {
                id: r.get(0)?,
                name: r.get(1)?,
                done: r.get::<_, i64>(2)? != 0,
                cargo_type: r.get(3)?,
                is_pickup: r.get::<_, i64>(4)? != 0,
                created_at: r.get(5)?,
                closed_at: r.get(6)?,
                order_count: r.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}
