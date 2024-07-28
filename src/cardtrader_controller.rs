use crate::error::CustomError;
use fantoccini::{ClientBuilder, Locator};
use futures::future::join_all;
use headless_chrome::{Browser, LaunchOptionsBuilder};
use indicatif::ProgressBar;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task;
use tokio::time::sleep;

const MAX_CONCURRENT_REQUESTS: usize = 5; // Limite de tarefas paralelas
const MAX_RETRIES: usize = 5; // Número máximo de tentativas

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

pub async fn fetch_multiple_prices_fantoccini(urls: Vec<String>) -> Result<Vec<f64>, CustomError> {
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));
    let mut tasks = Vec::new();
    let pb = ProgressBar::new(urls.len() as u64); // Inicializa a barra de progresso

    for url in urls {
        let semaphore_clone = Arc::clone(&semaphore);
        let pb_clone = pb.clone();
        let task = task::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();

            let cap = serde_json::from_str(
                r#"{"browserName":"chrome","goog:chromeOptions":{"args":["--headless"]}}"#,
            )
            .unwrap();

            let client = ClientBuilder::native()
                .capabilities(cap)
                .connect("http://localhost:9515")
                .await
                .map_err(|e| CustomError::new(&e.to_string()))?;

            client
                .goto(&url)
                .await
                .map_err(|e| CustomError::new(&e.to_string()))?;

            sleep(Duration::from_secs(5)).await;
            client
                .wait()
                .for_element(Locator::Css("div.price-box__price"))
                .await
                .map_err(|e| CustomError::new(&e.to_string()))?;

            let price_text = client
                .find(Locator::Css("div.price-box__price"))
                .await
                .map_err(|e| CustomError::new(&e.to_string()))?
                .text()
                .await
                .map_err(|e| CustomError::new(&e.to_string()))?;

            let price_text = price_text
                .trim()
                .replace("R$", "")
                .replace(" ", "")
                .replace(",", ".");

            let price = if let Ok(price) = price_text.parse::<f64>() {
                price
            } else {
                0.0 // Retorna 0.0 se a conversão falhar
            };

            client
                .close()
                .await
                .map_err(|e| CustomError::new(&e.to_string()))?;

            pb_clone.inc(1); // Incrementa a barra de progresso

            Ok(price) as Result<f64, CustomError>
        });
        tasks.push(task);
    }

    let results = join_all(tasks).await;
    pb.finish_with_message("Done"); // Finaliza a barra de progresso

    let mut prices = Vec::new();
    for result in results {
        match result {
            Ok(Ok(price)) => prices.push(price),
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(CustomError::new(&format!("Task failed: {}", e))),
        }
    }

    Ok(prices)
}
