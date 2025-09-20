use crate::{persistence_sqlite, config::Config};
use teloxide::{prelude::*, utils::command::BotCommands};
use std::sync::Arc;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "add a shopping list item.")]
    Add(String),
    #[command(description = "remove a shopping list item by number.")]
    Remove(u64),
    #[command(description = "show shopping list.")]
    Show,
    #[command(description = "clear shopping list.")]
    Clear,
}

pub struct CommandProcessor {
    db: Arc<persistence_sqlite::ListRepo>,
    config: Arc<Config>,
}

impl CommandProcessor {
    pub async fn new(config: Config) -> Arc<Self> {
        // Use /data directory if it exists (Docker), otherwise /tmp
        let db_path = if std::path::Path::new("/data").exists() {
            "/data/shopping_list.db"
        } else {
            "/tmp/shopping_list.db"
        };

        let db = persistence_sqlite::ListRepo::new(db_path)
            .await
            .expect("Failed to initialize database");

        Arc::new(Self {
            db: Arc::new(db),
            config: Arc::new(config),
        })
    }

    async fn format_list(processor: &Arc<CommandProcessor>) -> String {
        match processor.db.list().await {
            Ok(items) => {
                if items.is_empty() {
                    String::from("\n📋 List is now empty.")
                } else {
                    let mut list_text = String::from("\n\n📋 Current shopping list:\n");
                    for item in items {
                        list_text.push_str(&format!("  {}. {}\n", item.id, item.name));
                    }
                    list_text
                }
            },
            Err(e) => format!("\n❌ Error retrieving list: {}", e)
        }
    }

    pub async fn answer(bot: Bot, msg: Message, cmd: Command, processor: Arc<CommandProcessor>) -> ResponseResult<()> {
        // Check if this chat is allowed
        if !processor.config.is_chat_allowed(msg.chat.id.0) {
            log::warn!("Unauthorized access attempt from chat {}", msg.chat.id);
            bot.send_message(msg.chat.id, "⛔ This bot is not authorized for use in this chat.")
                .await?;
            return Ok(());
        }

        match cmd {
            Command::Help => {
                log::info!("{} used help in {}", msg.from.unwrap().username.unwrap().as_str(), msg.chat.id);
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .await?
            }
            Command::Add(s) => {
                log::info!(
                    "{} used add with {}",
                    msg.from.unwrap().username.unwrap().as_str(),
                    s
                );
                match processor.db.add_item(&s).await {
                    Ok(()) => {
                        let list = Self::format_list(&processor).await;
                        bot.send_message(msg.chat.id, format!("✅ Added '{}' to list{}", s, list)).await?
                    },
                    Err(e) => {
                        bot.send_message(msg.chat.id, format!("❌ {}", e)).await?
                    }
                }
            }
            Command::Remove(i) => {
                log::info!(
                    "{} used remove with {}",
                    msg.from.unwrap().username.unwrap().as_str(),
                    i
                );
                match processor.db.remove_item(i).await {
                    Ok(()) => {
                        let list = Self::format_list(&processor).await;
                        bot.send_message(msg.chat.id, format!("✅ Removed item #{} from list{}", i, list)).await?
                    },
                    Err(e) => {
                        bot.send_message(msg.chat.id, format!("❌ {}", e)).await?
                    }
                }
            }
            Command::Show => {
                log::info!("{} used show", msg.from.unwrap().username.unwrap().as_str());
                match processor.db.list().await {
                    Ok(items) => {
                        if items.is_empty() {
                            bot.send_message(msg.chat.id, "📋 List is empty.").await?
                        } else {
                            let mut list_text = String::from("📋 Shopping list:\n");
                            for item in items {
                                list_text.push_str(&format!("  {}. {}\n", item.id, item.name));
                            }
                            bot.send_message(msg.chat.id, list_text).await?
                        }
                    },
                    Err(e) => bot.send_message(msg.chat.id, format!("❌ Error: {}", e)).await?
                }
            }
            Command::Clear => {
                log::info!(
                    "{} used clear",
                    msg.from.unwrap().username.unwrap().as_str()
                );
                match processor.db.clear().await {
                    Ok(()) => {
                        bot.send_message(msg.chat.id, "🗑️ List cleared.").await?
                    },
                    Err(e) => {
                        bot.send_message(msg.chat.id, format!("❌ {}", e)).await?
                    }
                }
            }
        };

        Ok(())
    }
}
