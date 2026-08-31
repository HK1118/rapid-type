use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default)]
pub struct Stats {
    pub correct_count: usize,
    pub incorrect_count: usize,
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
}

impl Stats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn end(&mut self) {
        self.end_time = Some(Instant::now());
    }

    pub fn elapsed_time(&self) -> Option<Duration> {
        match (self.start_time, self.end_time) {
            (Some(s), Some(e)) => Some(e.duration_since(s)),
            (Some(s), None) => Some(Instant::now().duration_since(s)),
            _ => None,
        }
    }

    pub fn accuracy(&self) -> f64 {
        let total = self.correct_count + self.incorrect_count;
        if total == 0 {
            0.0
        } else {
            self.correct_count as f64 / total as f64
        }
    }

    pub fn kpm(&self) -> f64 {
        self.elapsed_time()
            .map(|elapsed| elapsed.as_secs_f64())
            .filter(|&secs| secs > 0.0)
            .map_or(0.0, |secs| self.correct_count as f64 / secs * 60.0)
    }
}
