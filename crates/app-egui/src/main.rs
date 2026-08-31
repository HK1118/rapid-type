#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod render;

use eframe::egui;
use game::{Difficulty, GameMode, Question, Session};
use rand::seq::SliceRandom;
use std::time::{Duration, Instant};

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
        egui::FontId::new(38.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(26.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(22.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(24.0, egui::FontFamily::Monospace),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(15.0, egui::FontFamily::Proportional),
    );
    ctx.set_global_style(style);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum PlayMode {
    #[default]
    Normal,
    TimeAttack,
}

impl PlayMode {
    fn label(&self) -> &'static str {
        match self {
            PlayMode::Normal => "ノーマル (10問)",
            PlayMode::TimeAttack => "タイムアタック",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Title,
    ModeSelect,
    DifficultySelect,
    Ready,
    Countdown,
    Playing,
    Result,
}

struct TypingGameApp {
    screen: Screen,
    session: Option<Session>,
    selected_mode: PlayMode,
    selected_difficulty: Difficulty,
    status_message: String,
    title_cursor: usize,  // 0: 開始, 1: 終了
    mode_cursor: usize,   // 0: ノーマル, 1: タイムアタック
    result_cursor: usize, // 0: もう一度遊ぶ, 1: タイトルに戻る
    countdown_start: Option<Instant>,
}

impl TypingGameApp {
    fn new() -> Self {
        Self {
            screen: Screen::Title,
            session: None,
            selected_mode: PlayMode::default(),
            selected_difficulty: Difficulty::default(),
            status_message: String::new(),
            title_cursor: 0,
            mode_cursor: 0,
            result_cursor: 0,
            countdown_start: None,
        }
    }

    fn start_game(&mut self) {
        let mut rng = rand::rng();
        let mut all_problems = self.selected_difficulty.pool();
        all_problems.shuffle(&mut rng);

        let mode = match self.selected_mode {
            PlayMode::Normal => {
                let problems: Vec<Question> = all_problems.into_iter().take(10).collect();
                GameMode::Normal {
                    questions: problems,
                }
            }
            PlayMode::TimeAttack => GameMode::TimeAttack {
                time_limit: Duration::from_secs(60),
                pool: all_problems,
            },
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
            self.result_cursor = 0;
            self.screen = Screen::Result;
        }
    }

    fn ui_title(&mut self, ui: &mut egui::Ui) {
        if ui.input(|i| i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown)) {
            self.title_cursor = (self.title_cursor + 1).min(1);
        }
        if ui.input(|i| i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp)) {
            self.title_cursor = self.title_cursor.saturating_sub(1);
        }
        if ui.input(|i| {
            i.key_pressed(egui::Key::L)
                || i.key_pressed(egui::Key::Space)
                || i.key_pressed(egui::Key::Enter)
        }) {
            if self.title_cursor == 0 {
                self.screen = Screen::ModeSelect;
            } else {
                std::process::exit(0);
            }
        }
        if ui.input(|i| i.key_pressed(egui::Key::Q) || i.key_pressed(egui::Key::Escape)) {
            std::process::exit(0);
        }

        ui.vertical_centered(|ui| {
            let content_id = ui.id().with("title_content");
            let prev_height: f32 = ui
                .ctx()
                .data_mut(|d| d.get_temp(content_id))
                .unwrap_or(360.0);

            let available_height = ui.available_height();
            let top_space = ((available_height - prev_height) * 0.42).max(0.0);
            ui.add_space(top_space);

            let response = ui.scope(|ui| {
                ui.add(
                    egui::Image::new(egui::include_image!("../assets/images/logo.png"))
                        .max_width(460.0),
                );
                ui.add_space(10.0);

                ui.label("タイピング練習ゲーム");
                ui.add_space(25.0);

                let btn_size = egui::vec2(200.0, 46.0);

                let is_start_selected = self.title_cursor == 0;
                let is_exit_selected = self.title_cursor == 1;

                if ui
                    .add_sized(
                        btn_size,
                        egui::Button::selectable(is_start_selected, "開始"),
                    )
                    .clicked()
                {
                    self.title_cursor = 0;
                    self.screen = Screen::ModeSelect;
                }
                ui.add_space(8.0);

                if ui
                    .add_sized(btn_size, egui::Button::selectable(is_exit_selected, "終了"))
                    .clicked()
                {
                    self.title_cursor = 1;
                    std::process::exit(0);
                }

                ui.add_space(25.0);
                ui.label(
                    egui::RichText::new("[↑ / ↓] 移動　　[Space / Enter] 決定　　[q / Esc] 終了")
                        .color(egui::Color32::from_gray(130))
                        .size(15.0),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("vim風操作も可能")
                        .color(egui::Color32::from_gray(130))
                        .size(15.0),
                );
            });

            let actual_height = response.response.rect.height();
            ui.ctx()
                .data_mut(|d| d.insert_temp(content_id, actual_height));
        });
    }

    /// モード選択画面 (ノーマル / タイムアタック)
    fn ui_mode_select(&mut self, ui: &mut egui::Ui) {
        if ui.input(|i| i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown)) {
            self.mode_cursor = (self.mode_cursor + 1).min(1);
        }
        if ui.input(|i| i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp)) {
            self.mode_cursor = self.mode_cursor.saturating_sub(1);
        }

        self.selected_mode = if self.mode_cursor == 0 {
            PlayMode::Normal
        } else {
            PlayMode::TimeAttack
        };

        if ui.input(|i| {
            i.key_pressed(egui::Key::L)
                || i.key_pressed(egui::Key::Space)
                || i.key_pressed(egui::Key::Enter)
        }) {
            self.screen = Screen::DifficultySelect;
        }
        if ui.input(|i| {
            i.key_pressed(egui::Key::H)
                || i.key_pressed(egui::Key::Q)
                || i.key_pressed(egui::Key::Escape)
        }) {
            self.screen = Screen::Title;
        }

        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading("モードを選択");
            ui.add_space(30.0);

            let btn_size = egui::vec2(260.0, 50.0);
            let is_normal = self.selected_mode == PlayMode::Normal;
            let is_time_attack = self.selected_mode == PlayMode::TimeAttack;

            if ui
                .add_sized(btn_size, egui::Button::selectable(is_normal, "ノーマル"))
                .clicked()
            {
                self.mode_cursor = 0;
                self.selected_mode = PlayMode::Normal;
            }
            ui.add_space(8.0);

            if ui
                .add_sized(
                    btn_size,
                    egui::Button::selectable(is_time_attack, "タイムアタック"),
                )
                .clicked()
            {
                self.mode_cursor = 1;
                self.selected_mode = PlayMode::TimeAttack;
            }

            ui.add_space(35.0);

            let action_btn_size = egui::vec2(220.0, 46.0);
            if ui
                .add_sized(action_btn_size, egui::Button::new("次へ進む"))
                .clicked()
            {
                self.screen = Screen::DifficultySelect;
            }
            ui.add_space(8.0);
            if ui
                .add_sized(action_btn_size, egui::Button::new("タイトルに戻る"))
                .clicked()
            {
                self.screen = Screen::Title;
            }

            ui.add_space(25.0);
            ui.label(
                egui::RichText::new("[↑ / ↓] 移動　　[Space / Enter] 決定　　[q / Esc] 戻る")
                    .color(egui::Color32::from_gray(130))
                    .size(15.0),
            );
        });
    }

    fn ui_difficulty_select(&mut self, ui: &mut egui::Ui) {
        let difficulties = [Difficulty::Easy, Difficulty::Normal, Difficulty::Hard];
        let mut current_idx = match self.selected_difficulty {
            Difficulty::Easy => 0,
            Difficulty::Normal => 1,
            Difficulty::Hard => 2,
        };

        if ui.input(|i| i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown)) {
            current_idx = (current_idx + 1).min(difficulties.len() - 1);
            self.selected_difficulty = difficulties[current_idx];
        }
        if ui.input(|i| i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp)) {
            current_idx = current_idx.saturating_sub(1);
            self.selected_difficulty = difficulties[current_idx];
        }

        if ui.input(|i| {
            i.key_pressed(egui::Key::L)
                || i.key_pressed(egui::Key::Space)
                || i.key_pressed(egui::Key::Enter)
        }) {
            self.screen = Screen::Ready;
        }

        if ui.input(|i| {
            i.key_pressed(egui::Key::H)
                || i.key_pressed(egui::Key::Q)
                || i.key_pressed(egui::Key::Escape)
        }) {
            self.screen = Screen::ModeSelect;
        }

        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.heading("難易度を選択");
            ui.add_space(20.0);

            ui.vertical_centered(|ui| {
                let is_easy = self.selected_difficulty == Difficulty::Easy;
                let is_normal = self.selected_difficulty == Difficulty::Normal;
                let is_hard = self.selected_difficulty == Difficulty::Hard;

                let btn_size = egui::vec2(220.0, 44.0);

                if ui
                    .add_sized(btn_size, egui::Button::selectable(is_easy, "イージー"))
                    .clicked()
                {
                    self.selected_difficulty = Difficulty::Easy;
                }
                ui.add_space(4.0);

                if ui
                    .add_sized(btn_size, egui::Button::selectable(is_normal, "ノーマル"))
                    .clicked()
                {
                    self.selected_difficulty = Difficulty::Normal;
                }
                ui.add_space(4.0);

                if ui
                    .add_sized(btn_size, egui::Button::selectable(is_hard, "ハード"))
                    .clicked()
                {
                    self.selected_difficulty = Difficulty::Hard;
                }
            });

            ui.add_space(25.0);

            let action_btn_size = egui::vec2(220.0, 46.0);

            if ui
                .add_sized(action_btn_size, egui::Button::new("次へ進む"))
                .clicked()
            {
                self.screen = Screen::Ready;
            }

            ui.add_space(8.0);
            if ui
                .add_sized(action_btn_size, egui::Button::new("モード選択に戻る"))
                .clicked()
            {
                self.screen = Screen::ModeSelect;
            }

            ui.add_space(25.0);
            ui.label(
                egui::RichText::new("[↑ / ↓] 移動　　[Space / Enter] 決定　　[q / Esc] 戻る")
                    .color(egui::Color32::from_gray(130))
                    .size(15.0),
            );
        });
    }

    fn ui_ready(&mut self, ui: &mut egui::Ui) {
        if ui.input(|i| {
            i.key_pressed(egui::Key::Space)
                || i.key_pressed(egui::Key::Enter)
                || i.key_pressed(egui::Key::L)
        }) {
            self.countdown_start = Some(Instant::now());
            self.screen = Screen::Countdown;
        }

        if ui.input(|i| {
            i.key_pressed(egui::Key::H)
                || i.key_pressed(egui::Key::Q)
                || i.key_pressed(egui::Key::Escape)
        }) {
            self.screen = Screen::DifficultySelect;
        }

        ui.vertical_centered(|ui| {
            ui.add_space(90.0);

            ui.label(
                egui::RichText::new(format!(
                    "{}  /  {}",
                    self.selected_mode.label(),
                    self.selected_difficulty.label()
                ))
                .color(egui::Color32::from_gray(180))
                .size(20.0),
            );
            ui.add_space(25.0);

            ui.label(
                egui::RichText::new("Spaceキーを押して開始")
                    .color(egui::Color32::from_rgb(255, 220, 90))
                    .size(42.0),
            );

            ui.add_space(45.0);

            let btn_size = egui::vec2(220.0, 46.0);

            if ui
                .add_sized(btn_size, egui::Button::new("難易度選択に戻る"))
                .clicked()
            {
                self.screen = Screen::DifficultySelect;
            }

            ui.add_space(35.0);
            ui.label(
                egui::RichText::new("[q / Esc] 戻る")
                    .color(egui::Color32::from_gray(130))
                    .size(15.0),
            );
        });
    }

    fn ui_countdown(&mut self, ui: &mut egui::Ui) {
        ui.ctx().request_repaint();

        let Some(start_time) = self.countdown_start else {
            self.screen = Screen::Ready;
            return;
        };

        let elapsed = start_time.elapsed().as_secs_f32();

        if elapsed >= 3.0 {
            self.countdown_start = None;
            self.start_game();
            return;
        }

        let count_num = 3 - elapsed.floor() as i32;

        ui.vertical_centered(|ui| {
            ui.add_space(110.0);

            ui.label(
                egui::RichText::new(format!(
                    "{}  /  {}",
                    self.selected_mode.label(),
                    self.selected_difficulty.label()
                ))
                .color(egui::Color32::from_gray(160))
                .size(20.0),
            );
            ui.add_space(30.0);

            ui.label(
                egui::RichText::new(format!("{count_num}"))
                    .color(egui::Color32::from_rgb(255, 215, 0))
                    .size(96.0),
            );
        });
    }

    fn ui_playing(&mut self, ui: &mut egui::Ui) {
        // [Esc] でプレイ中断
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.screen = Screen::Title;
            self.session = None;
            return;
        }

        // [Tab] または [F5] で即時リトライ
        if ui.input(|i| i.key_pressed(egui::Key::Tab) || i.key_pressed(egui::Key::F5)) {
            self.countdown_start = Some(Instant::now());
            self.screen = Screen::Countdown;
            return;
        }

        // タイムアタック時のタイマー更新・再描画
        if self.selected_mode == PlayMode::TimeAttack {
            ui.ctx().request_repaint();
        }

        // セッションの状態更新（時間切れチェック）
        if let Some(session) = self.session.as_mut() {
            session.update();
            if session.is_finished() {
                self.result_cursor = 0;
                self.screen = Screen::Result;
                return;
            }
        }

        ui.ctx().input(|i| {
            for event in &i.events {
                if let egui::Event::Text(text) = event {
                    for mut c in text.chars() {
                        if c == '\n' || c == '\r' || c == '\t' || c.is_ascii_control() {
                            continue;
                        }
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
                // ヘッダー情報（問題番号 or タイマー）
                ui.horizontal(|ui| {
                    match &session.mode {
                        GameMode::Normal { .. } => {
                            ui.label(format!(
                                "問題 ({}/{}) - {}",
                                session.current_question_index() + 1,
                                session.total_questions(),
                                self.selected_difficulty.label()
                            ));
                        }
                        GameMode::TimeAttack { .. } => {
                            let rem_secs =
                                session.remaining_time().unwrap_or_default().as_secs_f32();
                            let color = if rem_secs <= 10.0 {
                                egui::Color32::from_rgb(255, 90, 90)
                            } else {
                                egui::Color32::from_rgb(255, 220, 90)
                            };

                            ui.label(
                                egui::RichText::new(format!("残り時間: {:.1}秒", rem_secs))
                                    .color(color)
                                    .strong(),
                            );
                            ui.label(format!(
                                "　|　クリア: {}問",
                                session.current_question_index()
                            ));
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new("[Esc] 中断　[Tab] リトライ")
                                .color(egui::Color32::from_gray(120))
                                .size(14.0),
                        );
                    });
                });

                ui.add_space(20.0);

                let anchor_ratio = 0.35;
                if let Some(progress) = session.current_progress() {
                    // 1段目: 漢字
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

                    ui.add_space(4.0);

                    // 2段目: ふりがな
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

                    ui.add_space(4.0);

                    // 3段目: ローマ字
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
            }
        }
    }

    fn ui_result(&mut self, ui: &mut egui::Ui) {
        if ui.input(|i| i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown)) {
            self.result_cursor = (self.result_cursor + 1).min(1);
        }
        if ui.input(|i| i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp)) {
            self.result_cursor = self.result_cursor.saturating_sub(1);
        }
        if ui.input(|i| {
            i.key_pressed(egui::Key::L)
                || i.key_pressed(egui::Key::Space)
                || i.key_pressed(egui::Key::Enter)
        }) {
            if self.result_cursor == 0 {
                self.screen = Screen::Ready;
            } else {
                self.screen = Screen::Title;
                self.session = None;
            }
        }
        if ui.input(|i| {
            i.key_pressed(egui::Key::H)
                || i.key_pressed(egui::Key::Q)
                || i.key_pressed(egui::Key::Escape)
        }) {
            self.screen = Screen::Title;
            self.session = None;
        }

        if let Some(session) = self.session.as_ref()
            && let Some(result) = session.game_result()
        {
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);
                ui.heading("結果");
                ui.add_space(15.0);

                ui.label(format!(
                    "モード: {}  /  難易度: {}",
                    self.selected_mode.label(),
                    self.selected_difficulty.label()
                ));
                ui.add_space(10.0);

                if self.selected_mode == PlayMode::TimeAttack {
                    ui.label(
                        egui::RichText::new(format!(
                            "クリア問題数: {}問",
                            result.questions_completed
                        ))
                        .color(egui::Color32::from_rgb(255, 220, 90))
                        .size(30.0),
                    );
                }

                ui.label(format!(
                    "正確に入力した文字数: {}文字",
                    result.total_correct
                ));
                ui.label(format!("ミスタイプ数: {}回", result.total_incorrect));
                ui.label(format!("正確性: {:.1}%", result.accuracy * 100.0));
                ui.label(format!("タイプ数/分 (KPM): {:.1}", result.average_kpm));
                ui.label(format!(
                    "プレイ時間: {:.1}秒",
                    result.total_time.as_secs_f64()
                ));

                ui.add_space(25.0);
                let btn_size = egui::vec2(200.0, 46.0);

                let is_retry_selected = self.result_cursor == 0;
                let is_title_selected = self.result_cursor == 1;

                if ui
                    .add_sized(
                        btn_size,
                        egui::Button::selectable(is_retry_selected, "もう一度遊ぶ"),
                    )
                    .clicked()
                {
                    self.result_cursor = 0;
                    self.screen = Screen::Ready;
                }
                ui.add_space(8.0);
                if ui
                    .add_sized(
                        btn_size,
                        egui::Button::selectable(is_title_selected, "タイトルに戻る"),
                    )
                    .clicked()
                {
                    self.result_cursor = 1;
                    self.screen = Screen::Title;
                    self.session = None;
                }

                ui.add_space(25.0);
                ui.label(
                    egui::RichText::new(
                        "[↑ / ↓] 移動　　[Space / Enter] 決定　　[q / Esc] タイトルに戻る",
                    )
                    .color(egui::Color32::from_gray(130))
                    .size(15.0),
                );
            });
        }
    }
}

impl eframe::App for TypingGameApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        match self.screen {
            Screen::Title => self.ui_title(ui),
            Screen::ModeSelect => self.ui_mode_select(ui),
            Screen::DifficultySelect => self.ui_difficulty_select(ui),
            Screen::Ready => self.ui_ready(ui),
            Screen::Countdown => self.ui_countdown(ui),
            Screen::Playing => self.ui_playing(ui),
            Screen::Result => self.ui_result(ui),
        }
    }
}
