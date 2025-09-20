use std::env;

pub struct Config {
    token: String,
    allowed_chat_ids: Vec<i64>,
}

impl Config {
    pub fn new() -> Result<Config, String> {
        let token = get_env_var("BOT_TOKEN")?;

        let allowed_chat_ids = match env::var("ALLOWED_CHAT_IDS") {
            Ok(ids_str) => {
                ids_str
                    .split(',')
                    .filter_map(|s| s.trim().parse::<i64>().ok())
                    .collect()
            },
            Err(_) => Vec::new(), // If not set, allow all chats (empty list means no restrictions)
        };

        Ok(Config {
            token,
            allowed_chat_ids,
        })
    }

    pub fn bot_token(&self) -> &str {
        &self.token
    }

    pub fn is_chat_allowed(&self, chat_id: i64) -> bool {
        // If no chat IDs are configured, allow all
        if self.allowed_chat_ids.is_empty() {
            true
        } else {
            self.allowed_chat_ids.contains(&chat_id)
        }
    }
}

fn get_env_var(name: &str) -> Result<String, String> {
    let result = env::var(name);
    if result.is_err() {
        return Err(format!("{} is not defined", name));
    }
    Ok(result.unwrap())
}