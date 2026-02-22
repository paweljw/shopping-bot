mod config;
mod command;
mod persistence_sqlite;
mod api;

use std::sync::Arc;
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let config = match config::Config::new() {
        Ok(config) => config,
        Err(err) => panic!("{}", err),
    };

    let db_path = if std::path::Path::new("/data").exists() {
        "/data/shopping_list.db"
    } else {
        "/tmp/shopping_list.db"
    };

    let db = Arc::new(
        persistence_sqlite::ListRepo::new(db_path)
            .await
            .expect("Failed to initialize database"),
    );

    let bot = teloxide::Bot::new(config.bot_token());

    if let Some(api_token) = config.api_token() {
        let notify_chat_ids: Vec<ChatId> = config
            .allowed_chat_ids()
            .iter()
            .map(|&id| ChatId(id))
            .collect();
        let api_state = Arc::new(api::ApiState {
            db: db.clone(),
            api_token: api_token.to_string(),
            bot: bot.clone(),
            notify_chat_ids,
        });
        let port = config.api_port();
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .expect("Failed to bind API port");
        log::info!("API server listening on port {}", port);
        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, api::router(api_state)).await {
                log::error!("API server error: {}", e);
                std::process::exit(1);
            }
        });
    }

    let me = bot.get_me().send().await.unwrap();
    log::info!("Bot starting as {:?}", me);

    let command_processor = command::CommandProcessor::new(config, db).await;

    command::Command::repl(bot, move |bot, msg, cmd| {
        let processor = command_processor.clone();
        command::CommandProcessor::answer(bot, msg, cmd, processor)
    }).await;
}
