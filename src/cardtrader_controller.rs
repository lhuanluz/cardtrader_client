use headless_chrome::{Browser, LaunchOptionsBuilder};
use std::error::Error;
use std::time::Duration;

pub async fn fetch_card_price(
    card_name: &str,
    expansion_name: &str,
    version: &str,
) -> Result<f64, Box<dyn Error>> {
    let clean_card_name = card_name
        .replace(' ', "-")
        .replace(",", "")
        .replace("'", "")
        .replace(".", "")
        .replace(":", "")
        .to_lowercase();

    let clean_expansion_name = expansion_name
        .replace(' ', "-")
        .replace(",", "")
        .replace("'", "")
        .replace(".", "")
        .replace(":", "")
        .to_lowercase();

    let clean_version = version
        .replace(' ', "-")
        .replace(",", "")
        .replace("'", "")
        .replace(".", "")
        .replace(":", "")
        .to_lowercase();

    let url = format!(
        "https://www.cardtrader.com/cards/{}-{}-{}",
        clean_card_name, clean_version, clean_expansion_name
    );

    println!("Fetching URL: {}", url);

    let browser = Browser::new(
        LaunchOptionsBuilder::default()
            .headless(true)
            .build()
            .unwrap(),
    )?;
    let tab = browser.new_tab()?;
    tab.navigate_to(&url)?;
    tab.wait_until_navigated()?;
    std::thread::sleep(Duration::from_secs(5)); // Espera adicional para garantir que a página carregue completamente

    let price_element = tab.find_element("div.price-box__price")?;
    let price_text = price_element.get_inner_text()?;
    let price_text = price_text
        .trim()
        .replace("R$", "")
        .replace(" ", "")
        .replace(",", ".");

    println!("Raw price text: {}", price_text);

    let price: f64 = price_text.parse().map_err(|_| "Failed to parse price")?;
    Ok(price)
}
