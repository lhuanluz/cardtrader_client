mod api;
mod auth;
mod blueprints;
mod expansions;
mod prices;
mod telegram;

use inquire::{InquireError, Select};
use std::error::Error;
use whoami;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    let client = reqwest::Client::builder().build()?;
    let headers = auth::get_auth_headers();

    let user_name = whoami::username();
    println!("Hi, {}! welcome to CardTrader.", user_name);

    let expansions = api::fetch_expansions(&client, headers.clone()).await?;

    loop {
        let menu_options: Vec<&str> = vec![
            "Add card",
            "Add card by expansion",
            "Check my list prices",
            "Continuous price check",
            "Update database (Danger)",
            "Exit",
        ];
        let menu_ans: Result<&str, InquireError> =
            Select::new("What would you like to do?", menu_options.clone()).prompt();

        match menu_ans {
            Ok(choice) => match choice {
                "Add card" => blueprints::search_and_select_blueprints(&client, &headers).await?,
                "Add card by expansion" => {
                    expansions::show_expansions(&client, &headers, &expansions).await?
                }
                "Check my list prices" => {
                    prices::check_prices(&client, &headers, &user_name).await?
                }
                "Continuous price check" => {
                    prices::continuous_check_prices(&client, &headers, &user_name).await?
                }
                "Update database (Danger)" => {
                    expansions::save_all_blueprints(&client, &headers, &expansions).await?
                }
                "Exit" => break,
                _ => println!("Invalid choice"),
            },
            Err(_) => println!("There was an error, please try again"),
        }
    }

    Ok(())
}
