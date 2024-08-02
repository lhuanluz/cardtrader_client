mod api;
mod auth;
mod blueprint;
mod blueprint_controller;
mod cache;
mod cards_controller;
mod cardtrader_controller;
mod error;
mod expansion;
mod telegram;
mod wishlist_controller;

use inquire::{InquireError, Select};
use reqwest::Client;
use std::error::Error;
use tokio::main;

#[main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    let client = Client::builder().build()?;
    let headers = auth::get_auth_headers();
    println!("Loading the program, please wait a moment...");

    let expansions = api::fetch_expansions(&client, headers.clone()).await?;

    if !std::path::Path::new("all_blueprints.json").exists() {
        println!("all_blueprints.json not found. Generating it now...");
        blueprint_controller::save_all_blueprints_to_json(
            &client,
            headers.clone(),
            expansions.clone(),
        )
        .await?;
    }

    let blueprint_cache = cache::BlueprintCache::new();
    blueprint_cache.load_cache_from_json("all_blueprints.json")?;

    let user_name = whoami::username();
    println!("Hello, {}! Welcome to CardTrader!", user_name);

    tokio::spawn(async {
        telegram::run().await;
    });

    // Menu interativo
    loop {
        let menu_options: Vec<&str> = vec![
            "Add card",
            "Check prices",
            "Continuos price check",
            "Sync prices (Danger)",
            "Save all blueprints (Danger)",
            "Check with fantoccini",
            "Exit",
        ];
        let menu_ans: Result<&str, InquireError> =
            Select::new("What would you like to do?", menu_options.clone())
                .with_help_message("Use arrow keys to navigate, and Enter to select")
                .prompt();

        match menu_ans {
            Ok(choice) => match choice {
                "Save all blueprints" => {
                    blueprint_controller::save_all_blueprints_to_json(
                        &client,
                        headers.clone(),
                        expansions.clone(),
                    )
                    .await?
                }
                "Add card" => cards_controller::list_and_select_cards(&blueprint_cache).await?,
                "Check prices" => wishlist_controller::check_wishlist_prices().await?,
                "Continuos price check" => wishlist_controller::continuous_check_prices().await?,
                "Sync prices (Danger)" => wishlist_controller::sync_prices().await?,
                "Check with fantoccini" => {
                    cardtrader_controller::check_prices_with_fantoccini().await?
                }
                "Exit" => break,
                _ => println!("Invalid choice"),
            },
            Err(_) => println!("There was an error, please try again"),
        }
    }

    Ok(())
}
