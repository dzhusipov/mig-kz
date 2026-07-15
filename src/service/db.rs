use crate::models::currency::Currency;
use chrono::{Local, NaiveDateTime};
use rusqlite::{params, Connection, Result};

pub fn save_currencies(conn: &mut Connection, currencies: &[Currency]) -> Result<()> {
    ensure_schema(conn)?;

    let now = Local::now().naive_local();
    let tx = conn.transaction()?;

    for cur in currencies {
        tx.execute(
            "INSERT INTO mig_currency (mig_currency, mig_currency_buy, mig_currency_sell, mig_created)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                cur.currency,
                cur.buy,
                cur.sell,
                now.to_string(),
            ],
        )?;
    }

    tx.commit()?;
    Ok(())
}

pub fn get_last_update(conn: &Connection) -> Result<Option<NaiveDateTime>> {
    ensure_schema(conn)?;
    let mut stmt = conn.prepare("SELECT MAX(mig_created) FROM mig_currency")?;
    let row = stmt.query_row([], |row| row.get::<_, String>(0));
    match row {
        Ok(ts) => {
            let dt = NaiveDateTime::parse_from_str(&ts, "%Y-%m-%d %H:%M:%S").ok();
            Ok(dt)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

fn ensure_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS mig_currency (
            mig_id    INTEGER PRIMARY KEY AUTOINCREMENT,
            mig_currency  VARCHAR NOT NULL,
            mig_currency_buy  REAL NOT NULL,
            mig_currency_sell REAL NOT NULL,
            mig_created   TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS telegram_chats (
            id      INTEGER PRIMARY KEY AUTOINCREMENT,
            chat_id INTEGER,
            chat_name TEXT
        );
        "#,
    )?;
    Ok(())
}
