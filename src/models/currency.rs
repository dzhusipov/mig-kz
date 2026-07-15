use reqwest::Client;
use scraper::{Html, Selector};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
}

#[derive(Debug, PartialEq)]
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

impl From<ParseErrorKind> for ParseError {
    fn from(kind: ParseErrorKind) -> Self {
        ParseError { kind }
    }
}

pub struct AllCurrencies {
    pub currencies: Vec<Currency>,
}

/// Парсит курсы валют из HTML-страницы mig.kz.
/// Не делает HTTP-запросов — принимает готовый HTML.
pub fn parse_html(html: &str) -> Result<Vec<Currency>, ParseErrorKind> {
    let document = Html::parse_document(html);

    let table_selector = Selector::parse("table tbody tr")
        .map_err(|_| ParseErrorKind::HtmlParseError)?;
    let currency_selector = Selector::parse("td.currency")
        .map_err(|_| ParseErrorKind::HtmlParseError)?;
    let buy_selector = Selector::parse("td.buy")
        .map_err(|_| ParseErrorKind::HtmlParseError)?;
    let sell_selector = Selector::parse("td.sell")
        .map_err(|_| ParseErrorKind::HtmlParseError)?;

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
        return Err(ParseErrorKind::NoCurrenciesFound);
    }

    Ok(currencies)
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

        let currencies = parse_html(&body)?;

        Ok(AllCurrencies { currencies })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_html() -> &'static str {
        r#"
        <html>
        <body>
            <table class="informer">
                <tbody>
                    <tr>
                        <td class="buy delta-neutral">469.9</td>
                        <td class="currency">USD</td>
                        <td class="sell delta-neutral">472.9</td>
                    </tr>
                    <tr>
                        <td class="buy delta-neutral">536.5</td>
                        <td class="currency">EUR</td>
                        <td class="sell delta-neutral">541.5</td>
                    </tr>
                    <tr>
                        <td class="buy delta-neutral">5.75</td>
                        <td class="currency">RUB</td>
                        <td class="sell delta-neutral">5.87</td>
                    </tr>
                </tbody>
            </table>
        </body>
        </html>
        "#
    }

    #[test]
    fn test_parse_html_parses_all_currencies() {
        let currencies = parse_html(sample_html()).expect("should parse");
        assert_eq!(currencies.len(), 3);
        assert_eq!(currencies[0].currency, "USD");
        assert_eq!(currencies[0].buy, 469.9);
        assert_eq!(currencies[0].sell, 472.9);
        assert_eq!(currencies[1].currency, "EUR");
        assert_eq!(currencies[1].buy, 536.5);
        assert_eq!(currencies[1].sell, 541.5);
        assert_eq!(currencies[2].currency, "RUB");
        assert_eq!(currencies[2].buy, 5.75);
        assert_eq!(currencies[2].sell, 5.87);
    }

    #[test]
    fn test_parse_html_with_newlines_and_whitespace() {
        let html = r#"
        <table>
            <tbody>
                <tr>
                    <td class="buy delta-neutral">
469.9
</td>
                    <td class="currency">USD</td>
                    <td class="sell delta-neutral">
    472.9
</td>
                </tr>
            </tbody>
        </table>
        "#;
        let currencies = parse_html(html).expect("should parse with newlines");
        assert_eq!(currencies.len(), 1);
        assert_eq!(currencies[0].currency, "USD");
        assert_eq!(currencies[0].buy, 469.9);
        assert_eq!(currencies[0].sell, 472.9);
    }

    #[test]
    fn test_parse_html_empty_table_returns_error() {
        let html = r#"<html><body><table><tbody></tbody></table></body></html>"#;
        let result = parse_html(html);
        assert_eq!(result.unwrap_err(), ParseErrorKind::NoCurrenciesFound);
    }

    #[test]
    fn test_parse_html_missing_fields_skipped() {
        let html = r#"
        <table>
            <tbody>
                <tr>
                    <td class="buy delta-neutral">469.9</td>
                    <td class="currency">USD</td>
                </tr>
                <tr>
                    <td class="buy delta-neutral">536.5</td>
                    <td class="currency">EUR</td>
                    <td class="sell delta-neutral">541.5</td>
                </tr>
            </tbody>
        </table>
        "#;
        let currencies = parse_html(html).expect("should parse valid row");
        assert_eq!(currencies.len(), 1);
        assert_eq!(currencies[0].currency, "EUR");
    }

    #[test]
    fn test_parse_html_invalid_numbers_skipped() {
        let html = r#"
        <table>
            <tbody>
                <tr>
                    <td class="buy delta-neutral">N/A</td>
                    <td class="currency">USD</td>
                    <td class="sell delta-neutral">472.9</td>
                </tr>
            </tbody>
        </table>
        "#;
        let result = parse_html(html);
        assert_eq!(result.unwrap_err(), ParseErrorKind::NoCurrenciesFound);
    }

    #[test]
    fn test_parse_html_gold_currency() {
        let html = r#"
        <table>
            <tbody>
                <tr>
                    <td class="buy delta-neutral">59550</td>
                    <td class="currency" style="color: #f5eb0d">GOLD</td>
                    <td class="sell delta-neutral">62550</td>
                </tr>
            </tbody>
        </table>
        "#;
        let currencies = parse_html(html).expect("should parse GOLD");
        assert_eq!(currencies[0].currency, "GOLD");
        assert_eq!(currencies[0].buy, 59550.0);
        assert_eq!(currencies[0].sell, 62550.0);
    }

    #[test]
    fn test_parse_html_empty_string() {
        let result = parse_html("");
        assert_eq!(result.unwrap_err(), ParseErrorKind::NoCurrenciesFound);
    }

    #[test]
    fn test_currency_display() {
        let cur = Currency {
            currency: "USD".to_string(),
            buy: 469.9,
            sell: 472.9,
        };
        assert_eq!(format!("{}", cur), "USD: buy 469.90 sell 472.90");
    }

    #[test]
    fn test_currency_display_rounding() {
        let cur = Currency {
            currency: "RUB".to_string(),
            buy: 5.753,
            sell: 5.877,
        };
        assert_eq!(format!("{}", cur), "RUB: buy 5.75 sell 5.88");
    }

    #[test]
    fn test_parse_error_display() {
        let err = ParseError {
            kind: ParseErrorKind::HttpError("timeout".to_string()),
        };
        assert_eq!(format!("{}", err), "HTTP error: timeout");

        let err = ParseError {
            kind: ParseErrorKind::HtmlParseError,
        };
        assert_eq!(format!("{}", err), "Failed to parse HTML");

        let err = ParseError {
            kind: ParseErrorKind::NoCurrenciesFound,
        };
        assert_eq!(format!("{}", err), "No currencies found on page");
    }

    #[test]
    fn test_parse_error_is_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(ParseError {
            kind: ParseErrorKind::NoCurrenciesFound,
        });
        assert!(err.to_string().contains("No currencies found"));
    }

    #[test]
    fn test_parse_html_multiple_currencies() {
        let html = r#"
        <table>
            <tbody>
                <tr><td class="buy delta-neutral">469.9</td><td class="currency">USD</td><td class="sell delta-neutral">472.9</td></tr>
                <tr><td class="buy delta-neutral">536.5</td><td class="currency">EUR</td><td class="sell delta-neutral">541.5</td></tr>
                <tr><td class="buy delta-neutral">5.75</td><td class="currency">RUB</td><td class="sell delta-neutral">5.87</td></tr>
                <tr><td class="buy delta-neutral">5.37</td><td class="currency">KGS</td><td class="sell delta-neutral">5.77</td></tr>
                <tr><td class="buy delta-neutral">627</td><td class="currency">GBP</td><td class="sell delta-neutral">647</td></tr>
                <tr><td class="buy delta-neutral">69.9</td><td class="currency">CNY</td><td class="sell delta-neutral">72.3</td></tr>
                <tr><td class="buy delta-neutral">59550</td><td class="currency">GOLD</td><td class="sell delta-neutral">62550</td></tr>
            </tbody>
        </table>
        "#;
        let currencies = parse_html(html).expect("should parse all 7");
        assert_eq!(currencies.len(), 7);
        assert_eq!(currencies[0].currency, "USD");
        assert_eq!(currencies[6].currency, "GOLD");
    }

    #[test]
    fn test_parse_html_buy_less_than_sell() {
        let currencies = parse_html(sample_html()).expect("should parse");
        for cur in &currencies {
            assert!(
                cur.buy < cur.sell,
                "buy ({}) should be less than sell ({}) for {}",
                cur.buy, cur.sell, cur.currency
            );
        }
    }

    #[test]
    fn test_parse_error_kind_equality() {
        assert_eq!(
            ParseErrorKind::NoCurrenciesFound,
            ParseErrorKind::NoCurrenciesFound
        );
        assert_ne!(
            ParseErrorKind::NoCurrenciesFound,
            ParseErrorKind::HtmlParseError
        );
    }
}
