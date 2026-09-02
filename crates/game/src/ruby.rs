use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RubySegment {
    pub display: String,
    pub reading: String,
    pub is_ruby: bool,
}

impl RubySegment {
    pub fn new(display: impl Into<String>, reading: impl Into<String>, is_ruby: bool) -> Self {
        Self {
            display: display.into(),
            reading: reading.into(),
            is_ruby,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RubyParseError {
    #[error("閉じ括弧 ']' が見つかりません: {0}")]
    UnclosedBracket(String),

    #[error("ルビの区切り文字 '|' がありません: [{0}]")]
    MissingDelimiter(String),

    #[error("表示用テキストまたは読みが空です: [{0}]")]
    EmptyContent(String),

    #[error("読み（reading）に漢字や無効な文字が含まれています: '{0}' (文: \"{1}\")")]
    InvalidReadingChar(char, String),
}

/// カタカナ文字をひらがなに変換（ひらがなや記号はそのまま）
fn katakana_to_hiragana(c: char) -> char {
    match c {
        // カタカナ 'ァ' (U+30A1) 〜 'ヶ' (U+30F6) をひらがな (U+3041 〜) に変換
        '\u{30A1}'..='\u{30F6}' => char::from_u32(c as u32 - 0x60).unwrap_or(c),
        _ => c,
    }
}

/// 文字列全体のカタカナをひらがなに変換
fn to_hiragana_string(text: &str) -> String {
    text.chars().map(katakana_to_hiragana).collect()
}

pub fn parse_ruby(input: &str) -> Result<Vec<RubySegment>, RubyParseError> {
    let mut segments = Vec::new();
    let mut chars = input.chars().peekable();
    let mut plain_buf = String::new();

    while let Some(c) = chars.next() {
        if c == '[' {
            if !plain_buf.is_empty() {
                validate_plain_text(&plain_buf, input)?;
                // 平文に含まれるカタカナを自動でひらがな読みに変換してセグメント化
                let reading = to_hiragana_string(&plain_buf);
                segments.push(RubySegment::new(plain_buf.clone(), reading, false));
                plain_buf.clear();
            }

            let mut inner = String::new();
            let mut closed = false;
            for ic in chars.by_ref() {
                if ic == ']' {
                    closed = true;
                    break;
                }
                inner.push(ic);
            }

            if !closed {
                return Err(RubyParseError::UnclosedBracket(input.to_string()));
            }

            let (display, reading) = inner
                .split_once('|')
                .ok_or_else(|| RubyParseError::MissingDelimiter(inner.clone()))?;

            if display.is_empty() || reading.is_empty() {
                return Err(RubyParseError::EmptyContent(inner));
            }

            // ルビの読み側（reading）もカタカナがあればひらがなに統一
            let reading_hira = to_hiragana_string(reading);
            validate_reading_text(&reading_hira, input)?;

            segments.push(RubySegment::new(display, reading_hira, true));
        } else {
            plain_buf.push(c);
        }
    }

    if !plain_buf.is_empty() {
        validate_plain_text(&plain_buf, input)?;
        let reading = to_hiragana_string(&plain_buf);
        segments.push(RubySegment::new(plain_buf.clone(), reading, false));
    }

    Ok(segments)
}

/// 読み（ひらがな）として許容される文字かチェック
fn is_valid_reading_char(c: char) -> bool {
    matches!(c,
        '\u{3040}'..='\u{309F}' | // ひらがな
        '\u{30A0}'..='\u{30FF}' | // カタカナ
        '\u{0020}'..='\u{007E}' | // 半角英数・記号（スペース含む）
        '、' | '。' | '！' | '？'  | '〜' | '～' | '「' | '」'  | '　'
    )
}

/// ルビで囲まれていない平文のチェック（漢字のルビ抜けを検出）
fn validate_plain_text(plain: &str, full_input: &str) -> Result<(), RubyParseError> {
    for c in plain.chars() {
        if !is_valid_reading_char(c) {
            // 平文に漢字が含まれている = ルビの付け忘れ
            return Err(RubyParseError::InvalidReadingChar(
                c,
                full_input.to_string(),
            ));
        }
    }
    Ok(())
}

/// 読み（reading）部分の文字チェック
fn validate_reading_text(reading: &str, full_input: &str) -> Result<(), RubyParseError> {
    for c in reading.chars() {
        if !is_valid_reading_char(c) {
            return Err(RubyParseError::InvalidReadingChar(
                c,
                full_input.to_string(),
            ));
        }
    }
    Ok(())
}

/// 読みの入力完了文字数から、表示側（漢字側）の完了文字数を計算する
pub fn calculate_display_progress(segments: &[RubySegment], reading_completed: usize) -> usize {
    let mut rem_reading = reading_completed;
    let mut display_completed = 0;

    for seg in segments {
        let seg_reading_len = seg.reading.chars().count();
        let seg_display_len = seg.display.chars().count();

        if rem_reading >= seg_reading_len {
            // このセグメントを完全に打ち終えた
            rem_reading -= seg_reading_len;
            display_completed += seg_display_len;
        } else {
            // ひらがな、カタカナの場合のみ、1文字打つごとに表示側も1文字進める
            if !seg.is_ruby && seg_display_len == seg_reading_len {
                // 1文字打つごとにカタカナ/ひらがなも1文字リアルタイムに進める
                display_completed += rem_reading;
            }
            // 漢字ルビなど文字数が一致しない場合は完了まで保留
            break;
        }
    }

    display_completed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_normal_ruby() {
        let input = "[青|あお][空|ぞら]を[見|み][上|あ]げる";
        let segments = parse_ruby(input).unwrap();

        assert_eq!(
            segments,
            vec![
                RubySegment::new("青", "あお", true),
                RubySegment::new("空", "ぞら", true),
                RubySegment::new("を", "を", false),
                RubySegment::new("見", "み", true),
                RubySegment::new("上", "あ", true),
                RubySegment::new("げる", "げる", false),
            ]
        );
    }

    #[test]
    fn test_parse_plain_hiragana_and_katakana() {
        let input = "おにぎりとちょこれーと";
        let segments = parse_ruby(input).unwrap();
        assert_eq!(segments, vec![RubySegment::new(input, input, false)]);
    }

    #[test]
    fn test_parse_jukujikun() {
        // 熟字訓（分割できない複数文字の漢字）
        let input = "[今日|きょう]は[晴|は]れ";
        let segments = parse_ruby(input).unwrap();
        assert_eq!(
            segments,
            vec![
                RubySegment::new("今日", "きょう", true),
                RubySegment::new("は", "は", false),
                RubySegment::new("晴", "は", true),
                RubySegment::new("れ", "れ", false),
            ]
        );
    }

    #[test]
    fn test_error_missing_ruby_on_kanji() {
        // 「見上げる」の「上」にルビが抜けている場合のエラー検知
        let input = "[青|あお][空|ぞら]を[見|み]上げる";
        let result = parse_ruby(input);
        assert!(matches!(
            result,
            Err(RubyParseError::InvalidReadingChar('上', _))
        ));
    }

    #[test]
    fn test_error_unclosed_bracket() {
        let input = "[青|あお";
        let result = parse_ruby(input);
        assert!(matches!(result, Err(RubyParseError::UnclosedBracket(_))));
    }

    #[test]
    fn test_error_missing_delimiter() {
        let input = "[青あお]";
        let result = parse_ruby(input);
        assert!(matches!(result, Err(RubyParseError::MissingDelimiter(_))));
    }

    #[test]
    fn test_calculate_display_progress() {
        let segments = parse_ruby("[青|あお][空|ぞら]を[見|み][上|あ]げる").unwrap();

        assert_eq!(calculate_display_progress(&segments, 0), 0); // "" -> 0文字
        assert_eq!(calculate_display_progress(&segments, 1), 0); // "あ" -> 「青」保留 (0文字)
        assert_eq!(calculate_display_progress(&segments, 2), 1); // "あお" -> 「青」完了 (1文字)
        assert_eq!(calculate_display_progress(&segments, 3), 1); // "あおぞ" -> 「空」保留 (1文字)
        assert_eq!(calculate_display_progress(&segments, 4), 2); // "あおぞら" -> 「青空」完了 (2文字)
        assert_eq!(calculate_display_progress(&segments, 5), 3); // "あおぞらを" -> 「青空を」完了 (3文字)
        assert_eq!(calculate_display_progress(&segments, 6), 4); // "あおぞらをみ" -> 「青空を見」完了 (4文字)
        assert_eq!(calculate_display_progress(&segments, 7), 5); // "あおぞらをみあ" -> 「青空を見上」完了 (5文字)
        assert_eq!(calculate_display_progress(&segments, 8), 6); // "あおぞらをみあげ" -> 「青空を見上げ」完了 (6文字: 送り仮名「げ」が進む)
        assert_eq!(calculate_display_progress(&segments, 9), 7); // "あおぞらをみあげる" -> 全完了 (7文字)
    }

    #[test]
    fn test_plain_hiragana_progress() {
        let segments = parse_ruby("おにぎり").unwrap();

        // 1文字打つごとに1文字ずつ進む
        assert_eq!(calculate_display_progress(&segments, 0), 0); // "" -> 0文字
        assert_eq!(calculate_display_progress(&segments, 1), 1); // "お" -> 1文字
        assert_eq!(calculate_display_progress(&segments, 2), 2); // "おに" -> 2文字
        assert_eq!(calculate_display_progress(&segments, 3), 3); // "おにぎ" -> 3文字
        assert_eq!(calculate_display_progress(&segments, 4), 4); // "おにぎり" -> 4文字
    }

    #[test]
    fn test_jukujikun_progress_does_not_advance_partially() {
        // 漢字2文字・読み2文字の熟字訓
        let segments = parse_ruby("[風邪|かぜ][薬|ぐすり]").unwrap();

        // 0文字入力 ("") -> 0文字確定
        assert_eq!(calculate_display_progress(&segments, 0), 0);
        // 1文字入力 ("か") -> 「風邪」は保留されるので 0文字確定
        assert_eq!(calculate_display_progress(&segments, 1), 0);
        // 2文字入力 ("かぜ") -> 「風邪」が確定して 2文字確定
        assert_eq!(calculate_display_progress(&segments, 2), 2);
        // 3文字入力 ("かぜぐ") -> 「薬」は保留されるので 2文字確定
        assert_eq!(calculate_display_progress(&segments, 3), 2);
        // 5文字入力 ("かぜぐすり") -> 全完了で 3文字確定 ("風邪薬")
        assert_eq!(calculate_display_progress(&segments, 5), 3);
    }

    #[test]
    fn test_plain_katakana_advances_one_by_one() {
        // 平文のカタカナは1文字ずつ進む
        let segments = parse_ruby("アイス").unwrap();

        assert_eq!(calculate_display_progress(&segments, 0), 0);
        assert_eq!(calculate_display_progress(&segments, 1), 1); // "ア"
        assert_eq!(calculate_display_progress(&segments, 2), 2); // "アイ"
        assert_eq!(calculate_display_progress(&segments, 3), 3); // "アイス"
    }

    #[test]
    fn test_okurigana_multiple_chars_progress() {
        // 送り仮名「げる」が2文字ある場合
        let segments = parse_ruby("[見|み][上|あ]げる").unwrap();

        // "みあ" (2文字) -> "見上" (2文字完了)
        assert_eq!(calculate_display_progress(&segments, 2), 2);
        // "みあげ" (3文字) -> "見上げ" (3文字完了: 送り仮名「げ」が進む)
        assert_eq!(calculate_display_progress(&segments, 3), 3);
        // "みあげる" (4文字) -> "見上げる" (4文字完了)
        assert_eq!(calculate_display_progress(&segments, 4), 4);
    }

    #[test]
    fn test_katakana_auto_conversion() {
        let segments = parse_ruby("チョコレート").unwrap();
        assert_eq!(segments[0].display, "チョコレート");
        assert_eq!(segments[0].reading, "ちょこれーと");

        // 1文字ずつ進行する
        assert_eq!(calculate_display_progress(&segments, 0), 0);
        assert_eq!(calculate_display_progress(&segments, 1), 1); // "ち" -> "チ"
        assert_eq!(calculate_display_progress(&segments, 2), 2); // "ちょ" -> "チョ"
        assert_eq!(calculate_display_progress(&segments, 6), 6); // 全完了
    }

    #[test]
    fn test_mixed_kanji_katakana() {
        let segments = parse_ruby("[秋|あき]のチョコレートケーキ").unwrap();
        assert_eq!(
            segments,
            vec![
                RubySegment::new("秋", "あき", true),
                RubySegment::new("のチョコレートケーキ", "のちょこれーとけーき", false),
            ]
        );
    }
}
