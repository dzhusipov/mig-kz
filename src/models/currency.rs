use reqwest::Client;
use scraper::{Html, Selector};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Currency {
    pub currency: String,
    pub buy: f64,
    pub sell: f64,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: buy {:.2} sell {:.2}", self.currency, self.buy, self.sell)
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub kind: ParseErrorKind,
}

#[derive(Debug)]
pub enum ParseErrorKind {
    HttpError(String),
    HtmlParseError,
    NoCurrenciesFound,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ParseErrorKind::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            ParseErrorKind::HtmlParseError => write!(f, "Failed to parse HTML"),
            ParseErrorKind::NoCurrenciesFound => write!(f, "No currencies found on page"),
        }
    }
}

impl std::error::Error for ParseError {}

pub struct AllCurrencies {
    pub currencies: Vec<Currency>,
}

impl AllCurrencies {
    pub async fn new() -> Result<AllCurrencies, ParseError> {
        let client = Client::builder()
            .user_agent("mig-kz-currency-checker/0.2")
            .build()
            .map_err(|e| ParseError {
                kind: ParseErrorKind::HttpError(format!("Failed to build client: {}", e)),
            })?;

        let response = client
            .get("https://mig.kz/")
            .send()
            .await
            .map_err(|e| ParseError {
                kind: ParseErrorKind::HttpError(format!("Request failed: {}", e)),
            })?;

        if !response.status().is_success() {
            return Err(ParseError {
                kind: ParseErrorKind::HttpError(format!(
                    "HTTP {}: {}",
                    response.status(),
                    response.text().await.unwrap_or_default()
                )),
            });
        }

        let body = response
            .text()
            .await
            .map_err(|e| ParseError {
                kind: ParseErrorKind::HttpError(format!("Failed to read body: {}", e)),
            })?;

        let document = Html::parse_document(&body);

        let table_selector = Selector::parse("table tbody tr")
            .map_err(|_| ParseError { kind: ParseErrorKind::HtmlParseError })?;
        let currency_selector = Selector::parse("td.currency")
            .map_err(|_| ParseError { kind: ParseErrorKind::HtmlParseError })?;
        let buy_selector = Selector::parse("td.buy")
            .map_err(|_| ParseError { kind: ParseErrorKind::HtmlParseError })?;
        let sell_selector = Selector::parse("td.sell")
            .map_err(|_| ParseError { kind: ParseErrorKind::HtmlParseError })?;

        let mut currencies = Vec::new();

        for element in document.select(&table_selector) {
            let currency_text = element
                .select(&currency_selector)
                .next()
                .map(|el| el.inner_html())
                .map(|s| s.replace(|c: char| c.is_whitespace(), " ").trim().to_string())
                .filter(|s| !s.is_empty());

            let buy_text = element
                .select(&buy_selector)
                .next()
                .map(|el| el.inner_html())
                .map(|s| s.replace(|c: char| c.is_whitespace(), " ").trim().to_string())
                .filter(|s| !s.is_empty());

            let sell_text = element
                .select(&sell_selector)
                .next()
                .map(|el| el.inner_html())
                .map(|s| s.replace(|c: char| c.is_whitespace(), " ").trim().to_string())
                .filter(|s| !s.is_empty());

            if let (Some(currency), Some(buy), Some(sell)) =
                (currency_text, buy_text, sell_text)
            {
                if let (Ok(buy_val), Ok(sell_val)) = (buy.parse::<f64>(), sell.parse::<f64>()) {
                    currencies.push(Currency {
                        currency,
                        buy: buy_val,
                        sell: sell_val,
                    });
                }
            }
        }

        if currencies.is_empty() {
            return Err(ParseError {
                kind: ParseErrorKind::NoCurrenciesFound,
            });
        }

        Ok(AllCurrencies { currencies })
    }
}
