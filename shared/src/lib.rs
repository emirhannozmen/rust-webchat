// shared/src/lib.rs
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub sender: String,
    pub content: String,
}

pub fn init_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}