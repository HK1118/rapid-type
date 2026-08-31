use crate::question::Question;
use crate::result::{GameResult, InputResult, Progress, QuestionStats};
use crate::stats::Stats;
use crate::status::Status;
use rand::seq::SliceRandom;
use std::time::{Duration, Instant};
use typing_engine::TypingEngine;

#[derive(Debug, Clone)]
pub enum GameMode {
    Normal {
        questions: Vec<Question>,
    },
    TimeAttack {
        time_limit: Duration,
        pool: Vec<Question>,
    },
}

pub struct Session {
    mode: GameMode,
    questions: Vec<Question>,
    current_question_idx: usize,
    active_engine: Option<TypingEngine>,
    pub stats: Stats,
    question_stats: Vec<QuestionStats>,
    pub status: Status,
    start_time: Option<Instant>,
    end_time: Option<Instant>,
    question_start_time: Option<Instant>,
    typed_romaji_count: usize,
    typed_romaji: String,
}

impl Session {
    pub fn new(mode: GameMode) -> Self {
        let (questions, _time_limit) = match &mode {
            GameMode::Normal { questions } => (questions.clone(), None),
            GameMode::TimeAttack { time_limit, pool } => {
                let mut questions = pool.clone();
                questions.shuffle(&mut rand::rng());
                (questions, Some(*time_limit))
            }
        };

        Self {
            mode,
            questions,
            current_question_idx: 0,
            active_engine: None,
            stats: Stats::new(),
            question_stats: Vec::new(),
            status: Status::Ready,
            start_time: None,
            end_time: None,
            question_start_time: None,
            typed_romaji_count: 0,
            typed_romaji: String::new(),
        }
    }

    pub fn start(&mut self) {
        self.status = Status::Playing;
        self.start_time = Some(Instant::now());
        self.load_current_question();
    }

    pub fn submit_input(&mut self, input: char) -> InputResult {
        if self.status != Status::Playing {
            return InputResult::AlreadyCompleted;
        }

        if self.is_time_up() {
            self.status = Status::TimeUp;
            return InputResult::TimeUp;
        }

        // let input = c.to_ascii_lowercase();

        let result = {
            let Some(engine) = self.active_engine.as_mut() else {
                return InputResult::AlreadyCompleted;
            };
            engine.input(input)
        };

        let mut progress = self.build_progress(self.active_engine.as_ref().unwrap());
        let expected = self
            .active_engine
            .as_ref()
            .unwrap()
            .guide()
            .chars()
            .next()
            .map(|c| c.to_string())
            .unwrap_or_default();

        match result {
            typing_engine::EngineInputResult::Accepted => {
                self.stats.correct_count += 1;
                self.typed_romaji_count += 1;
                self.typed_romaji.push(input);
                progress.typed_romaji.push(input);
                progress.typed_romaji_count += 1;
                InputResult::Accepted { progress }
            }
            typing_engine::EngineInputResult::Rejected => {
                self.stats.incorrect_count += 1;
                InputResult::Rejected { expected }
            }
            typing_engine::EngineInputResult::Completed => {
                self.stats.correct_count += 1;
                self.typed_romaji_count += 1;
                self.typed_romaji.push(input);
                progress.typed_romaji.push(input);
                progress.typed_romaji_count += 1;
                let stats = self.finish_question();
                InputResult::Completed { stats }
            }
            typing_engine::EngineInputResult::AlreadyCompleted => {
                let stats = self.finish_question();
                InputResult::Completed { stats }
            }
        }
    }

    pub fn current_question(&self) -> Option<&Question> {
        self.questions.get(self.current_question_idx)
    }

    pub fn current_question_index(&self) -> usize {
        self.current_question_idx
    }

    pub fn total_questions(&self) -> usize {
        self.questions.len()
    }

    pub fn current_progress(&self) -> Option<Progress> {
        self.active_engine
            .as_ref()
            .map(|engine| self.build_progress(engine))
    }

