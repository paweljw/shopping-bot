# Shopping Bot

A Telegram bot for managing shopping lists with SQLite persistence.

## Features

- Add items to shopping list
- Remove items by ID
- View current list
- Clear entire list
- Persistent storage using SQLite
- Multi-chat support with optional access control

## Commands

- `/help` - Display available commands
- `/add <item>` - Add an item to the shopping list
- `/remove <id>` - Remove an item by its ID
- `/show` - Display the current shopping list
- `/clear` - Clear the entire shopping list

## Deployment

### Using Docker Compose (Recommended)

1. Clone the repository
2. Copy `.env.example` to `.env` and configure:
   ```bash
   cp .env.example .env
   ```
3. Edit `.env` and set your bot token:
   ```
   BOT_TOKEN=your_bot_token_from_botfather
   ```
4. (Optional) Set allowed chat IDs for access control:
   ```
   ALLOWED_CHAT_IDS=123456789,-987654321
   ```
5. Build and run:
   ```bash
   docker-compose up -d
   ```

### Using Docker

Build the image:
```bash
docker build -t shopping-bot .
```

Run the container:
```bash
docker run -d \
  --name shopping-bot \
  -e BOT_TOKEN=your_bot_token \
  -e ALLOWED_CHAT_IDS=123456789,-987654321 \
  -v shopping-bot-data:/data \
  shopping-bot
```

### Local Development

1. Install Rust
2. Set environment variables:
   ```bash
   export BOT_TOKEN=your_bot_token
   export ALLOWED_CHAT_IDS=123456789,-987654321  # Optional
   ```
3. Run the bot:
   ```bash
   cargo run
   ```

## Environment Variables

- `BOT_TOKEN` (required): Your Telegram bot token from @BotFather
- `ALLOWED_CHAT_IDS` (optional): Comma-separated list of allowed chat IDs. Leave empty to allow all chats.
- `RUST_LOG` (optional): Log level (error, warn, info, debug, trace)

## Data Storage

- In Docker: Data is stored in `/data/shopping_list.db` (persisted via volume)
- Local development: Data is stored in `/tmp/shopping_list.db`