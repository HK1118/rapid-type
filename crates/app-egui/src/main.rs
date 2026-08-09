#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use eframe::egui::text::LayoutJob;
use std::time::{Duration, Instant};
use typing_engine::{EngineInputResult, TypingEngine};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rapid Type",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(TypingGameApp::new()))
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "jp".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/NotoSansJP-Regular.otf"))
            .into(),
    );

    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        family.insert(0, "jp".to_owned());
    }
    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        family.insert(0, "jp".to_owned());
    }

    ctx.set_fonts(fonts);

    let mut style = (*ctx.global_style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(42.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(28.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(24.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(24.0, egui::FontFamily::Monospace),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(20.0, egui::FontFamily::Proportional),
    );
    ctx.set_global_style(style);
}

struct TypingGameApp {
    problems: Vec<&'static str>,
    current_problem_idx: usize,
    engine: TypingEngine,
    typed_guide_input: String,
    status_message: String,
    last_new_duration: Duration,
    last_input_duration: Duration,
}

impl TypingGameApp {
    fn new() -> Self {
        let problems = vec![
            "かんかんにおこる",
            "じゅげむじゅげむごこうのすりきれかいじゃりすいぎょのすいぎょうまつうんらいまつふうらいまつくうねるところにすむところやぶらこうじのぶらこうじぱいぽぱいぽぱいぽのしゅーりんがんしゅーりんがんのぐーりんだいぐーりんだいのぽんぽこぴーのぽんぽこなーのちょうきゅうめいのちょうすけ",
            "らーめん",
            "きょう",
            "がっこう",
            "にとをおうものいっとをもえず",
            "きゅうきゅうしゃ",
            "はっしゃ",
        ];
        let start = Instant::now();
        let engine = TypingEngine::new(problems[0]).expect("initial problem must be valid");
        let last_new_duration = start.elapsed();

        Self {
            problems,
            current_problem_idx: 0,
            engine,
            typed_guide_input: String::new(),
            status_message: "Type to start".to_string(),
            last_new_duration,
            last_input_duration: Duration::ZERO,
        }
    }

    fn current_problem(&self) -> &str {
        self.problems[self.current_problem_idx]
    }

    fn reset_current_problem(&mut self) {
        let current = self.current_problem().to_string();
        let start = Instant::now();
        self.engine = TypingEngine::new(&current).expect("problem must be valid");
        self.last_new_duration = start.elapsed();
        self.typed_guide_input.clear();
    }

    fn advance_problem(&mut self) {
        self.current_problem_idx = (self.current_problem_idx + 1) % self.problems.len();
        self.reset_current_problem();
    }

    fn handle_char_input(&mut self, c: char) {
        let input = c.to_ascii_lowercase();
        let start = Instant::now();
        let result = self.engine.input(input);
        self.last_input_duration = start.elapsed();
        match result {
            EngineInputResult::Accepted => {
                self.typed_guide_input.push(input);
                self.status_message = format!("Accepted: {input}");
            }
            EngineInputResult::Rejected => {
                self.status_message = format!("Rejected: {input}");
            }
            EngineInputResult::Completed => {
                self.typed_guide_input.push(input);
                self.advance_problem();
                self.status_message = "Completed! Next problem loaded.".to_string();
            }
            EngineInputResult::AlreadyCompleted => {
                self.status_message = "Already completed. Moving to next problem.".to_string();
                self.advance_problem();
            }
        }
    }
}

impl eframe::App for TypingGameApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let events = ui.ctx().input(|i| i.events.clone());
        for event in events {
            if let egui::Event::Text(text) = event {
                for c in text.chars() {
                    if c.is_ascii_control() || c.is_whitespace() {
                        continue;
                    }
                    self.handle_char_input(c);
                }
            }
        }

        ui.label(format!(
            "Problem ({}/{})",
            self.current_problem_idx + 1,
            self.problems.len(),
        ));
        let problem_anchor_ratio = 0.35;
        let (done, current, remaining) = anchored_progress_segments_by_width(
            ui.ctx(),
            self.current_problem(),
            self.engine.completed_char_count(),
            ui.available_width(),
            problem_anchor_ratio,
            egui::FontId::new(56.0, egui::FontFamily::Proportional),
        );
        ui.add(
            egui::Label::new(colored_progress_job(
                &done, &current, &remaining, 56.0, true,
            ))
            .extend(),
        );

        ui.label("Guide");
        let full_guide = format!("{}{}", self.typed_guide_input, self.engine.guide());
        let typed_count = self.typed_guide_input.chars().count();
        let (guide_done, guide_current, guide_remaining) = anchored_progress_segments_by_width(
            ui.ctx(),
            &full_guide,
            typed_count,
            ui.available_width(),
            problem_anchor_ratio,
            egui::FontId::new(44.0, egui::FontFamily::Monospace),
        );
        ui.add(
            egui::Label::new(colored_progress_job(
                &guide_done,
                &guide_current,
                &guide_remaining,
                44.0,
                false,
            ))
            .extend(),
        );
        ui.label(format!(
            "Progress: {} / {}",
            self.engine.completed_char_count(),
            self.current_problem().chars().count()
        ));
        ui.label(format!("Completed: {}", self.engine.completed_reading()));
        ui.label(format!(
            "Furthest: {}",
            self.engine.furthest_completed_reading()
        ));
        ui.separator();
        ui.label(format!("Status: {}", self.status_message));
        ui.label(format!(
            "Debug timings: new={} / input={}",
            format_duration(self.last_new_duration),
            format_duration(self.last_input_duration)
        ));
        let problem_anchor_idx = self
            .engine
            .completed_char_count()
            .min(self.current_problem().chars().count().saturating_sub(1));
        let guide_anchor_idx = typed_count.min(full_guide.chars().count().saturating_sub(1));
        ui.label(format!(
            "Debug anchor idx: problem={} / guide={} (target {:.0}%)",
            problem_anchor_idx,
            guide_anchor_idx,
            problem_anchor_ratio * 100.0
        ));
        ui.label("Type keys while this window is focused.");

        if ui.button("Skip to next problem").clicked() {
            self.advance_problem();
            self.status_message = "Skipped to next problem.".to_string();
        }
    }
}

