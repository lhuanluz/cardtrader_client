mod api;
mod auth;
mod blueprint;
mod blueprint_controller;
mod cache;
mod cards_controller;
mod cardtrader_controller;

use inquire::{InquireError, Select};
use reqwest::Client;
use std::error::Error;
use std::fs::File;
use tokio::main;

#[main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    let client = Client::builder().build()?;
    let headers = auth::get_auth_headers();
    println!("Loading the program, please wait a moment...");

    let expansions = api::fetch_expansions(&client, headers.clone()).await?;
    let blueprint_cache = cache::BlueprintCache::new();

    // Verifica se o arquivo all_blueprints.json existe
    if !File::open("all_blueprints.json").is_ok() {
        println!("all_blueprints.json not found. Generating it now...");
        blueprint_controller::save_all_blueprints_to_json(&client, &headers, &expansions).await?;
    }

    blueprint_cache.load_cache_from_json("all_blueprints.json")?;

    let user_name = whoami::username();
    println!("Hello, {}! Welcome to CardTrader!", user_name);

    // Menu interativo
    loop {
        let menu_options: Vec<&str> = vec!["Save all blueprints", "Add card", "Exit"];
        let menu_ans: Result<&str, InquireError> =
            Select::new("What would you like to do?", menu_options.clone())
                .with_help_message("Use arrow keys to navigate, and Enter to select")
                .prompt();

        match menu_ans {
            Ok(choice) => match choice {
                "Save all blueprints" => {
                    blueprint_controller::save_all_blueprints_to_json(
                        &client,
                        &headers,
                        &expansions,
                    )
                    .await?
                }
                "Add card" => cards_controller::list_and_select_cards(&blueprint_cache).await?,
                "Exit" => break,
                _ => println!("Invalid choice"),
            },
            Err(_) => println!("There was an error, please try again"),
        }
    }

    Ok(())
}
