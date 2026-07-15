# mig-kz

Курс валют mig.kz — парсер на Rust с сохранением в SQLite.

## Возможности

- Парсинг курсов покупки/продажи с mig.kz (USD, EUR, RUB, KGS, GBP, CNY, GOLD)
- Сохранение в SQLite с timestamp
- Логирование через `tracing`
- Кросс-компиляция (x86_64 Linux musl)

## Установка

```bash
cargo build --release
```

## Запуск

```bash
# Консольный режим
./target/release/mig-kz

# С детальным логированием
RUST_LOG=mig_kz=debug ./target/release/mig-kz
```

Вывод:
```
USD: buy 469.90 sell 472.90
EUR: buy 536.50 sell 541.50
RUB: buy 5.75 sell 5.87
...
```

## Docker

```bash
docker build -t mig-kz .
docker run mig-kz
```

## Тесты

```bash
cargo test
```

## Структура

```
src/
├── main.rs           — точка входа, логирование, сохранение в БД
├── models/
│   ├── currency.rs   — парсер HTML + Currency/ParseError
└── service/
    └── db.rs         — SQLite: ensure_schema, save_currencies, get_last_update
```

## Зависимости

| Crate | Назначение |
|---|---|
| `reqwest` (rustls) | HTTP-запросы |
| `scraper` | HTML-парсинг (CSS selectors) |
| `rusqlite` (bundled) | SQLite |
| `tokio` | Async runtime |
| `tracing` | Логирование |
| `chrono` | Даты |
| `anyhow` | Error handling |
