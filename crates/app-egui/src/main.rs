#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod render;

use eframe::egui;
use game::{Difficulty, GameMode, Question, Session};
use rand::seq::SliceRandom;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rapid Type",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Title,
    DifficultySelect,
    Playing,
    Result,
}

struct TypingGameApp {
    screen: Screen,
    session: Option<Session>,
    selected_difficulty: Difficulty,
    status_message: String,
}

impl TypingGameApp {
    fn new() -> Self {
        Self {
            screen: Screen::Title,
            session: None,
            selected_difficulty: Difficulty::default(),
            status_message: String::new(),
        }
    }

    fn start_game(&mut self) {
        let mut rng = rand::rng();
        let mut all_problems = self.selected_difficulty.pool();
        all_problems.shuffle(&mut rng);
        let problems: Vec<Question> = all_problems.into_iter().take(10).collect();

        let mode = GameMode::Normal {
            questions: problems,
        };
        let mut session = Session::new(mode);
        session.start();

        self.session = Some(session);
        self.screen = Screen::Playing;
        self.status_message = "Type to start".to_string();
    }

    fn handle_char_input(&mut self, c: char) {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        if session.is_finished() {
            return;
        }

        let result = session.submit_input(c);
        match result {
            game::InputResult::Accepted { progress } => {
                self.status_message = format!(
                    "Accepted: {} ({}/{})",
                    c.to_ascii_lowercase(),
                    progress.completed_chars,
                    progress.total_chars
                );
            }
            game::InputResult::Rejected { expected } => {
                self.status_message = format!(
                    "Rejected: {} (expected: {})",
                    c.to_ascii_lowercase(),
                    expected
                );
            }
            game::InputResult::Completed { stats } => {
                self.status_message = format!(
                    "Completed! Accuracy: {:.1}%, KPM: {:.1}",
                    stats.accuracy * 100.0,
                    stats.kpm
                );
            }
            game::InputResult::AlreadyCompleted => {
                self.status_message = "Already completed".to_string();
            }
            game::InputResult::TimeUp => {
                self.status_message = "Time up!".to_string();
            }
        }

        if session.is_finished() {
            self.screen = Screen::Result;
        }
    }

