# mig-kz

Курсы валют mig.kz — парсер на Rust с HTTP-запросом, парсингом HTML и сохранением в SQLite.

[![Build Status](https://img.shields.io/badge/tests-27_passing-brightgreen)](https://github.com/dzhusipov/mig-kz)

## Что делает

1. Делает HTTP-запрос к `https://mig.kz/`
2. Парсит HTML-таблицу с курсами валют (CSS selectors)
3. Выводит курсы в консоль
4. Сохраняет все курсы в SQLite с timestamp

## Курсы

Парсит 7 валют:

| Валюта | Описание |
|---|---|
| USD | Доллар США |
| EUR | Евро |
| RUB | Российский рубль |
| KGS | Киргизский сом |
| GBP | Фунт стерлингов |
| CNY | Китайский юань |
| GOLD | Золото |

## Быстрый старт

### Требования

- Rust 1.75+ (`rustup`)
- SQLite (включён через `rusqlite` — bundled, ничего ставить не надо)

### Установка и запуск

```bash
# Клонировать
git clone https://github.com/dzhusipov/mig-kz.git
cd mig-kz

# Собрать
cargo build --release

# Запустить
./target/release/mig-kz
```

### Вывод

```
USD: buy 469.90 sell 472.90
EUR: buy 536.50 sell 541.50
RUB: buy 5.75  sell 5.87
KGS: buy 5.37  sell 5.77
GBP: buy 627.00 sell 647.00
CNY: buy 69.90  sell 72.30
GOLD: buy 59550.00 sell 62550.00
```

## Логирование

Проект использует [`tracing`](https://crates.io/crates/tracing). Уровень логирования контролируется через `RUST_LOG`:

```bash
# Только ошибки
RUST_LOG=mig_kz=error ./target/release/mig-kz

# Информационный (по умолчанию)
RUST_LOG=mig_kz=info ./target/release/mig-kz

# Детальный (debug)
RUST_LOG=mig_kz=debug ./target/release/mig-kz

# Всё (trace)
RUST_LOG=trace ./target/release/mig-kz
```

### Пример вывода с логированием

```
2026-07-15T08:29:00.972556Z  INFO mig_kz: mig-kz currency checker starting
2026-07-15T08:29:01.093716Z  INFO mig_kz: Fetched 7 currencies from mig.kz
USD: buy 469.90 sell 472.90
EUR: buy 536.50 sell 541.50
RUB: buy 5.75 sell 5.87
KGS: buy 5.37 sell 5.77
GBP: buy 627.00 sell 647.00
CNY: buy 69.90 sell 72.30
GOLD: buy 59550.00 sell 62550.00
2026-07-15T08:29:01.093789Z  INFO mig_kz: Database path: db/mig.db
2026-07-15T08:29:01.098115Z  INFO mig_kz: Last DB update: 2026-07-15 08:29:01
2026-07-15T08:29:01.098130Z  INFO mig_kz: Done
```

## SQLite

### Схема базы данных

```sql
CREATE TABLE mig_currency (
    mig_id           INTEGER PRIMARY KEY AUTOINCREMENT,
    mig_currency     VARCHAR NOT NULL,    -- USD, EUR, RUB...
    mig_currency_buy REAL NOT NULL,       -- Курс покупки
    mig_currency_sell REAL NOT NULL,      -- Курс продажи
    mig_created      TEXT NOT NULL        -- Timestamp
);

CREATE TABLE telegram_chats (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id   INTEGER,
    chat_name TEXT
);
```

### Пример запроса

```bash
sqlite3 db/mig.db "SELECT mig_currency, mig_currency_buy, mig_currency_sell, mig_created FROM mig_currency ORDER BY mig_created DESC LIMIT 5;"
```

```
USD|472.9|469.9|2026-07-15 08:29:01
EUR|541.5|536.5|2026-07-15 08:29:01
RUB|5.87|5.75|2026-07-15 08:29:01
KGS|5.77|5.37|2026-07-15 08:29:01
GBP|647.0|627.0|2026-07-15 08:29:01
```

### Путь к базе данных

- В Docker: `db/mig.db` рядом с бинарником
- Локально: `db/mig.db` в текущей директории

## Docker

### Сборка и запуск

```bash
docker build -t mig-kz .
docker run mig-kz
```

### Dockerfile

```dockerfile
FROM rust:1.75-slim AS build
WORKDIR /app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:3.19
WORKDIR /app
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/mig-kz /app/mig-kz
COPY --from=build /app/db /app/db
CMD ["/app/mig-kz"]
```

Multi-stage build:
1. **build** — Rust-образ с `musl` для статической линковки (нет зависимостей на хосте)
2. **alpine** — минимальный образ (~5MB) с готовым бинарником

## Тесты

```bash
# Все тесты
cargo test

# Только unit-тесты (без network)
cargo test --lib

# Только интеграционные тесты (с реальным HTTP)
cargo test integration_tests -- --nocapture

# С выводом результатов
cargo test -- --nocapture
```

### Coverage

```
running 27 tests
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Структура тестов

| Модуль | Кол-во | Что тестирует |
|---|---|---|
| `models/currency.rs` | 14 | `parse_html()` — парсинг HTML, newlines, пустая таблица, пропуск невалидных, GOLD, 7 валют, `Display`, `ParseError`, `PartialEq` |
| `service/db.rs` | 9 | `ensure_schema()` — создание, idempotent; `save_currencies()` — insert, values, empty list; `get_last_update()` — empty, after save |
| `main.rs` | 2 | `get_db_path()` — fallback, valid PathBuf |
| `models/currency.rs` (integration) | 2 | Реальный HTTP-запрос к mig.kz, валидация 7 валют, USD range, buy < sell |

### Пример unit-теста

```rust
#[test]
fn test_parse_html_parses_all_currencies() {
    let currencies = parse_html(sample_html()).expect("should parse");
    assert_eq!(currencies.len(), 3);
    assert_eq!(currencies[0].currency, "USD");
    assert_eq!(currencies[0].buy, 469.9);
    assert_eq!(currencies[0].sell, 472.9);
}
```

### Пример integration-теста

```rust
#[tokio::test]
async fn test_real_mig_kz_request() {
    let client = reqwest::Client::builder()
        .user_agent("mig-kz-currency-checker/0.2-test")
        .build().unwrap();

    let response = client
        .get("https://mig.kz/")
        .send()
        .await
        .expect("HTTP request to mig.kz should succeed");

    assert!(response.status().is_success());

    let body = response.text().await.expect("Should read body");
    let currencies = parse_html(&body).expect("parse_html should succeed");

    assert!(currencies.len() >= 7, "Expected at least 7 currencies");

    let usd = currencies.iter().find(|c| c.currency == "USD").unwrap();
    assert!(usd.buy < usd.sell, "buy < sell for USD");
    assert!(usd.buy > 100.0 && usd.buy < 1000.0, "USD in reasonable range");
}
```

## Структура проекта

```
mig-kz/
├── Cargo.toml              — зависимости и настройки сборки
├── Cargo.lock              — lockfile
├── Dockerfile              — multi-stage Docker build
├── Makefile                — build, test, docker targets
├── README.md               — этот файл
├── .gitignore
├── db/
│   └── mig.db              — SQLite база данных (создаётся автоматически)
└── src/
    ├── main.rs             — точка входа: логирование, HTTP, парсинг, сохранение
    ├── models/
    │   ├── mod.rs          — экспорт модулей
    │   └── currency.rs     — Currency, ParseError, AllCurrencies, parse_html()
    └── service/
        ├── mod.rs          — экспорт модулей
        └── db.rs           — SQLite: ensure_schema, save_currencies, get_last_update
```

## Архитектура

```
┌─────────────────────────────────────────────────┐
│                    main.rs                      │
│  tracing → AllCurrencies::new() → save_currencies│
└──────────────────┬──────────────────────────────┘
                   │
          ┌────────▼────────┐
          │  AllCurrencies  │
          │  ::new() [async]│
          └────────┬────────┘
                   │
      ┌────────────▼────────────┐
      │  reqwest → GET mig.kz   │
      │  → HTML body            │
      └────────────┬────────────┘
                   │
          ┌────────▼────────┐
          │   parse_html()  │
          │  scraper (CSS)  │
          └────────┬────────┘
                   │
      ┌────────────▼────────────┐
      │  Vec<Currency>          │
      │  [USD, EUR, RUB, ...]   │
      └────────────┬────────────┘
                   │
          ┌────────▼────────┐
          │   save_currencies│
          │   (SQLite tx)   │
          └─────────────────┘
```

## Зависимости

| Crate | Версия | Назначение |
|---|---|---|
| `reqwest` | 0.11.22 | HTTP-клиент (rustls, без OpenSSL) |
| `scraper` | 0.18.1 | HTML-парсинг через CSS selectors |
| `rusqlite` | 0.29.0 | SQLite (bundled — компилирует C-код) |
| `tokio` | 1.33.0 | Async runtime (full) |
| `tracing` | 0.1.40 | Structured logging |
| `tracing-subscriber` | 0.3.18 | EnvFilter для RUST_LOG |
| `chrono` | 0.4.31 | DateTime, timestamps |
| `anyhow` | 1.0.86 | Ergonomic error propagation |

## Release-профиль

```toml
[profile.release]
strip = true        # Удаляет symbols → бинарник меньше
opt-level = "z"     # Оптимизация по размеру
lto = true          # Link Time Optimization
codegen-units = 1   # Одна единица компиляции → лучший оптимиз
```

Размер release-бинарника: ~2-3 MB (статически линкованный, musl).

## Error Handling

```rust
// Типизированные ошибки
pub enum ParseErrorKind {
    HttpError(String),       // HTTP-ошибка (timeout, 404...)
    HtmlParseError,          // Невалидный CSS-селектор
    NoCurrenciesFound,       // На странице нет валют
}

// Пример использования
match AllCurrencies::new().await {
    Ok(data) => { /* ... */ }
    Err(e) => {
        tracing::error!("Parse error: {}", e);
        // e.kind() → ParseErrorKind::HttpError("timeout")
    }
}
```

## Как добавить новую валюту

Ничего менять не нужно. Парсер извлекает **все** строки из таблицы `.informer` на mig.kz. Если mig.kz добавит новую валюту — она автоматически появится при следующем запуске.

## Лицензия

MIT