fn anchored_progress_segments_by_width(
    ctx: &egui::Context,
    text: &str,
    completed_chars: usize,
    max_width: f32,
    anchor_ratio: f32,
    font_id: egui::FontId,
) -> (String, String, String) {
    let chars: Vec<char> = text.chars().collect();
    let total_chars = chars.len();
    if total_chars == 0 {
        return (String::new(), String::new(), String::new());
    }

    let completed = completed_chars.min(total_chars);
    let current_idx = completed.min(total_chars.saturating_sub(1));

    let mut prefix_widths = Vec::with_capacity(total_chars + 1);
    prefix_widths.push(0.0f32);
    for &ch in &chars {
        let glyph_width = ctx.fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(ch.to_string(), font_id.clone(), egui::Color32::WHITE)
                .size()
                .x
        });
        let next = prefix_widths.last().copied().unwrap_or(0.0) + glyph_width;
        prefix_widths.push(next);
    }

    let clamped_width = max_width.max(font_id.size * 4.0);
    let target_left = (prefix_widths[current_idx] - clamped_width * anchor_ratio).max(0.0);
    let mut start_char = prefix_widths
        .partition_point(|&w| w <= target_left)
        .saturating_sub(1);
    start_char = start_char.min(current_idx);

    let right_limit = prefix_widths[start_char] + clamped_width;
    let mut end_char = prefix_widths.partition_point(|&w| w <= right_limit);
    end_char = end_char.clamp(start_char + 1, total_chars);
    if end_char <= current_idx {
        end_char = current_idx + 1;
    }

    if completed >= total_chars {
        let done: String = chars[start_char..end_char].iter().collect();
        return (done, String::new(), String::new());
    }

    let visible_completed = completed
        .saturating_sub(start_char)
        .min(end_char - start_char);
    let done_end = start_char + visible_completed;
    let done: String = chars[start_char..done_end].iter().collect();

    if done_end < end_char {
        let current = chars[done_end].to_string();
        let remaining: String = chars[(done_end + 1)..end_char].iter().collect();
        (done, current, remaining)
    } else {
        (done, String::new(), String::new())
    }
}

fn format_duration(duration: Duration) -> String {
    let micros = duration.as_micros();
    if micros >= 1_000 {
        format!("{:.3} ms", micros as f64 / 1_000.0)
    } else {
        format!("{micros} µs")
    }
}

fn colored_progress_job(
    done: &str,
    current: &str,
    remaining: &str,
    size: f32,
    is_japanese: bool,
) -> LayoutJob {
    let mut job = LayoutJob::default();

    let done_color = egui::Color32::from_gray(80);
    let current_color = egui::Color32::from_rgb(255, 220, 90);
    let remaining_color = egui::Color32::from_gray(180);
    let family = if is_japanese {
        egui::FontFamily::Proportional
    } else {
        egui::FontFamily::Monospace
    };

    if !done.is_empty() {
        job.append(
            done,
            0.0,
            egui::TextFormat {
                font_id: egui::FontId::new(size, family.clone()),
                color: done_color,
                ..Default::default()
            },
        );
    }
    if !current.is_empty() {
        job.append(
            current,
            0.0,
            egui::TextFormat {
                font_id: egui::FontId::new(size, family.clone()),
                color: current_color,
                ..Default::default()
            },
        );
    }
    if !remaining.is_empty() {
        job.append(
            remaining,
            0.0,
            egui::TextFormat {
                font_id: egui::FontId::new(size, family),
                color: remaining_color,
                ..Default::default()
            },
        );
    }

    job
}
