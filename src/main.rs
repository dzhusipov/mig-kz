mod models;
mod service;

use std::fs;
use std::path::PathBuf;
use std::process;

use models::currency::AllCurrencies;
use service::db::{save_currencies, get_last_update};

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("mig_kz=info"),
    )
    .init();

    if let Err(e) = tokio::runtime::Runtime::new()
        .expect("tokio runtime")
        .block_on(run())
    {
        tracing::error!("{}", e);
        process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    tracing::info!("mig-kz currency checker starting");

    let currencies = AllCurrencies::new().await?;

    tracing::info!(
        "Fetched {} currencies from mig.kz",
        currencies.currencies.len()
    );

    for cur in &currencies.currencies {
        println!("{}", cur);
    }

    let db_path = get_db_path();
    tracing::info!("Database path: {}", db_path.display());

    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut conn = rusqlite::Connection::open(&db_path)?;
    save_currencies(&mut conn, &currencies.currencies)?;

    if let Ok(Some(last)) = get_last_update(&conn) {
        tracing::info!("Last DB update: {}", last);
    }

    tracing::info!("Done");
    Ok(())
}

fn get_db_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join("db/mig.db");
            if candidate.exists() {
                return candidate;
            }
        }
    }

    PathBuf::from("db/mig.db")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_db_path_fallback() {
        let path = get_db_path();
        assert_eq!(path, PathBuf::from("db/mig.db"));
    }

    #[test]
    fn test_get_db_path_returns_valid_pathbuf() {
        let path = get_db_path();
        assert!(path.to_string_lossy().contains("mig.db"));
    }
}
