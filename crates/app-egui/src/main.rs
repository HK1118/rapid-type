#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod render;

use eframe::egui;
use game::{Difficulty, GameMode, Question, Session};
use rand::seq::SliceRandom;
use std::time::{Duration, Instant};

fn show_box(title: &str, message: &str) {
    #[cfg(windows)]
    unsafe {
        use std::ffi::CString;
        unsafe extern "system" {
            fn MessageBoxA(
                hwnd: *mut std::ffi::c_void,
                lpText: *const i8,
                lpCaption: *const i8,
                uType: u32,
            ) -> i32;
        }
        let t = CString::new(title).unwrap_or_default();
        let m = CString::new(message).unwrap_or_default();
        MessageBoxA(std::ptr::null_mut(), m.as_ptr(), t.as_ptr(), 0x10);
    }
    #[cfg(not(windows))]
    {
        eprintln!("{}: {}", title, message);
    }
}

fn main() {
    // 1. パニック発生時の検知
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("RapidType Panic:\n{}", info);
        let _ = std::fs::write("crash.log", &msg);
        show_box("RapidType Crash", &msg);
    }));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 750.0])
            .with_min_inner_size([800.0, 550.0]),
        ..Default::default()
    };

    // 2. 実行と初期化エラーの検知
    let result = eframe::run_native(
        "Rapid Type",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(TypingGameApp::new()))
        }),
    );

    // 3. run_native がエラーを返した場合にダイアログとログを出力
    if let Err(err) = result {
        let msg = format!("RapidType Launch Error:\n{:?}", err);
        let _ = std::fs::write("error.log", &msg);
        show_box("RapidType Launch Error", &msg);
    }
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

    // 画面スケーリングの安定化用
    last_window_height: f32,

    // 低スペックPC向け文字幅キャッシュ
    cached_question_id: Option<String>,
    cached_display_widths: Vec<f32>,
    cached_reading_widths: Vec<f32>,

    miss_flash_time: Option<Instant>,
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
            last_window_height: 0.0,
            cached_question_id: None,
            cached_display_widths: Vec::new(),
            cached_reading_widths: Vec::new(),
            miss_flash_time: None,
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
        self.cached_question_id = None; // キャッシュクリア
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
                    c, progress.completed_chars, progress.total_chars
                );
            }
            game::InputResult::Rejected { expected } => {
                self.miss_flash_time = Some(Instant::now());
                self.status_message = format!("Rejected: {} (expected: {})", c, expected);
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
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        if ui.input(|i| i.key_pressed(egui::Key::Q) || i.key_pressed(egui::Key::Escape)) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
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
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }

                ui.add_space(25.0);
                ui.label(
                    egui::RichText::new("[↑ / ↓] 移動　　[Space / Enter] 決定　　[q / Esc] 終了")
                        .color(egui::Color32::from_gray(165))
                        .size(15.0),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("vim風操作も可能")
                        .color(egui::Color32::from_gray(165))
                        .size(15.0),
                );
            });

            let actual_height = response.response.rect.height();
            ui.ctx()
                .data_mut(|d| d.insert_temp(content_id, actual_height));
        });
    }

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
                    .color(egui::Color32::from_gray(165))
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
                    .color(egui::Color32::from_gray(165))
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
                .color(egui::Color32::from_gray(210))
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
                    .color(egui::Color32::from_gray(165))
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
                .color(egui::Color32::from_gray(200))
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

        // テキスト入力の即時処理
        let mut text_received = false;
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
                        text_received = true;
                    }
                }
            }
        });

        // キー入力があったら即座に描画要求を出してラグを最小化
        if text_received {
            ui.ctx().request_repaint();
        }

        if let Some(session) = self.session.as_ref() {
            if let Some(question) = session.current_question() {
                // 問題が変わった時だけ文字幅を再計算（毎フレームのCPU負荷をゼロにする）
                if self.cached_question_id.as_deref() != Some(&question.id) {
                    self.cached_display_widths = render::compute_prefix_widths(
                        ui.ctx(),
                        &question.display,
                        egui::FontId::new(48.0, egui::FontFamily::Proportional),
                    );
                    self.cached_reading_widths = render::compute_prefix_widths(
                        ui.ctx(),
                        &question.reading,
                        egui::FontId::new(26.0, egui::FontFamily::Proportional),
                    );
                    self.cached_question_id = Some(question.id.clone());
                }

                // ヘッダー情報
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
                                .color(egui::Color32::from_gray(160))
                                .size(14.0),
                        );
                    });
                });

                ui.add_space(20.0);

                let anchor_ratio = 0.35;
                if let Some(progress) = session.current_progress() {
                    // 1段目: 漢字（キャッシュを利用）
                    let display_completed =
                        question.display_completed_chars(progress.completed_chars);
                    let (disp_done, disp_curr, disp_rem) =
                        render::anchored_progress_segments_cached(
                            &question.display,
                            display_completed,
                            ui.available_width(),
                            anchor_ratio,
                            48.0,
                            &self.cached_display_widths,
                        );
                    ui.add(
                        egui::Label::new(render::colored_progress_job(
                            &disp_done, &disp_curr, &disp_rem, 48.0, true,
                        ))
                        .extend(),
                    );

                    ui.add_space(4.0);

                    // 2段目: ふりがな（キャッシュを利用）
                    let (read_done, read_curr, read_rem) =
                        render::anchored_progress_segments_cached(
                            &question.reading,
                            progress.completed_chars,
                            ui.available_width(),
                            anchor_ratio,
                            26.0,
                            &self.cached_reading_widths,
                        );
                    ui.add(
                        egui::Label::new(render::colored_progress_job(
                            &read_done, &read_curr, &read_rem, 26.0, true,
                        ))
                        .extend(),
                    );

                    ui.add_space(4.0);

                    // 3段目: ローマ字（等幅フォント用の超高速幅計算を利用）
                    let full_guide = format!("{}{}", progress.typed_romaji, progress.guide);
                    let typed_count = progress.typed_romaji_count;
                    let guide_widths = render::compute_monospace_prefix_widths(
                        ui.ctx(),
                        &full_guide,
                        egui::FontId::new(36.0, egui::FontFamily::Monospace),
                    );

                    let (guide_done, guide_current, guide_remaining) =
                        render::anchored_progress_segments_cached(
                            &full_guide,
                            typed_count,
                            ui.available_width(),
                            anchor_ratio,
                            36.0,
                            &guide_widths,
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

            if let Some(flash_time) = self.miss_flash_time {
                let elapsed = flash_time.elapsed().as_secs_f32();
                let duration = 0.10; // フラッシュの長さ（約130ミリ秒）

                if elapsed < duration {
                    // 時間経過に合わせてアルファ値を 50 -> 0 に減衰（フェードアウト）
                    let progress = elapsed / duration;
                    let alpha = ((1.0 - progress) * 40.0) as u8;

                    // 画面最前面レイヤーに薄い赤を描画
                    let screen_rect = ui.ctx().viewport_rect();
                    let painter = ui.ctx().layer_painter(egui::LayerId::new(
                        egui::Order::Foreground,
                        egui::Id::new("miss_flash_layer"),
                    ));
                    painter.rect_filled(
                        screen_rect,
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(200, 35, 35, alpha),
                    );

                    // フェードアウトのアニメーションを滑らかに描画するためにフレーム更新を要求
                    ui.ctx().request_repaint();
                } else {
                    self.miss_flash_time = None;
                }
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
            let score = calculate_score(&result);
            let (rank, rank_color, comment) = evaluate_rank(score);

            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.heading("結果発表");
                ui.add_space(6.0);

                ui.label(
                    egui::RichText::new(format!(
                        "{}  /  {}",
                        self.selected_mode.label(),
                        self.selected_difficulty.label()
                    ))
                    .color(egui::Color32::from_gray(170))
                    .size(16.0),
                );
                ui.add_space(14.0);

                // ★ ランク表示（特大・ランクカラー）
                ui.label(
                    egui::RichText::new(format!("ランク: {rank}"))
                        .color(rank_color)
                        .size(54.0)
                        .strong(),
                );

                // ★ スコア表示
                ui.label(
                    egui::RichText::new(format!("スコア: {} 点", format_number(score)))
                        .color(egui::Color32::from_rgb(255, 230, 100))
                        .size(30.0)
                        .strong(),
                );
                ui.add_space(4.0);

                // ★ 一言評価コメント
                ui.label(
                    egui::RichText::new(comment)
                        .color(egui::Color32::from_gray(190))
                        .size(15.0),
                );

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(12.0);

                // 詳細統計
                if self.selected_mode == PlayMode::TimeAttack {
                    ui.label(
                        egui::RichText::new(format!(
                            "クリア問題数: {}問",
                            result.questions_completed
                        ))
                        .color(egui::Color32::from_rgb(255, 220, 90))
                        .size(20.0),
                    );
                }

                ui.label(format!("タイプ数/分 (KPM): {:.1}", result.average_kpm));
                ui.label(format!("正確性: {:.1}%", result.accuracy * 100.0));
                ui.label(format!(
                    "正解打鍵数: {}文字　/　ミス: {}回",
                    result.total_correct, result.total_incorrect
                ));
                ui.label(format!(
                    "プレイ時間: {:.1}秒",
                    result.total_time.as_secs_f64()
                ));

                ui.add_space(20.0);
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

                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new(
                        "[↑ / ↓] 移動　　[Space / Enter] 決定　　[q / Esc] タイトルに戻る",
                    )
                    .color(egui::Color32::from_gray(165))
                    .size(15.0),
                );
            });
        }
    }
}

