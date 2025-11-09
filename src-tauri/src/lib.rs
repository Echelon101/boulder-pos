use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, path::PathBuf, sync::Mutex};
use tauri::{path::BaseDirectory, Manager, State};

struct DatabasePath(PathBuf);

impl DatabasePath {
    fn connect(&self) -> rusqlite::Result<Connection> {
        let conn = Connection::open(&self.0)?;
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
        Ok(conn)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductRecord {
    id: i64,
    name: String,
    price_cents: i64,
    accent: Option<String>,
    icon: Option<String>,
    note: Option<String>,
    product_type_id: Option<i64>,
    product_type_name: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProductPayload {
    id: Option<i64>,
    name: String,
    price_cents: i64,
    accent: Option<String>,
    icon: Option<String>,
    note: Option<String>,
    product_type_id: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TransactionRecord {
    id: i64,
    product_id: Option<i64>,
    quantity: i64,
    total_cents: i64,
    description: Option<String>,
    created_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransactionPayload {
    product_id: Option<i64>,
    quantity: i64,
    total_cents: i64,
    description: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductTypeRecord {
    id: i64,
    name: String,
    color: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProductTypePayload {
    id: Option<i64>,
    name: String,
    color: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MemberRecord {
    id: i64,
    first_name: String,
    last_name: String,
    email: Option<String>,
    phone: Option<String>,
    status: String,
    notes: Option<String>,
    balance_cents: i64,
    created_at: String,
    updated_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MemberPayload {
    id: Option<i64>,
    first_name: String,
    last_name: String,
    email: Option<String>,
    phone: Option<String>,
    status: Option<String>,
    notes: Option<String>,
    balance_cents: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MembershipRecord {
    id: i64,
    name: String,
    description: Option<String>,
    price_cents: Option<i64>,
    duration_days: Option<i64>,
    max_uses: Option<i64>,
    created_at: String,
    updated_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MembershipPayload {
    id: Option<i64>,
    name: String,
    description: Option<String>,
    price_cents: Option<i64>,
    duration_days: Option<i64>,
    max_uses: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckinRecord {
    id: i64,
    member_id: i64,
    member_name: String,
    membership_name: Option<String>,
    created_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckinPayload {
    member_id: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MemberMembershipRecord {
    id: i64,
    member_id: i64,
    membership_id: i64,
    membership_name: String,
    remaining_uses: Option<i64>,
    start_date: Option<String>,
    end_date: Option<String>,
    created_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssignMemberMembershipPayload {
    member_id: i64,
    membership_id: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RoleRecord {
    id: i64,
    name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UserRecord {
    id: i64,
    username: String,
    display_name: String,
    role: String,
    active: bool,
    created_at: String,
    updated_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserPayload {
    id: Option<i64>,
    username: String,
    display_name: String,
    password: Option<String>,
    role_id: i64,
    active: bool,
}

#[derive(Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginResponse {
    id: i64,
    username: String,
    display_name: String,
    role: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BucketRecord {
    id: i64,
    name: String,
    status: String,
    created_at: String,
    updated_at: String,
    item_count: i64,
    total_cents: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BucketItemRecord {
    id: i64,
    bucket_id: i64,
    product_id: i64,
    product_name: String,
    quantity: i64,
    price_cents: i64,
    line_total_cents: i64,
    accent: Option<String>,
    icon: Option<String>,
    note: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateBucketPayload {
    name: Option<String>,
}

#[derive(Deserialize)]
struct RenameBucketPayload {
    id: i64,
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BucketItemsRequest {
    bucket_id: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddBucketItemPayload {
    bucket_id: i64,
    product_id: i64,
    quantity: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BucketIdPayload {
    bucket_id: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckoutBucketPayload {
    bucket_id: i64,
    member_id: Option<i64>,
    use_balance: bool,
    payment_method: Option<String>,
}

struct DefaultProduct {
    name: &'static str,
    price_cents: i64,
    accent: Option<&'static str>,
    icon: Option<&'static str>,
    note: Option<&'static str>,
}

const DEFAULT_PRODUCTS: &[DefaultProduct] = &[
    DefaultProduct {
        name: "Eistee",
        price_cents: 380,
        accent: Some("#83ff4e"),
        icon: Some("🥤"),
        note: None,
    },
    DefaultProduct {
        name: "Saftschorle",
        price_cents: 350,
        accent: Some("#84ff4f"),
        icon: Some("🧃"),
        note: None,
    },
    DefaultProduct {
        name: "Fruchtsaft",
        price_cents: 390,
        accent: Some("#9bff57"),
        icon: Some("🍹"),
        note: None,
    },
    DefaultProduct {
        name: "Hauslimonade",
        price_cents: 350,
        accent: Some("#9dff5d"),
        icon: Some("🍋"),
        note: None,
    },
    DefaultProduct {
        name: "Mineralwasser",
        price_cents: 200,
        accent: Some("#acf5ff"),
        icon: Some("🚰"),
        note: None,
    },
    DefaultProduct {
        name: "Apfelsaft",
        price_cents: 340,
        accent: Some("#6ded47"),
        icon: Some("🍏"),
        note: None,
    },
    DefaultProduct {
        name: "Flat White",
        price_cents: 360,
        accent: Some("#33a55d"),
        icon: Some("☕"),
        note: None,
    },
    DefaultProduct {
        name: "Cappuccino",
        price_cents: 320,
        accent: Some("#2f9755"),
        icon: Some("🥛"),
        note: None,
    },
    DefaultProduct {
        name: "Latte Macchiato",
        price_cents: 380,
        accent: Some("#2d8b50"),
        icon: Some("🧋"),
        note: None,
    },
    DefaultProduct {
        name: "Affogato",
        price_cents: 420,
        accent: Some("#297d47"),
        icon: Some("🍨"),
        note: None,
    },
    DefaultProduct {
        name: "Brezel",
        price_cents: 250,
        accent: Some("#f73ccb"),
        icon: Some("🥨"),
        note: None,
    },
    DefaultProduct {
        name: "Käsekuchen",
        price_cents: 340,
        accent: Some("#ff46d3"),
        icon: Some("🍰"),
        note: None,
    },
    DefaultProduct {
        name: "Brownie",
        price_cents: 290,
        accent: Some("#ff40c8"),
        icon: Some("🧁"),
        note: None,
    },
    DefaultProduct {
        name: "Schokoriegel",
        price_cents: 150,
        accent: Some("#d81b60"),
        icon: Some("🍫"),
        note: None,
    },
    DefaultProduct {
        name: "Nachos",
        price_cents: 410,
        accent: Some("#e53935"),
        icon: Some("🌮"),
        note: None,
    },
    DefaultProduct {
        name: "Pizzaschnecke",
        price_cents: 380,
        accent: Some("#c62828"),
        icon: Some("🍕"),
        note: None,
    },
    DefaultProduct {
        name: "Focaccia",
        price_cents: 390,
        accent: Some("#b71c1c"),
        icon: Some("🥙"),
        note: None,
    },
    DefaultProduct {
        name: "Tagesangebot",
        price_cents: 550,
        accent: Some("#f44336"),
        icon: Some("⭐"),
        note: Some("Chefwahl"),
    },
];

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn list_products(db: State<DatabasePath>) -> Result<Vec<ProductRecord>, String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT p.id,
                p.name,
                p.price_cents,
                p.accent,
                p.icon,
                p.note,
                p.product_type_id,
                pt.name as product_type_name
            FROM products p
            LEFT JOIN product_types pt ON pt.id = p.product_type_id
            ORDER BY p.name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ProductRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                price_cents: row.get(2)?,
                accent: row.get(3)?,
                icon: row.get(4)?,
                note: row.get(5)?,
                product_type_id: row.get(6)?,
                product_type_name: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_product(db: State<DatabasePath>, payload: ProductPayload) -> Result<i64, String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    if let Some(id) = payload.id {
        conn.execute(
            "UPDATE products SET name = ?, price_cents = ?, accent = ?, icon = ?, note = ?, product_type_id = ? WHERE id = ?",
            params![
                payload.name,
                payload.price_cents,
                payload.accent,
                payload.icon,
                payload.note,
                payload.product_type_id,
                id
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(id)
    } else {
        conn.execute(
            "INSERT INTO products (name, price_cents, accent, icon, note, product_type_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                payload.name,
                payload.price_cents,
                payload.accent,
                payload.icon,
                payload.note,
                payload.product_type_id
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }
}

#[tauri::command]
fn delete_product(db: State<DatabasePath>, id: i64) -> Result<(), String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM products WHERE id = ?", [id])
        .map_err(|e| e.to_string())
        .map(|_| ())
}

#[tauri::command]
fn record_transaction(db: State<DatabasePath>, payload: TransactionPayload) -> Result<(), String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO transactions (product_id, quantity, total_cents, description) \
        VALUES (?1, ?2, ?3, ?4)",
        params![
            payload.product_id,
            payload.quantity.max(1),
            payload.total_cents,
            payload.description
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn list_transactions(db: State<DatabasePath>) -> Result<Vec<TransactionRecord>, String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, product_id, quantity, total_cents, description, created_at \
            FROM transactions ORDER BY id DESC LIMIT 50",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(TransactionRecord {
                id: row.get(0)?,
                product_id: row.get(1)?,
                quantity: row.get(2)?,
                total_cents: row.get(3)?,
                description: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_buckets(db: State<DatabasePath>) -> Result<Vec<BucketRecord>, String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "
        SELECT
            b.id,
            b.name,
            b.status,
            b.created_at,
            b.updated_at,
            COALESCE(SUM(bi.quantity), 0) AS item_count,
            COALESCE(SUM(bi.quantity * bi.price_cents), 0) AS total_cents
        FROM buckets b
        LEFT JOIN bucket_items bi ON bi.bucket_id = b.id
        WHERE b.status = 'open'
        GROUP BY b.id
        ORDER BY b.created_at ASC
        ",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(BucketRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                status: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                item_count: row.get(5)?,
                total_cents: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn create_bucket(
    db: State<DatabasePath>,
    _payload: Option<CreateBucketPayload>,
) -> Result<i64, String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    let name = generate_bucket_name(&conn).map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO buckets (name) VALUES (?1)", params![name])
        .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
fn rename_bucket(db: State<DatabasePath>, payload: RenameBucketPayload) -> Result<(), String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE buckets SET name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        params![payload.name.trim(), payload.id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_bucket_items(
    db: State<DatabasePath>,
    payload: BucketItemsRequest,
) -> Result<Vec<BucketItemRecord>, String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "
        SELECT
            bi.id,
            bi.bucket_id,
            bi.product_id,
            bi.product_name,
            bi.quantity,
            bi.price_cents,
            (bi.quantity * bi.price_cents) AS line_total_cents,
            p.accent,
            p.icon,
            p.note
        FROM bucket_items bi
        LEFT JOIN products p ON p.id = bi.product_id
        WHERE bi.bucket_id = ?
        ORDER BY bi.id ASC
        ",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([payload.bucket_id], |row| {
            Ok(BucketItemRecord {
                id: row.get(0)?,
                bucket_id: row.get(1)?,
                product_id: row.get(2)?,
                product_name: row.get(3)?,
                quantity: row.get(4)?,
                price_cents: row.get(5)?,
                line_total_cents: row.get(6)?,
                accent: row.get(7)?,
                icon: row.get(8)?,
                note: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn add_product_to_bucket(
    db: State<DatabasePath>,
    payload: AddBucketItemPayload,
) -> Result<(), String> {
    let conn = db.connect().map_err(|e| e.to_string())?;

    let product: Option<(String, i64)> = conn
        .query_row(
            "SELECT name, price_cents FROM products WHERE id = ?",
            [payload.product_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let (product_name, price_cents) =
        product.ok_or_else(|| "Produkt nicht gefunden".to_string())?;
    let quantity = payload.quantity.unwrap_or(1).max(1);

    conn.execute(
        "
        INSERT INTO bucket_items (bucket_id, product_id, product_name, quantity, price_cents)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(bucket_id, product_id) DO UPDATE SET
            quantity = bucket_items.quantity + excluded.quantity,
            price_cents = excluded.price_cents,
            product_name = excluded.product_name
        ",
        params![
            payload.bucket_id,
            payload.product_id,
            product_name,
            quantity,
            price_cents
        ],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE buckets SET updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        [payload.bucket_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn close_bucket(db: State<DatabasePath>, payload: BucketIdPayload) -> Result<(), String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    let updated = conn
        .execute(
            "UPDATE buckets SET status = 'closed', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            [payload.bucket_id],
        )
        .map_err(|e| e.to_string())?;
    if updated == 0 {
        return Err("Bucket nicht gefunden".into());
    }
    conn.execute("DELETE FROM bucket_items WHERE bucket_id = ?", [payload.bucket_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete_bucket(db: State<DatabasePath>, payload: BucketIdPayload) -> Result<(), String> {
    let mut conn = db.connect().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM bucket_items WHERE bucket_id = ?", [payload.bucket_id])
        .map_err(|e| e.to_string())?;
    let removed = tx
        .execute("DELETE FROM buckets WHERE id = ?", [payload.bucket_id])
        .map_err(|e| e.to_string())?;
    if removed == 0 {
        return Err("Bucket nicht gefunden".into());
    }
    tx.commit().map_err(|e| e.to_string())
}

#[tauri::command]
fn checkout_bucket(
    db: State<DatabasePath>,
    payload: CheckoutBucketPayload,
) -> Result<(), String> {
    let mut conn = db.connect().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let bucket: Option<(String, String)> = tx
        .query_row(
            "SELECT name, status FROM buckets WHERE id = ?",
            [payload.bucket_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let (bucket_name, status) =
        bucket.ok_or_else(|| "Bucket nicht gefunden".to_string())?;
    if status.as_str() != "open" {
        return Err("Bucket ist nicht mehr offen.".into());
    }

    let total_cents: i64 = tx
        .query_row(
            "SELECT COALESCE(SUM(quantity * price_cents), 0) FROM bucket_items WHERE bucket_id = ?",
            [payload.bucket_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if total_cents <= 0 {
        return Err("Bucket ist leer.".into());
    }

    if payload.use_balance {
        let member_id = payload
            .member_id
            .ok_or_else(|| "Mitglied auswählen, um Guthaben zu verwenden.".to_string())?;
        tx.execute(
            "UPDATE members SET balance_cents = balance_cents - ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![total_cents, member_id],
        )
        .map_err(|e| e.to_string())?;
    }

    let method = if payload.use_balance {
        "Guthaben".to_string()
    } else {
        payload
            .payment_method
            .unwrap_or_else(|| "Bar".to_string())
    };

    tx.execute(
        "INSERT INTO transactions (product_id, quantity, total_cents, description) VALUES (NULL, 1, ?, ?)",
        params![
            total_cents,
            format!("{} bezahlt ({})", bucket_name, method)
        ],
    )
    .map_err(|e| e.to_string())?;

    tx.execute("DELETE FROM bucket_items WHERE bucket_id = ?", [payload.bucket_id])
        .map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE buckets SET status = 'closed', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        [payload.bucket_id],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())
}

#[tauri::command]
fn list_product_types(db: State<DatabasePath>) -> Result<Vec<ProductTypeRecord>, String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "
        SELECT id, name, color, created_at, updated_at
        FROM product_types
        ORDER BY name COLLATE NOCASE
        ",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ProductTypeRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_product_type(db: State<DatabasePath>, payload: ProductTypePayload) -> Result<i64, String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    if let Some(id) = payload.id {
        conn.execute(
            "UPDATE product_types SET name = ?, color = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![payload.name, payload.color, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(id)
    } else {
        conn.execute(
            "INSERT INTO product_types (name, color) VALUES (?1, ?2)",
            params![payload.name, payload.color],
        )
        .map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }
}

#[tauri::command]
fn delete_product_type(db: State<DatabasePath>, id: i64) -> Result<(), String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM product_types WHERE id = ?", [id])
        .map_err(|e| e.to_string())
        .map(|_| ())
}

#[tauri::command]
fn list_members(db: State<DatabasePath>) -> Result<Vec<MemberRecord>, String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "
        SELECT m.id,
               m.first_name,
               m.last_name,
               m.email,
               m.phone,
               m.status,
               m.notes,
               m.balance_cents,
               m.created_at,
               m.updated_at
        FROM members m
        ORDER BY m.last_name COLLATE NOCASE, m.first_name COLLATE NOCASE
        ",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(MemberRecord {
                id: row.get(0)?,
                first_name: row.get(1)?,
                last_name: row.get(2)?,
                email: row.get(3)?,
                phone: row.get(4)?,
                status: row.get(5)?,
                notes: row.get(6)?,
                balance_cents: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_member(db: State<DatabasePath>, payload: MemberPayload) -> Result<i64, String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    if let Some(id) = payload.id {
        conn.execute(
            "UPDATE members
            SET first_name = ?,
                last_name = ?,
                email = ?,
                phone = ?,
                status = ?,
                notes = ?,
                balance_cents = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?",
            params![
                payload.first_name,
                payload.last_name,
                payload.email,
                payload.phone,
                payload.status.unwrap_or_else(|| "active".into()),
                payload.notes,
                payload.balance_cents.unwrap_or(0),
                id
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(id)
    } else {
        conn.execute(
            "INSERT INTO members (first_name, last_name, email, phone, status, notes, balance_cents)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                payload.first_name,
                payload.last_name,
                payload.email,
                payload.phone,
                payload.status.unwrap_or_else(|| "active".into()),
                payload.notes,
                payload.balance_cents.unwrap_or(0)
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }
}

#[tauri::command]
fn delete_member(db: State<DatabasePath>, id: i64) -> Result<(), String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM members WHERE id = ?", [id])
        .map_err(|e| e.to_string())
        .map(|_| ())
}

#[tauri::command]
fn list_memberships(db: State<DatabasePath>) -> Result<Vec<MembershipRecord>, String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "
        SELECT id,
               membership_type,
               notes,
               price_cents,
               duration_days,
               max_uses,
               created_at,
               updated_at
        FROM memberships
        ORDER BY created_at DESC
        ",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(MembershipRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                price_cents: row.get(3)?,
                duration_days: row.get(4)?,
                max_uses: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_membership(db: State<DatabasePath>, payload: MembershipPayload) -> Result<i64, String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    if let Some(id) = payload.id {
        conn.execute(
            "UPDATE memberships
            SET membership_type = ?, notes = ?, price_cents = ?, duration_days = ?, max_uses = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?",
            params![
                payload.name,
                payload.description,
                payload.price_cents,
                payload.duration_days,
                payload.max_uses,
                id
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(id)
    } else {
        conn.execute(
            "INSERT INTO memberships (membership_type, notes, price_cents, duration_days, max_uses)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                payload.name,
                payload.description,
                payload.price_cents,
                payload.duration_days,
                payload.max_uses
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }
}

#[tauri::command]
fn delete_membership(db: State<DatabasePath>, id: i64) -> Result<(), String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM memberships WHERE id = ?", [id])
        .map_err(|e| e.to_string())
        .map(|_| ())
}

#[tauri::command]
fn list_member_memberships(db: State<DatabasePath>) -> Result<Vec<MemberMembershipRecord>, String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "
        SELECT mm.id,
               mm.member_id,
               mm.membership_id,
               ms.membership_type,
               mm.remaining_uses,
               mm.start_date,
               mm.end_date,
               mm.created_at
        FROM member_memberships mm
        JOIN memberships ms ON ms.id = mm.membership_id
        ORDER BY mm.member_id, mm.created_at DESC
        ",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(MemberMembershipRecord {
                id: row.get(0)?,
                member_id: row.get(1)?,
                membership_id: row.get(2)?,
                membership_name: row.get(3)?,
                remaining_uses: row.get(4)?,
                start_date: row.get(5)?,
                end_date: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
fn assign_member_membership(
    db: State<DatabasePath>,
    payload: AssignMemberMembershipPayload,
) -> Result<i64, String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    let membership: (String, Option<i64>, Option<i64>) = conn
        .query_row(
            "SELECT membership_type, duration_days, max_uses FROM memberships WHERE id = ?",
            [payload.membership_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| "Mitgliedschaft nicht gefunden".to_string())?;

    let duration_days = membership.1;
    let max_uses = membership.2;
    let end_interval = duration_days.map(|days| format!("+{} days", days));

    conn.execute(
        "INSERT INTO member_memberships (member_id, membership_id, remaining_uses, start_date, end_date)
        VALUES (?1, ?2, ?3, DATE('now','localtime'), CASE WHEN ?4 IS NOT NULL THEN DATE('now','localtime', ?4) ELSE NULL END)",
        params![payload.member_id, payload.membership_id, max_uses, end_interval],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
fn delete_member_membership(db: State<DatabasePath>, id: i64) -> Result<(), String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM member_memberships WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete_transaction(db: State<DatabasePath>, id: i64) -> Result<(), String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM transactions WHERE id = ?", [id])
        .map_err(|e| e.to_string())
        .map(|_| ())
}

#[tauri::command]
fn record_checkin(db: State<DatabasePath>, payload: CheckinPayload) -> Result<(), String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    let existing = conn
        .query_row(
            "SELECT 1 FROM member_checkins WHERE member_id = ? AND DATE(created_at, 'localtime') = DATE('now', 'localtime')",
            [payload.member_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if existing.is_some() {
        return Err("Mitglied ist heute bereits eingecheckt".into());
    }

    let candidate = conn
        .query_row(
            "SELECT mm.id, mm.membership_id, mm.remaining_uses
            FROM member_memberships mm
            JOIN memberships ms ON ms.id = mm.membership_id
            WHERE mm.member_id = ?
              AND (mm.remaining_uses IS NULL OR mm.remaining_uses > 0)
              AND (mm.end_date IS NULL OR DATE(mm.end_date) >= DATE('now','localtime'))
            ORDER BY (mm.end_date IS NULL) ASC, mm.end_date ASC, mm.created_at ASC
            LIMIT 1",
            [payload.member_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, Option<i64>>(2)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let (member_membership_id, membership_id, remaining_uses) = candidate
        .ok_or_else(|| "Keine aktive Mitgliedschaft gefunden".to_string())?;

    conn.execute(
        "INSERT INTO member_checkins (member_id, membership_id, member_membership_id) VALUES (?1, ?2, ?3)",
        params![payload.member_id, membership_id, member_membership_id],
    )
    .map_err(|e| e.to_string())?;

    if let Some(uses) = remaining_uses {
        let new_uses = uses.saturating_sub(1);
        conn.execute(
            "UPDATE member_memberships SET remaining_uses = ? WHERE id = ?",
            params![new_uses, member_membership_id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
fn delete_checkin(db: State<DatabasePath>, id: i64) -> Result<(), String> {
    let mut conn = db.connect().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let details = tx
        .query_row(
            "SELECT member_membership_id, membership_id FROM member_checkins WHERE id = ?",
            [id],
            |row| Ok((row.get::<_, Option<i64>>(0)?, row.get::<_, Option<i64>>(1)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let (member_membership_id, _membership_id) =
        details.ok_or_else(|| "Check-in nicht gefunden".to_string())?;

    if let Some(mm_id) = member_membership_id {
        if let Some((remaining_uses, max_uses)) = tx
            .query_row(
                "SELECT remaining_uses, ms.max_uses
                FROM member_memberships mm
                JOIN memberships ms ON ms.id = mm.membership_id
                WHERE mm.id = ?",
                [mm_id],
                |row| Ok((row.get::<_, Option<i64>>(0)?, row.get::<_, Option<i64>>(1)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?
        {
            if let Some(current) = remaining_uses {
                let cap = max_uses.unwrap_or(i64::MAX);
                let new_value = (current + 1).min(cap);
                tx.execute(
                    "UPDATE member_memberships SET remaining_uses = ? WHERE id = ?",
                    params![new_value, mm_id],
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }

    tx.execute("DELETE FROM member_checkins WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

#[tauri::command]
fn list_checkins_today(db: State<DatabasePath>) -> Result<Vec<CheckinRecord>, String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "
        SELECT c.id,
               c.member_id,
               m.first_name || ' ' || m.last_name AS member_name,
               ms.membership_type,
               c.created_at
        FROM member_checkins c
        JOIN members m ON m.id = c.member_id
        LEFT JOIN memberships ms ON ms.id = c.membership_id
        WHERE DATE(c.created_at, 'localtime') = DATE('now', 'localtime')
        ORDER BY c.created_at DESC
        ",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(CheckinRecord {
                id: row.get(0)?,
                member_id: row.get(1)?,
                member_name: row.get(2)?,
                membership_name: row.get(3).ok(),
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_transactions_today(db: State<DatabasePath>) -> Result<Vec<TransactionRecord>, String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, product_id, quantity, total_cents, description, created_at \
            FROM transactions \
            WHERE DATE(created_at, 'localtime') = DATE('now', 'localtime') \
            ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(TransactionRecord {
                id: row.get(0)?,
                product_id: row.get(1)?,
                quantity: row.get(2)?,
                total_cents: row.get(3)?,
                description: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_roles(db: State<DatabasePath>) -> Result<Vec<RoleRecord>, String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name FROM user_roles ORDER BY id ASC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(RoleRecord {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_users(db: State<DatabasePath>) -> Result<Vec<UserRecord>, String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT u.id, u.username, u.display_name, r.name, u.active, u.created_at, u.updated_at \
            FROM users u \
            JOIN user_roles r ON r.id = u.role_id \
            ORDER BY u.username COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(UserRecord {
                id: row.get(0)?,
                username: row.get(1)?,
                display_name: row.get(2)?,
                role: row.get(3)?,
                active: row.get::<_, i64>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_user(db: State<DatabasePath>, payload: UserPayload) -> Result<i64, String> {
    let conn = db.connect().map_err(|e| e.to_string())?;

    if let Some(id) = payload.id {
        if let Some(password) = payload.password.as_ref() {
            let hashed = hash_password(password)?;
            conn.execute(
                "UPDATE users SET username = ?, display_name = ?, password_hash = ?, role_id = ?, active = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                params![
                    payload.username,
                    payload.display_name,
                    hashed,
                    payload.role_id,
                    if payload.active { 1 } else { 0 },
                    id
                ],
            )
            .map_err(|e| e.to_string())?;
        } else {
            conn.execute(
                "UPDATE users SET username = ?, display_name = ?, role_id = ?, active = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                params![
                    payload.username,
                    payload.display_name,
                    payload.role_id,
                    if payload.active { 1 } else { 0 },
                    id
                ],
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(id)
    } else {
        let password = payload
            .password
            .as_ref()
            .ok_or_else(|| "Passwort erforderlich".to_string())?;
        let hashed = hash_password(password)?;
        conn.execute(
            "INSERT INTO users (username, display_name, password_hash, role_id, active) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                payload.username,
                payload.display_name,
                hashed,
                payload.role_id,
                if payload.active { 1 } else { 0 }
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }
}

#[tauri::command]
fn delete_user(db: State<DatabasePath>, id: i64) -> Result<(), String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM users WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn login_user(db: State<DatabasePath>, payload: LoginPayload) -> Result<LoginResponse, String> {
    let conn = db.connect().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT u.id, u.username, u.display_name, u.password_hash, r.name, u.active \
            FROM users u JOIN user_roles r ON r.id = u.role_id WHERE u.username = ?",
        )
        .map_err(|e| e.to_string())?;

    let user = stmt
        .query_row([&payload.username], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })
        .optional()
        .map_err(|e| e.to_string())?;

    let (id, username, display_name, hash, role, active_flag) =
        user.ok_or_else(|| "Benutzer nicht gefunden".to_string())?;

    if active_flag == 0 {
        return Err("Benutzer ist deaktiviert".into());
    }

    if !verify_password(&hash, &payload.password)? {
        return Err("Passwort ungültig".into());
    }

    Ok(LoginResponse {
        id,
        username,
        display_name,
        role,
    })
}

#[tauri::command]
fn get_settings(state: State<SettingsState>) -> Result<AppSettings, String> {
    state.get()
}

#[tauri::command]
fn update_settings(
    state: State<SettingsState>,
    payload: AppSettings,
) -> Result<AppSettings, String> {
    state.save(payload.clone())?;
    Ok(payload)
}

fn initialize_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS user_roles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        );
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            display_name TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            role_id INTEGER NOT NULL,
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(role_id) REFERENCES user_roles(id)
        );
        CREATE TABLE IF NOT EXISTS product_types (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            color TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS products (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            price_cents INTEGER NOT NULL,
            accent TEXT,
            icon TEXT,
            note TEXT,
            product_type_id INTEGER,
            FOREIGN KEY(product_type_id) REFERENCES product_types(id)
        );
        CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            product_id INTEGER,
            quantity INTEGER NOT NULL DEFAULT 1,
            total_cents INTEGER NOT NULL,
            description TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(product_id) REFERENCES products(id)
        );
        CREATE TABLE IF NOT EXISTS members (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULL,
            email TEXT,
            phone TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            notes TEXT,
            balance_cents INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS memberships (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            membership_type TEXT NOT NULL,
            price_cents INTEGER,
            notes TEXT,
            duration_days INTEGER,
            max_uses INTEGER,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS member_memberships (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            member_id INTEGER NOT NULL,
            membership_id INTEGER NOT NULL,
            remaining_uses INTEGER,
            start_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            end_date TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(member_id) REFERENCES members(id) ON DELETE CASCADE,
            FOREIGN KEY(membership_id) REFERENCES memberships(id)
        );
        CREATE TABLE IF NOT EXISTS buckets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'open',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS bucket_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            bucket_id INTEGER NOT NULL,
            product_id INTEGER NOT NULL,
            product_name TEXT NOT NULL,
            quantity INTEGER NOT NULL DEFAULT 1,
            price_cents INTEGER NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(bucket_id) REFERENCES buckets(id) ON DELETE CASCADE,
            FOREIGN KEY(product_id) REFERENCES products(id),
            UNIQUE(bucket_id, product_id)
        );
        CREATE TABLE IF NOT EXISTS member_checkins (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            member_id INTEGER NOT NULL,
            membership_id INTEGER,
            member_membership_id INTEGER,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(member_id) REFERENCES members(id) ON DELETE CASCADE,
            FOREIGN KEY(membership_id) REFERENCES memberships(id),
            FOREIGN KEY(member_membership_id) REFERENCES member_memberships(id) ON DELETE SET NULL
        );
        CREATE INDEX IF NOT EXISTS idx_bucket_items_bucket ON bucket_items(bucket_id);
        ",
    )
}

fn ensure_product_type_column(conn: &Connection) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(products)")?;
    let mut has_column = false;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == "product_type_id" {
            has_column = true;
            break;
        }
    }

    if !has_column {
        let _ = conn.execute(
            "ALTER TABLE products ADD COLUMN product_type_id INTEGER",
            [],
        );
    }
    Ok(())
}

fn load_settings_from_disk(path: &PathBuf, default: AppSettings) -> AppSettings {
    match fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or(default),
        Err(_) => default,
    }
}

fn ensure_member_columns(conn: &Connection) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(members)")?;
    let mut has_balance = false;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == "balance_cents" {
            has_balance = true;
        }
    }

    if !has_balance {
        conn.execute(
            "ALTER TABLE members ADD COLUMN balance_cents INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }
    Ok(())
}

fn ensure_membership_columns(conn: &Connection) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(memberships)")?;
    let mut has_duration = false;
    let mut has_max = false;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == "duration_days" {
            has_duration = true;
        } else if name == "max_uses" {
            has_max = true;
        }
    }

    if !has_duration {
        conn.execute(
            "ALTER TABLE memberships ADD COLUMN duration_days INTEGER",
            [],
        )?;
    }
    if !has_max {
        conn.execute("ALTER TABLE memberships ADD COLUMN max_uses INTEGER", [])?;
    }
    Ok(())
}

fn ensure_checkin_columns(conn: &Connection) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(member_checkins)")?;
    let mut has_member_membership_id = false;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == "member_membership_id" {
            has_member_membership_id = true;
        }
    }

    if !has_member_membership_id {
        conn.execute(
            "ALTER TABLE member_checkins ADD COLUMN member_membership_id INTEGER",
            [],
        )?;
    }
    Ok(())
}

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| e.to_string())
}

fn verify_password(hash: &str, password: &str) -> Result<bool, String> {
    let parsed = PasswordHash::new(hash).map_err(|e| e.to_string())?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

fn hash_password_for_seed(password: &str) -> rusqlite::Result<String> {
    hash_password(password).map_err(|e| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            e,
        )))
    })
}

fn seed_default_products(conn: &mut Connection) -> rusqlite::Result<()> {
    let existing: i64 = conn.query_row("SELECT COUNT(*) FROM products", [], |row| row.get(0))?;
    if existing > 0 {
        return Ok(());
    }

    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO products (name, price_cents, accent, icon, note) VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        for product in DEFAULT_PRODUCTS {
            stmt.execute(params![
                product.name,
                product.price_cents,
                product.accent,
                product.icon,
                product.note
            ])?;
        }
    }
    tx.commit()
}

fn seed_default_product_types(conn: &Connection) -> rusqlite::Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM product_types", [], |row| row.get(0))?;
    if count > 0 {
        return Ok(());
    }
    conn.execute_batch(
        "
        INSERT INTO product_types (name, color) VALUES
            ('Getränke', '#83ff4e'),
            ('Snacks', '#ff46d3'),
            ('Desserts', '#ff8af2');
        ",
    )
}

fn ensure_default_bucket(conn: &mut Connection) -> rusqlite::Result<()> {
    let existing = conn
        .query_row("SELECT id FROM buckets LIMIT 1", [], |row| {
            row.get::<_, i64>(0)
        })
        .optional()?;
    if existing.is_some() {
        return Ok(());
    }
    conn.execute("INSERT INTO buckets (name) VALUES ('Bucket 1')", [])?;
    Ok(())
}

fn generate_bucket_name(conn: &Connection) -> rusqlite::Result<String> {
    let mut stmt = conn.prepare("SELECT name FROM buckets WHERE status = 'open'")?;
    let mut used_numbers: HashSet<i64> = HashSet::new();
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(0)?;
        if let Some(number) = extract_bucket_number(&name) {
            used_numbers.insert(number);
        }
    }

    let mut candidate = 1;
    while used_numbers.contains(&candidate) {
        candidate += 1;
    }
    Ok(format!("Bucket {}", candidate))
}

fn extract_bucket_number(name: &str) -> Option<i64> {
    let digits: String = name.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

fn ensure_roles(conn: &mut Connection) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    for role in ["admin", "manager", "user"] {
        tx.execute(
            "INSERT OR IGNORE INTO user_roles (name) VALUES (?1)",
            [role],
        )?;
    }
    tx.commit()
}

fn ensure_admin_user(conn: &Connection) -> rusqlite::Result<()> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT id FROM users WHERE username = 'admin' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()?;
    if exists.is_some() {
        return Ok(());
    }

    let role_id: i64 = conn.query_row(
        "SELECT id FROM user_roles WHERE name = 'admin'",
        [],
        |row| row.get(0),
    )?;

    let hashed = hash_password_for_seed("admin")?;
    conn.execute(
        "INSERT INTO users (username, display_name, password_hash, role_id, active) VALUES ('admin', 'Administrator', ?1, ?2, 1)",
        params![hashed, role_id],
    )?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir()?;
            fs::create_dir_all(&app_dir)?;
            let db_path = app_dir.join("boulderado-pos.db");

            let mut conn = Connection::open(&db_path)?;
            initialize_db(&conn)?;
            ensure_product_type_column(&conn)?;
            ensure_member_columns(&conn)?;
            ensure_membership_columns(&conn)?;
            ensure_checkin_columns(&conn)?;
            ensure_roles(&mut conn)?;
            ensure_admin_user(&conn)?;
            seed_default_product_types(&conn)?;
            seed_default_products(&mut conn)?;
            ensure_default_bucket(&mut conn)?;
            drop(conn);

            let exe_dir = app
                .path()
                .resolve(".", BaseDirectory::Executable)
                .map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("could not resolve executable directory: {e}"),
                    )
                })?;
            let conf_dir = exe_dir.join("conf");
            fs::create_dir_all(&conf_dir)?;
            let settings_path = conf_dir.join("settings.json");
            let default_settings = AppSettings::with_defaults(&db_path);
            let initial_settings = load_settings_from_disk(&settings_path, default_settings);

            app.manage(DatabasePath(db_path));
            app.manage(SettingsState::new(settings_path, initial_settings));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            list_products,
            save_product,
            delete_product,
            record_transaction,
            list_transactions,
            delete_transaction,
            record_checkin,
            delete_checkin,
            list_checkins_today,
            list_transactions_today,
            list_buckets,
            create_bucket,
            rename_bucket,
            get_bucket_items,
            add_product_to_bucket,
            close_bucket,
            delete_bucket,
            checkout_bucket,
            list_product_types,
            save_product_type,
            delete_product_type,
            list_members,
            save_member,
            delete_member,
            list_memberships,
            save_membership,
            delete_membership,
            list_member_memberships,
            assign_member_membership,
            delete_member_membership,
            list_users,
            save_user,
            delete_user,
            list_roles,
            login_user,
            get_settings,
            update_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    db_location: String,
    language: String,
    currency: String,
    auto_updates: bool,
    enable_backups: bool,
}

impl AppSettings {
    fn with_defaults(db_path: &PathBuf) -> Self {
        Self {
            db_location: db_path.to_string_lossy().to_string(),
            language: "de".into(),
            currency: "EUR".into(),
            auto_updates: true,
            enable_backups: false,
        }
    }
}

struct SettingsState {
    path: PathBuf,
    current: Mutex<AppSettings>,
}

impl SettingsState {
    fn new(path: PathBuf, settings: AppSettings) -> Self {
        Self {
            path,
            current: Mutex::new(settings),
        }
    }

    fn get(&self) -> Result<AppSettings, String> {
        let guard = self
            .current
            .lock()
            .map_err(|_| "Einstellungen konnten nicht gelesen werden".to_string())?;
        Ok(guard.clone())
    }

    fn save(&self, updated: AppSettings) -> Result<(), String> {
        let serialized = serde_json::to_string_pretty(&updated).map_err(|e| e.to_string())?;
        fs::write(&self.path, serialized).map_err(|e| e.to_string())?;

        let mut guard = self
            .current
            .lock()
            .map_err(|_| "Einstellungen konnten nicht gespeichert werden".to_string())?;
        *guard = updated;
        Ok(())
    }
}
