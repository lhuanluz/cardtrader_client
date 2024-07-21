# CardTrader Client

CardTrade Client is a Rust-based application that allows users to track the prices of specific cards on CardTrader and receive notifications via Telegram when the prices drop below a certain threshold.

## Features

- Fetch expansions(duh!?) and blueprints(all cards) from CardTrader.
- Track prices of specific cards.
- Continuous price checking with alerts.
- Telegram integration for notifications.
- Store and load credentials and configuration from `.env` file.

## Prerequisites

- Rust (latest stable version)
- Cargo (latest stable version)
- A Telegram Bot token and chat ID for receiving notifications.
- CardTrader API credentials.

## Installation

1. **Clone the repository:**
 ```
 git clone https://github.com/yourusername/cardtrader-price-checker.git
 cd cardtrader-price-checker
 ```
2. **Create a .env file:**

Copy the `.env-example` to `.env` and fill in your credentials:

```sh
cp .env-example .env
.env
```
#### .env file
```
TELEGRAM_TOKEN=your_telegram_token_here
TELEGRAM_CHAT_ID=your_telegram_chat_id_here
CARD_TRADER_AUTH=Bearer your_card_trader_auth_token_here
CARD_TRADER_COOKIE=_card_trader_session=your_card_trader_cookie_here
```
3. **Run the project:**
```
cargo run
```
