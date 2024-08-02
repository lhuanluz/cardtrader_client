use crate::error::CustomError;
use crate::wishlist_controller::WishlistItem;
use fantoccini::wd::Capabilities;
use fantoccini::ClientBuilder;
use fantoccini::Locator;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use headless_chrome::{Browser, LaunchOptionsBuilder};
use indicatif::ProgressBar;
use regex::Regex;
use std::error::Error;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

const MAX_CONCURRENT_REQUESTS: usize = 15; // Limite de tarefas paralelas
const MAX_RETRIES: usize = 2; // Número máximo de tentativas

async fn fetch_price(
    url: String,
    browser: Arc<Browser>,
    semaphore: Arc<Semaphore>,
) -> Result<f64, CustomError> {
    let _permit = semaphore
        .acquire()
        .await
        .map_err(|e| CustomError::new(&e.to_string()))?;

    for _ in 0..MAX_RETRIES {
        let tab_result = browser.new_tab();
        if let Ok(tab) = tab_result {
            let navigate_result = tab.navigate_to(&url);
            if let Ok(_) = navigate_result {
                let wait_result = tab.wait_until_navigated();
                if let Ok(_) = wait_result {
                    std::thread::sleep(Duration::from_secs(5)); // Espera adicional para garantir que a página carregue completamente
                    let price_element_result = tab.find_element("div.price-box__price");
                    if let Ok(price_element) = price_element_result {
                        let price_text_result = price_element.get_inner_text();
                        if let Ok(price_text) = price_text_result {
                            let price_text = price_text
                                .trim()
                                .replace("R$", "")
                                .replace(" ", "")
                                .replace(",", ".");

                            if let Ok(price) = price_text.parse::<f64>() {
                                return Ok(price);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(0.0) // Retorna 0.0 se todas as tentativas falharem
}

pub async fn fetch_card_price(
    card_name: &str,
    expansion_name: &str,
    version: &str,
) -> Result<f64, CustomError> {
    let clean_card_name = card_name
        .replace(' ', "-")
        .replace(",", "")
        .replace("'", "-")
        .replace(".", "")
        .replace(":", "")
        .to_lowercase();

    let clean_expansion_name = expansion_name
        .replace(' ', "-")
        .replace(",", "")
        .replace("'", "-")
        .replace(".", "")
        .replace(":", "")
        .to_lowercase();

    let clean_version = version
        .replace(' ', "-")
        .replace(",", "")
        .replace("'", "-")
        .replace(".", "")
        .replace(":", "")
        .to_lowercase();

    let url = if version.is_empty() {
        format!(
            "https://www.cardtrader.com/cards/{}-{}",
            clean_card_name, clean_expansion_name
        )
    } else {
        format!(
            "https://www.cardtrader.com/cards/{}-{}-{}",
            clean_card_name, clean_version, clean_expansion_name
        )
    };

    let browser = Arc::new(
        Browser::new(
            LaunchOptionsBuilder::default()
                .headless(true)
                .build()
                .unwrap(),
        )
        .map_err(|e| CustomError::new(&e.to_string()))?,
    );
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));

    let price = fetch_price(url, browser, semaphore).await?;

    Ok(price)
}

pub async fn check_prices_with_fantoccini() -> Result<(), Box<dyn Error>> {
    // Ler o arquivo wishlist.json
    let wishlist_data = fs::read_to_string("wishlist.json")?;
    let mut wishlist: Vec<WishlistItem> = serde_json::from_str(&wishlist_data)?;

    // Configurar capacidades para o Chrome

    // Configurar o cliente fantoccini
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));
    let mut tasks = FuturesUnordered::new();
    let pb = ProgressBar::new(wishlist.len() as u64);

    for item in wishlist.iter_mut() {
        let mut item = item.clone(); // Clone para mover para dentro da task
        let semaphore = semaphore.clone();

        let pb_clone = pb.clone();
        tasks.push(tokio::spawn(async move {
            let cap: Capabilities = serde_json::from_str(
                r#"{"browserName":"chrome","goog:chromeOptions":{"args":["--headless"]}}"#,
            )
            .unwrap();
            let _permit = semaphore.acquire().await.unwrap(); // Aqui usamos o clone do semaphore

            let c = ClientBuilder::native()
                .capabilities(cap.clone())
                .connect("http://localhost:9515")
                .await
                .expect("failed to connect to WebDriver");

            // Preparar a URL com base nas informações do item
            let clean_card_name = item
                .card_name
                .replace(' ', "-")
                .replace(",", "")
                .replace("'", "-")
                .replace(".", "")
                .replace(":", "")
                .to_lowercase();

            let clean_expansion_name = {
                let re = Regex::new(r"'(\w)|'\s").unwrap();
                re.replace_all(&item.expansion_name, |caps: &regex::Captures| {
                    if let Some(capture) = caps.get(1) {
                        // O apóstrofo é seguido por uma letra
                        format!("-{}", capture.as_str())
                    } else {
                        // O apóstrofo é seguido por um espaço, remover o apóstrofo
                        String::new()
                    }
                })
                .replace("/", "-")
                .replace(' ', "-")
                .replace(",", "")
                .replace(".", "")
                .replace(":", "")
                .to_lowercase()
                .to_string()
            };
            let clean_version = item
                .version
                .replace(' ', "-")
                .replace(",", "")
                .replace("'", "-")
                .replace(".", "")
                .replace(":", "")
                .to_lowercase();

            let url = if item.version.is_empty() {
                format!(
                    "https://www.cardtrader.com/cards/{}-{}",
                    clean_card_name, clean_expansion_name
                )
            } else {
                format!(
                    "https://www.cardtrader.com/cards/{}-{}-{}",
                    clean_card_name, clean_version, clean_expansion_name
                )
            };

            // Navegar para a URL e obter o preço
            if let Err(e) = c.goto(&url).await {
                println!("Failed to navigate to URL: {}", e);
                return item; // Retorna o item sem alterar o preço
            }

            if let Err(e) = c
                .wait()
                .for_element(Locator::Css("div.price-box__price"))
                .await
            {
                //println!("Failed to wait for element on url: {}", url);
                //println!("Element not found: {}", e);
                return item; // Retorna o item sem alterar o preço
            }

            // Obter o preço atual
            if let Ok(price_element) = c.find(Locator::Css("div.price-box__price")).await {
                if let Ok(price_text) = price_element.text().await {
                    let price_text = price_text
                        .trim()
                        .replace("R$", "")
                        .replace(" ", "")
                        .replace(",", ".");

                    if let Ok(current_price) = price_text.parse() {
                        item.price = current_price;
                    }
                }
            }
            pb_clone.inc(1);

            /*println!(
                "Price for {} - {} - {} is {}",
                item.card_name, item.expansion_name, item.version, item.price
            );*/

            // Fecha o cliente
            c.close().await.expect("Failed to close client");

            item
        }));
    }

    // Processar todas as tarefas
    while let Some(res) = tasks.next().await {
        match res {
            Ok(updated_item) => {
                if let Some(item) = wishlist.iter_mut().find(|i| {
                    i.card_name == updated_item.card_name
                        && i.expansion_name == updated_item.expansion_name
                        && i.version == updated_item.version
                }) {
                    item.price = updated_item.price;
                }
            }
            Err(e) => println!("Task failed: {:?}", e),
        }
    }

    pb.finish_with_message("Finished checking prices");
    // Salvar a wishlist atualizada
    let updated_wishlist_data = serde_json::to_string(&wishlist)?;
    fs::write("wishlist.json", updated_wishlist_data)?;

    println!("Prices checked and wishlist updated.");

    Ok(())
}
