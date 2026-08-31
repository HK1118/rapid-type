use std::time::Duration;
use crate::stats::Stats;

#[derive(Debug, Clone, PartialEq)]
pub struct Progress {
    pub completed_chars: usize,
    pub total_chars: usize,
    pub guide: String,
    pub typed_romaji: String,
    pub typed_romaji_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuestionStats {
    pub correct_count: usize,
    pub incorrect_count: usize,
    pub elapsed_time: Duration,
    pub accuracy: f64,
    pub kpm: f64,
}

impl QuestionStats {
    pub fn from_stats(stats: &Stats, elapsed: Duration) -> Self {
        let total = stats.correct_count + stats.incorrect_count;
        let accuracy = if total == 0 {
            0.0
        } else {
            stats.correct_count as f64 / total as f64
        };
        let kpm = if elapsed.as_secs_f64() > 0.0 {
            stats.correct_count as f64 / elapsed.as_secs_f64() * 60.0
        } else {
            0.0
        };

        Self {
            correct_count: stats.correct_count,
            incorrect_count: stats.incorrect_count,
            elapsed_time: elapsed,
            accuracy,
            kpm,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputResult {
    Accepted { progress: Progress },
    Rejected { expected: String },
    Completed { stats: QuestionStats },
    AlreadyCompleted,
    TimeUp,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameResult {
    pub total_correct: usize,
    pub total_incorrect: usize,
    pub total_time: Duration,
    pub accuracy: f64,
    pub average_kpm: f64,
    pub questions_completed: usize,
}