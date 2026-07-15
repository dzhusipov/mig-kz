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
    let row = stmt.query_row([], |row| row.get::<_, Option<String>>(0));
    match row {
        Ok(Some(ts)) => {
            let dt = NaiveDateTime::parse_from_str(&ts, "%Y-%m-%d %H:%M:%S%.6f").ok();
            Ok(dt)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Ok(None) => Ok(None),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::currency::Currency;

    fn make_conn() -> Connection {
        Connection::open_in_memory().unwrap()
    }

    #[test]
    fn test_ensure_schema_creates_tables() {
        let conn = make_conn();
        ensure_schema(&conn).unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='mig_currency'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_ensure_schema_idempotent() {
        let conn = make_conn();
        ensure_schema(&conn).unwrap();
        ensure_schema(&conn).unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='mig_currency'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_save_currencies_inserts_rows() {
        let mut conn = make_conn();
        let currencies = vec![
            Currency { currency: "USD".to_string(), buy: 469.9, sell: 472.9 },
            Currency { currency: "EUR".to_string(), buy: 536.5, sell: 541.5 },
        ];
        save_currencies(&mut conn, &currencies).unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM mig_currency",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_save_currencies_preserves_values() {
        let mut conn = make_conn();
        let currencies = vec![
            Currency { currency: "USD".to_string(), buy: 469.9, sell: 472.9 },
        ];
        save_currencies(&mut conn, &currencies).unwrap();

        let (currency, buy, sell): (String, f64, f64) = conn.query_row(
            "SELECT mig_currency, mig_currency_buy, mig_currency_sell FROM mig_currency LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).unwrap();

        assert_eq!(currency, "USD");
        assert!((buy - 469.9).abs() < f64::EPSILON);
        assert!((sell - 472.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_save_currencies_empty_list() {
        let mut conn = make_conn();
        save_currencies(&mut conn, &[]).unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM mig_currency",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_get_last_update_empty_db() {
        let conn = make_conn();
        let result = get_last_update(&conn).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_get_last_update_after_save() {
        let mut conn = make_conn();
        let currencies = vec![
            Currency { currency: "USD".to_string(), buy: 469.9, sell: 472.9 },
        ];
        save_currencies(&mut conn, &currencies).unwrap();

        let last = get_last_update(&conn).unwrap();
        assert!(last.is_some());
        let dt = last.unwrap();
        let now = Local::now().naive_local();
        let diff = (now - dt).num_seconds().abs();
        assert!(diff <= 2, "time diff {}s", diff);
    }

    #[test]
    fn test_save_and_retrieve_multiple() {
        let mut conn = make_conn();
        let currencies = vec![
            Currency { currency: "USD".to_string(), buy: 469.9, sell: 472.9 },
            Currency { currency: "EUR".to_string(), buy: 536.5, sell: 541.5 },
            Currency { currency: "RUB".to_string(), buy: 5.75, sell: 5.87 },
        ];
        save_currencies(&mut conn, &currencies).unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM mig_currency",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 3);

        let mut stmt = conn.prepare("SELECT mig_currency FROM mig_currency ORDER BY mig_currency").unwrap();
        let rows: Vec<String> = stmt.query_map([], |row| row.get(0)).unwrap()
            .collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(rows, vec!["EUR", "RUB", "USD"]);
    }

    #[test]
    fn test_ensure_schema_creates_telegram_chats_table() {
        let conn = make_conn();
        ensure_schema(&conn).unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='telegram_chats'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 1);
    }
}
