use reqwest::Client;
use scraper::{Html, Selector};
use postgres_types::ToSql;
use postgres_types::FromSql;

#[derive(Debug, ToSql, FromSql)]
pub struct Currency {
    pub currency: String,
    pub buy: String,
    pub sell: String,
}

pub struct AllCurrencies {
    pub currencies: Vec<Currency>,
}

impl AllCurrencies {
    pub async fn new() -> AllCurrencies {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();
        let response = client.get("https://mig.kz/").send().await.unwrap();

        let document = Html::parse_document(&response.text().await.unwrap());

        let table_selector = Selector::parse(".informer tbody tr").unwrap();
        let currency_selector = Selector::parse(".currency").unwrap();
        let buy_selector = Selector::parse(".buy").unwrap();
        let sell_selector = Selector::parse(".sell").unwrap();
        let mut currency_arr = Vec::new();
        for element in document.select(&table_selector) {
            let currency = element.select(&currency_selector).map(|x| x.inner_html());
            let mut currency_str = String::new();
            for c in currency {
                currency_str = c;
            }

            let buy = element.select(&buy_selector).map(|x| x.inner_html());
            let mut buy_str = String::new();
            for b in buy {
                buy_str = b;
            }

            let sell = element.select(&sell_selector).map(|x| x.inner_html());
            let mut sell_str = String::new();
            for s in sell {
                sell_str = s;
            }
            let currency_model = Currency {
                currency: currency_str.to_owned(),
                buy: buy_str.to_owned(),
                sell: sell_str.to_owned(),
            };

            currency_arr.push(currency_model);
        }

        AllCurrencies {
            currencies: currency_arr,
        }
    }
}
