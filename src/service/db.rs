use rusqlite::{params, Connection, Result};
use chrono::{NaiveDateTime, Local};

#[allow(dead_code)]
struct Currency {
    id: i32,
    currency: String,
    buy: String,
    sell: String,
    updated_at: NaiveDateTime,
}
    
impl Currency {
    #[allow(dead_code)]
    fn from_row(row: &rusqlite::Row) -> Self {
        Currency {
            id: row.get(0).unwrap(),
            currency: row.get(1).unwrap(),
            buy: row.get(2).unwrap(),
            sell: row.get(3).unwrap(),
            // current date
            updated_at: Local::now().naive_local(),
        }
    }
}

#[allow(dead_code)]
fn connect_to_database() -> Result<Connection> {
    let conn = Connection::open("db/mig.db")?;
    Ok(conn)
}

#[allow(dead_code)]
fn get_currency(conn: &Connection, id: i32) -> Result<Option<Currency>> {
    let mut stmt = conn.prepare("SELECT id, currency, buy, sell FROM currencies WHERE id = ?1")?;
    let mut rows = stmt.query(params![id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(Currency::from_row(&row)))
    } else {
        Ok(None)
    }
}

#[allow(dead_code)]
fn put_currency(conn: &Connection, currency: &Currency) -> Result<()> {
    conn.execute(
        "INSERT INTO currencies (currency, buy, sell) VALUES (?1, ?2, ?3, ?4)",
        params![
            currency.currency, 
            currency.buy, 
            currency.sell, 
            currency.updated_at
        ],
    )?;

    Ok(())
}

#[allow(dead_code)]
fn update_currency(conn: &Connection, currency: &Currency) -> Result<()> {
    conn.execute(
        "UPDATE currencies SET currency = ?2, buy = ?3, sell = ?4 WHERE id = ?1",
        params![
            currency.id, 
            currency.currency, 
            currency.buy, 
            currency.sell
        ],
    )?;

    Ok(())
}