    fn ui_title(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            // --- 1. コンテンツ全体の高さを取得して上部余白を計算 ---
            let content_id = ui.id().with("title_content");
            let prev_height: f32 = ui
                .ctx()
                .data_mut(|d| d.get_temp(content_id))
                .unwrap_or(360.0);

            let available_height = ui.available_height();
            // 0.45〜0.5 を掛けることで上下中央（少し上寄りの心地よい位置）にする
            let top_space = ((available_height - prev_height) * 0.45).max(0.0);
            ui.add_space(top_space);

            // --- 2. タイトル画面のコンテンツ ---
            let response = ui.scope(|ui| {
                // 画像
                ui.add(
                    egui::Image::new(egui::include_image!("../assets/images/logo.png"))
                        .max_width(500.0),
                );
                ui.add_space(15.0);

                ui.label("タイピング練習");
                ui.add_space(30.0);

                if ui
                    .add_sized([220.0, 50.0], egui::Button::new("開始"))
                    .clicked()
                {
                    self.screen = Screen::DifficultySelect;
                }
                ui.add_space(10.0);

                if ui
                    .add_sized([220.0, 50.0], egui::Button::new("終了"))
                    .clicked()
                {
                    std::process::exit(0);
                }
            });

            // --- 3. 今回描画されたコンテンツの高さを保存（次フレームで使用） ---
            let actual_height = response.response.rect.height();
            ui.ctx()
                .data_mut(|d| d.insert_temp(content_id, actual_height));
        });
    }

    fn ui_difficulty_select(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(30.0);
            ui.heading("難易度を選択");
            ui.add_space(20.0);

            ui.vertical_centered(|ui| {
                let is_easy = self.selected_difficulty == Difficulty::Easy;
                let is_normal = self.selected_difficulty == Difficulty::Normal;
                let is_hard = self.selected_difficulty == Difficulty::Hard;

                let btn_size = egui::vec2(220.0, 50.0);

                if ui
                    .add_sized(btn_size, egui::Button::selectable(is_easy, "イージー"))
                    .clicked()
                {
                    self.selected_difficulty = Difficulty::Easy;
                }

                if ui
                    .add_sized(btn_size, egui::Button::selectable(is_normal, "ノーマル"))
                    .clicked()
                {
                    self.selected_difficulty = Difficulty::Normal;
                }

                if ui
                    .add_sized(btn_size, egui::Button::selectable(is_hard, "ハード"))
                    .clicked()
                {
                    self.selected_difficulty = Difficulty::Hard;
                }
            });

            ui.add_space(20.0);

            if ui
                .add_sized([220.0, 50.0], egui::Button::new("ゲームを開始"))
                .clicked()
            {
                self.start_game();
            }

            ui.add_space(10.0);
            if ui
                .add_sized([220.0, 50.0], egui::Button::new("タイトルに戻る"))
                .clicked()
            {
                self.screen = Screen::Title;
            }
        });
    }

    fn ui_playing(&mut self, ui: &mut egui::Ui) {
        // キー入力イベントの処理
        ui.ctx().input(|i| {
            for event in &i.events {
                if let egui::Event::Text(text) = event {
                    for mut c in text.chars() {
                        // 改行やタブ・制御文字のみスキップ（スペースは許可）
                        if c == '\n' || c == '\r' || c == '\t' || c.is_ascii_control() {
                            continue;
                        }
                        // 全角スペースを半角スペースに正規化
                        if c == '　' {
                            c = ' ';
                        }
                        self.handle_char_input(c);
                    }
                }
            }
        });

        if let Some(session) = self.session.as_ref() {
            if let Some(question) = session.current_question() {
                ui.label(format!(
                    "問題 ({}/{}) - {}",
                    session.current_question_index() + 1,
                    session.total_questions(),
                    self.selected_difficulty.label()
                ));

                ui.add_space(20.0);

                let anchor_ratio = 0.35; // 現在打っている文字の画面横位置（左から35%）
                if let Some(progress) = session.current_progress() {
                    // --- 1段目: 漢字表示テキスト ---
                    let display_completed =
                        question.display_completed_chars(progress.completed_chars);
                    let (disp_done, disp_curr, disp_rem) =
                        render::anchored_progress_segments_by_width(
                            ui.ctx(),
                            &question.display,
                            display_completed,
                            ui.available_width(),
                            anchor_ratio,
                            egui::FontId::new(48.0, egui::FontFamily::Proportional),
                        );
                    ui.add(
                        egui::Label::new(render::colored_progress_job(
                            &disp_done, &disp_curr, &disp_rem, 48.0, true,
                        ))
                        .extend(),
                    );

                    ui.add_space(5.0);

                    // --- 2段目: ひらがな読み ---
                    let (read_done, read_curr, read_rem) =
                        render::anchored_progress_segments_by_width(
                            ui.ctx(),
                            &question.reading,
                            progress.completed_chars,
                            ui.available_width(),
                            anchor_ratio,
                            egui::FontId::new(26.0, egui::FontFamily::Proportional),
                        );
                    ui.add(
                        egui::Label::new(render::colored_progress_job(
                            &read_done, &read_curr, &read_rem, 26.0, true,
                        ))
                        .extend(),
                    );

                    ui.add_space(5.0);

                    // --- 3段目: ローマ字ガイド ---
                    let full_guide = format!("{}{}", progress.typed_romaji, progress.guide);
                    let typed_count = progress.typed_romaji_count;
                    let (guide_done, guide_current, guide_remaining) =
                        render::anchored_progress_segments_by_width(
                            ui.ctx(),
                            &full_guide,
                            typed_count,
                            ui.available_width(),
                            anchor_ratio,
                            egui::FontId::new(36.0, egui::FontFamily::Monospace),
                        );
                    ui.add(
                        egui::Label::new(render::colored_progress_job(
                            &guide_done,
                            &guide_current,
                            &guide_remaining,
                            36.0,
                            false,
                        ))
                        .extend(),
                    );
                }
            }

            if cfg!(debug_assertions) {
                ui.add_space(20.0);
                ui.label(format!("Status: {}", self.status_message));
                ui.label(format!("Game Status: {:?}", session.status));
                if let Some(remaining) = session.remaining_time() {
                    ui.label(format!("Remaining: {:.1}s", remaining.as_secs_f64()));
                }
            }
        }
    }

    fn ui_result(&mut self, ui: &mut egui::Ui) {
        if let Some(session) = self.session.as_ref()
            && let Some(result) = session.game_result()
        {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.heading("結果");
                ui.add_space(20.0);

                ui.label(format!("難易度: {}", self.selected_difficulty.label()));
                ui.label(format!("ミスタイプ数: {}回", result.total_incorrect));
                ui.label(format!("正確性: {:.1}%", result.accuracy * 100.0));
                ui.label(format!("タイプ数/分: {:.1}", result.average_kpm));
                ui.label(format!(
                    "合計時間: {:.1}秒",
                    result.total_time.as_secs_f64()
                ));

                ui.add_space(30.0);
                if ui
                    .add_sized([220.0, 50.0], egui::Button::new("もう一度遊ぶ"))
                    .clicked()
                {
                    self.start_game();
                }
                ui.add_space(10.0);
                if ui
                    .add_sized([220.0, 50.0], egui::Button::new("タイトルに戻る"))
                    .clicked()
                {
                    self.screen = Screen::Title;
                    self.session = None;
                }
            });
        }
    }
}

impl eframe::App for TypingGameApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        match self.screen {
            Screen::Title => self.ui_title(ui),
            Screen::DifficultySelect => self.ui_difficulty_select(ui),
            Screen::Playing => self.ui_playing(ui),
            Screen::Result => self.ui_result(ui),
        }
    }
}
