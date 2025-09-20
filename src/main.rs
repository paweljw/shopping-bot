mod config;
mod command;
mod persistence_sqlite;

use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let config = match config::Config::new() {
        Ok(config) => config,
        Err(err) => panic!("{}", err),
    };

    let bot = teloxide::Bot::new(config.bot_token());

    let me = bot.get_me().send().await.unwrap();
    log::info!("Bot starting as {:?}", me);

    let command_processor = command::CommandProcessor::new(config).await;

    command::Command::repl(bot, move |bot, msg, cmd| {
        let processor = command_processor.clone();
        command::CommandProcessor::answer(bot, msg, cmd, processor)
    }).await;
}
