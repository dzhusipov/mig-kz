mod models;
mod service;

use models::currency::AllCurrencies;

#[tokio::main]
async fn main() {
    let currencies = AllCurrencies::new().await;
    for currency in currencies.currencies {
        println!(
            "{}: buy {} sell {}",
            currency.currency, currency.buy, currency.sell,
        );
    }
}