    pub fn remaining_time(&self) -> Option<Duration> {
        let time_limit = match &self.mode {
            GameMode::TimeAttack { time_limit, .. } => *time_limit,
            GameMode::Normal { .. } => return None,
        };
        let elapsed = self.start_time.map(|s| s.elapsed()).unwrap_or_default();
        time_limit.checked_sub(elapsed)
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.status, Status::Completed | Status::TimeUp)
    }

    pub fn game_result(&self) -> Option<GameResult> {
        if !self.is_finished() {
            return None;
        }
        let total_time = self
            .start_time
            .zip(self.end_time)
            .map(|(s, e)| e.duration_since(s))
            .unwrap_or_default();
        let total_correct: usize = self.question_stats.iter().map(|s| s.correct_count).sum();
        let total_incorrect: usize = self.question_stats.iter().map(|s| s.incorrect_count).sum();
        let questions_completed = self.question_stats.len();

        let accuracy = if total_correct + total_incorrect == 0 {
            0.0
        } else {
            total_correct as f64 / (total_correct + total_incorrect) as f64
        };

        let average_kpm = if total_time.as_secs_f64() > 0.0 {
            total_correct as f64 / total_time.as_secs_f64() * 60.0
        } else {
            0.0
        };

        Some(GameResult {
            total_correct,
            total_incorrect,
            total_time,
            accuracy,
            average_kpm,
            questions_completed,
        })
    }

    fn load_current_question(&mut self) {
        if self.current_question_idx >= self.questions.len() {
            self.finish_game();
            return;
        }

        let question = &self.questions[self.current_question_idx];
        match TypingEngine::new(&question.reading) {
            Ok(engine) => {
                self.active_engine = Some(engine);
                self.question_start_time = Some(Instant::now());
                self.stats.start();
                self.typed_romaji_count = 0;
                self.typed_romaji.clear();
            }
            Err(_) => {
                self.current_question_idx += 1;
                self.load_current_question();
            }
        }
    }

    fn build_progress(&self, engine: &TypingEngine) -> Progress {
        Progress {
            completed_chars: engine.completed_char_count(),
            total_chars: engine
                .furthest_completed_char_count()
                .max(engine.completed_char_count()),
            guide: engine.guide().to_string(),
            typed_romaji: self.typed_romaji.clone(),
            typed_romaji_count: self.typed_romaji_count,
        }
    }

    fn finish_question(&mut self) -> QuestionStats {
        let elapsed = self
            .question_start_time
            .map(|s| s.elapsed())
            .unwrap_or_default();
        let q_stats = QuestionStats::from_stats(&self.stats, elapsed);
        self.question_stats.push(q_stats.clone());

        self.current_question_idx += 1;
        self.stats = Stats::new();

        if self.current_question_idx >= self.questions.len() || self.is_time_up() {
            self.finish_game();
        } else {
            self.load_current_question();
        }

        q_stats
    }

    fn finish_game(&mut self) {
        self.status = if self.is_time_up() {
            Status::TimeUp
        } else {
            Status::Completed
        };
        self.end_time = Some(Instant::now());
        self.active_engine = None;
        self.stats.end();
    }

    fn is_time_up(&self) -> bool {
        match &self.mode {
            GameMode::TimeAttack { time_limit, .. } => self
                .start_time
                .map(|s| s.elapsed() >= *time_limit)
                .unwrap_or(false),
            GameMode::Normal { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::question::Question;
    use crate::status::Status;

    #[test]
    fn test_session_loads_first_question() {
        let problems = vec![Question::new("かんかんにおこる")];
        let mode = GameMode::Normal {
            questions: problems,
        };
        let mut session = Session::new(mode);
        session.start();

        assert_eq!(session.status, Status::Playing);
        assert!(session.current_question().is_some());
        assert!(session.active_engine.is_some());
        println!(
            "First question guide: {}",
            session.current_progress().unwrap().guide
        );
    }
}
