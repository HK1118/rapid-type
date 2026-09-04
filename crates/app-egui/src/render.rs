use eframe::egui;
use eframe::egui::text::LayoutJob;

/// 事前計算された累積幅配列を用いて、表示範囲（完了・現在・残り）を高速に切り出す
pub fn anchored_progress_segments_cached(
    text: &str,
    completed_chars: usize,
    max_width: f32,
    anchor_ratio: f32,
    font_size: f32,
    prefix_widths: &[f32],
) -> (String, String, String) {
    let chars: Vec<char> = text.chars().collect();
    let total_chars = chars.len();
    if total_chars == 0 {
        return (String::new(), String::new(), String::new());
    }

    let completed = completed_chars.min(total_chars);
    let current_idx = completed.min(total_chars.saturating_sub(1));

    let clamped_width = max_width.max(font_size * 4.0);
    let target_left = (prefix_widths.get(current_idx).copied().unwrap_or(0.0)
        - clamped_width * anchor_ratio)
        .max(0.0);

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

/// プロポーショナルフォント用：テキストの累積幅リストを計算（問題切替時のみ1度実行）
pub fn compute_prefix_widths(ctx: &egui::Context, text: &str, font_id: egui::FontId) -> Vec<f32> {
    let total_chars = text.chars().count();
    let galley = ctx.fonts_mut(|fonts| {
        fonts.layout_no_wrap(text.to_string(), font_id.clone(), egui::Color32::WHITE)
    });

    let mut prefix_widths = Vec::with_capacity(total_chars + 1);
    prefix_widths.push(0.0f32);

    if let Some(row) = galley.rows.first() {
        for glyph in &row.glyphs {
            prefix_widths.push(glyph.max_x());
        }
    }

    while prefix_widths.len() <= total_chars {
        let last = prefix_widths.last().copied().unwrap_or(0.0);
        prefix_widths.push(last + font_id.size * 0.5);
    }

    prefix_widths
}

/// 等幅フォント用：1文字幅から掛け算で累積幅を即座に生成（レイアウト処理なしで超高速）
pub fn compute_monospace_prefix_widths(
    ctx: &egui::Context,
    text: &str,
    font_id: egui::FontId,
) -> Vec<f32> {
    let char_width = ctx.fonts_mut(|fonts| fonts.glyph_width(&font_id, 'm'));
    let char_width = if char_width > 0.0 {
        char_width
    } else {
        font_id.size * 0.6
    };

    let total_chars = text.chars().count();
    let mut prefix_widths = Vec::with_capacity(total_chars + 1);
    for i in 0..=total_chars {
        prefix_widths.push(i as f32 * char_width);
    }
    prefix_widths
}

/// 色付けテキストレイアウトを生成（視認性向上トーン）
pub fn colored_progress_job(
    done: &str,
    current: &str,
    remaining: &str,
    size: f32,
    is_japanese: bool,
) -> LayoutJob {
    let mut job = LayoutJob::default();

    // 背景と同化しにくいように全体的にトーンアップ
    let done_color = egui::Color32::from_gray(110);
    let current_color = egui::Color32::from_rgb(255, 230, 100);
    let remaining_color = egui::Color32::from_gray(225);

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
