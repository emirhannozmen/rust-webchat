use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub sender: String,
    pub content: String,
    pub room: String, 
    pub timestamp: String,
}

pub fn init_logger() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init();
}