use serde::{Deserialize, Serialize};

use crate::ruby::{RubySegment, calculate_display_progress, parse_ruby};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Question {
    pub id: String,
    pub display: String,
    pub reading: String,
    pub segments: Vec<RubySegment>,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub difficulty: u8,
}

impl Question {
    // 問題の読みと表示は一旦同じ文字列にする。
    pub fn new(reading: impl Into<String>) -> Self {
        let r = reading.into();
        Self::from_ruby(&r)
    }

    pub fn from_ruby(text: &str) -> Self {
        let segments = parse_ruby(text).unwrap_or_else(|err| {
            panic!("問題のルビ記法に誤りがあります: {err}");
        });

        let display: String = segments.iter().map(|s| s.display.as_str()).collect();
        let reading: String = segments.iter().map(|s| s.reading.as_str()).collect();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            display,
            reading,
            segments,
            category: String::new(),
            difficulty: 0,
        }
    }

    pub fn display_completed_chars(&self, reading_completed: usize) -> usize {
        calculate_display_progress(&self.segments, reading_completed)
    }
}