impl eframe::App for TypingGameApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx();
        let current_zoom = ctx.zoom_factor();
        let unscaled_height = ctx.viewport_rect().height() * current_zoom;

        // ウィンドウの物理サイズが実際に変化した時だけ set_zoom_factor を呼ぶ
        // （毎フレーム呼ぶことによるテクスチャ再生成と入力遅延・残像・振動を防止）
        if (self.last_window_height - unscaled_height).abs() > 1.0 {
            self.last_window_height = unscaled_height;
            let target_scale = (unscaled_height / 780.0).clamp(0.8, 2.5);
            ctx.set_zoom_factor(target_scale);
        }

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

/// スコア計算: KPM × (正確率 ^ 3) × 10
fn calculate_score(result: &game::GameResult) -> u64 {
    if result.accuracy <= 0.0 || result.average_kpm <= 0.0 {
        return 0;
    }
    let raw = result.average_kpm * result.accuracy.powi(3) * 10.0;
    raw.round().max(0.0) as u64
}

/// ランク・表示色・コメントを判定
fn evaluate_rank(score: u64) -> (&'static str, egui::Color32, &'static str) {
    match score {
        s if s >= 4000 => (
            "SS",
            egui::Color32::from_rgb(255, 215, 0), // ゴールド
            "神業レベル！驚異的なタイピング速度です！",
        ),
        s if s >= 3200 => (
            "S",
            egui::Color32::from_rgb(255, 220, 90), // イエローゴールド
            "素晴らしい！プロフェッショナル級の腕前！",
        ),
        s if s >= 2400 => (
            "A",
            egui::Color32::from_rgb(255, 160, 50), // オレンジ
            "かなり速い！ブラインドタッチも完璧です！",
        ),
        s if s >= 1600 => (
            "B",
            egui::Color32::from_rgb(100, 220, 130), // グリーン
            "実用十分！業務や日常で困らないスピード！",
        ),
        s if s >= 900 => (
            "C",
            egui::Color32::from_rgb(100, 190, 255), // ライトブルー
            "順調に上達中！ミスを減らすとさらにスコアUP！",
        ),
        _ => (
            "D",
            egui::Color32::from_gray(180), // グレー
            "まずは正確に打つことを意識してみましょう！",
        ),
    }
}

/// 数字を3桁カンマ区切りにする (例: 2450 -> "2,450")
fn format_number(num: u64) -> String {
    let s = num.to_string();
    let len = s.len();
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(c);
    }
    result
}
