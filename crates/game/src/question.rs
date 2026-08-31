use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Question {
    pub id: String,
    pub reading: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub difficulty: u8,
}

impl Question {
    // 問題の読みと表示は一旦同じ文字列にする。
    pub fn new(reading: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            reading: reading.into(),
            category: String::new(),
            difficulty: 0,
        }
    }
}
