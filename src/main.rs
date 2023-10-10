use std::collections::HashMap;
use reqwest::Client;
use scraper::{Html, Selector};

#[tokio::main]
async fn main() {
    let client = Client::new();
    let response = client.get("https://mig.kz/exchange/").send().await.unwrap();
    let document = Html::parse_document(&response.text().unwrap());

    let mut exchange_rates = HashMap::new();

    let selector = Selector::parse(".informer tbody tr").unwrap();
    for element in document.select(&selector) {
        let currency = element.select(".currency").first().unwrap().text().contents().collect::<Vec<_>>()[0].clone();
        let buy = element.select(".buy").first().unwrap().text().contents().collect::<Vec<_>>()[0].clone();
        let sell = element.select(".sell").first().unwrap().text().contents().collect::<Vec<_>>()[0].clone();

        exchange_rates.insert(currency, (buy, sell));
    }

    for (currency, (buy, sell)) in exchange_rates.iter() {
        println!("{}: buy {} sell {}", currency, buy, sell);
    }
}
