use std::env;

pub struct Config {
    token: String,
    allowed_chat_ids: Vec<i64>,
    api_token: Option<String>,
    api_port: u16,
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

        let api_token = env::var("API_TOKEN").ok();

        let api_port = env::var("API_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        Ok(Config {
            token,
            allowed_chat_ids,
            api_token,
            api_port,
        })
    }

    pub fn bot_token(&self) -> &str {
        &self.token
    }

    pub fn is_chat_allowed(&self, chat_id: i64) -> bool {
        if self.allowed_chat_ids.is_empty() {
            true
        } else {
            self.allowed_chat_ids.contains(&chat_id)
        }
    }

    pub fn api_token(&self) -> Option<&str> {
        self.api_token.as_deref()
    }

    pub fn api_port(&self) -> u16 {
        self.api_port
    }
}

fn get_env_var(name: &str) -> Result<String, String> {
    let result = env::var(name);
    if result.is_err() {
        return Err(format!("{} is not defined", name));
    }
    Ok(result.unwrap())
}