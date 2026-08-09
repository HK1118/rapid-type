use std::collections::{HashMap, HashSet, VecDeque};

use crate::result::EngineInputResult;
use crate::romaji_table::{ROMAJI_TABLE, RomajiOption, SYMBOL_TABLE};

const SMALL_KANAS: [char; 9] = ['ぁ', 'ぃ', 'ぅ', 'ぇ', 'ぉ', 'ゃ', 'ゅ', 'ょ', 'ゎ'];
const SOKUON: &str = "っ";
const HATSUON: &str = "ん";
const COMPOSED_OPTION_PRIORITY_BIAS: u8 = 32;
const MAX_COMBINATIONS: usize = 32;

#[derive(Debug, Clone)]
struct BuildOption {
    romaji: String,
    priority: u8,
    progress_markers: Vec<(usize, usize)>,
}

impl BuildOption {
    fn from_romaji_option(option: RomajiOption) -> Self {
        Self {
            romaji: option.romaji,
            priority: option.priority,
            progress_markers: Vec::new(),
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Node {
    transitions: Vec<(char, usize)>,
    is_terminal: bool,
}

#[derive(Debug, Clone)]
pub struct TypingEngine {
    nodes: Vec<Node>,
    node_completed_chars: Vec<usize>,
    current_states: HashSet<usize>,
    prev_states: HashSet<usize>,
    cached_guide: String,
    reading_text: String,
}

impl TypingEngine {
    pub fn new(input: &str) -> Result<Self, String> {
        let mut nodes = vec![Node::default()];
        let mut node_completed_chars = vec![0usize];
        let mut parent = 0usize;
        let tokens = tokenize(input);

        let mut i = 0;
        while i < tokens.len() {
            let token = tokens[i].as_str();
            let (options, consumed_tokens) = if token == SOKUON {
                let sokuon_count = count_consecutive_token(&tokens, i, SOKUON);
                let next_token = tokens.get(i + sokuon_count).map(String::as_str);
                let has_following_token = next_token.is_some();
                let options =
                    sokuon_options_with_markers(sokuon_count, next_token).ok_or_else(|| {
                        format!("\"{}\" cannot be expanded without a following kana", SOKUON)
                    })?;
                (options, sokuon_count + usize::from(has_following_token))
            } else {
                (
                    resolve_build_options(token, tokens.get(i + 1).map(String::as_str))?,
                    1,
                )
            };

            let consumed_chars = tokens[i..i + consumed_tokens]
                .iter()
                .map(|t| t.chars().count())
                .sum::<usize>();

            let merge_node = nodes.len();
            nodes.push(Node::default());
            node_completed_chars.push(node_completed_chars[parent] + consumed_chars);

            for opt in options {
                let chars: Vec<char> = opt.romaji.chars().collect();
                if chars.is_empty() {
                    continue;
                }

                let mut progress_markers = opt.progress_markers;
                progress_markers.sort_unstable_by_key(|(input_chars, _)| *input_chars);

                let mut curr = parent;
                for (j, &c) in chars.iter().enumerate() {
                    let is_last = j == chars.len() - 1;

                    if is_last {
                        let has_exact_transition = nodes[curr]
                            .transitions
                            .iter()
                            .any(|&(ch, next)| ch == c && next == merge_node);
                        if !has_exact_transition {
                            nodes[curr].transitions.push((c, merge_node));
                        }
                    } else {
                        let mut next_idx_opt = None;
                        for &(ch, next) in &nodes[curr].transitions {
                            if ch == c && next != merge_node {
                                next_idx_opt = Some(next);
                                break;
                            }
                        }

                        curr = if let Some(next_idx) = next_idx_opt {
                            next_idx
                        } else {
                            let new_idx = nodes.len();
                            nodes.push(Node::default());
                            node_completed_chars.push(node_completed_chars[curr]);
                            nodes[curr].transitions.push((c, new_idx));
                            new_idx
                        };

                        let consumed_input_chars = j + 1;
                        for &(marker_input_chars, marker_completed_chars) in &progress_markers {
                            if marker_input_chars == consumed_input_chars {
                                node_completed_chars[curr] = node_completed_chars[curr]
                                    .max(node_completed_chars[parent] + marker_completed_chars);
                            }
                        }
                    }
                }
            }

            parent = merge_node;
            i += consumed_tokens;
        }

        nodes[parent].is_terminal = true;

        let mut engine = Self {
            nodes,
            node_completed_chars,
            current_states: [0].into_iter().collect(),
            prev_states: HashSet::new(),
            cached_guide: String::new(),
            reading_text: input.to_string(),
        };

        engine.cached_guide = engine.compute_guide();
        engine.prev_states = engine.current_states.clone();

        Ok(engine)
    }

    pub fn input(&mut self, c: char) -> EngineInputResult {
        let currently_completed = self
            .current_states
            .iter()
            .any(|&idx| self.nodes[idx].is_terminal);
        if currently_completed {
            return EngineInputResult::AlreadyCompleted;
        }

        let mut next_states = HashSet::new();
        for &s_idx in &self.current_states {
            for &(ch, next_idx) in &self.nodes[s_idx].transitions {
                if ch == c {
                    next_states.insert(next_idx);
                }
            }
        }

        if next_states.is_empty() {
            EngineInputResult::Rejected
        } else {
            self.current_states = next_states;
            if self.current_states != self.prev_states {
                self.cached_guide = self.compute_guide();
                self.prev_states = self.current_states.clone();
            }

            let is_now_completed = self
                .current_states
                .iter()
                .any(|&idx| self.nodes[idx].is_terminal);
            if is_now_completed {
                EngineInputResult::Completed
            } else {
                EngineInputResult::Accepted
            }
        }
    }

    pub fn guide(&self) -> &str {
        &self.cached_guide
    }

    pub fn completed_char_count(&self) -> usize {
        self.completed_char_range().0
    }

    pub fn furthest_completed_char_count(&self) -> usize {
        self.completed_char_range().1
    }

    pub fn completed_char_range(&self) -> (usize, usize) {
        let mut min = usize::MAX;
        let mut max = 0usize;

        for &state in &self.current_states {
            let completed = self.node_completed_chars[state];
            min = min.min(completed);
            max = max.max(completed);
        }

        if min == usize::MAX {
            (0, 0)
        } else {
            (min, max)
        }
    }

    pub fn completed_reading(&self) -> &str {
        let completed = self.completed_char_count();
        let end = byte_index_at_char_count(&self.reading_text, completed);
        &self.reading_text[..end]
    }

    pub fn furthest_completed_reading(&self) -> &str {
        let completed = self.furthest_completed_char_count();
        let end = byte_index_at_char_count(&self.reading_text, completed);
        &self.reading_text[..end]
    }

    fn compute_guide(&self) -> String {
        let mut starts: Vec<usize> = self.current_states.iter().copied().collect();
        starts.sort_unstable();

        let mut queue = VecDeque::new();
        let mut visited = vec![false; self.nodes.len()];
        let mut parents: Vec<Option<(usize, char)>> = vec![None; self.nodes.len()];

        for start in starts {
            if !visited[start] {
                visited[start] = true;
                queue.push_back(start);
            }
        }

        let mut goal = None;
        while let Some(node_idx) = queue.pop_front() {
            if self.nodes[node_idx].is_terminal {
                goal = Some(node_idx);
                break;
            }

            for &(c, next_idx) in &self.nodes[node_idx].transitions {
                if !visited[next_idx] {
                    visited[next_idx] = true;
                    parents[next_idx] = Some((node_idx, c));
                    queue.push_back(next_idx);
                }
            }
        }

        let Some(mut node_idx) = goal else {
            return String::new();
        };

        let mut guide_chars = Vec::new();
        while let Some((prev_idx, c)) = parents[node_idx] {
            guide_chars.push(c);
            node_idx = prev_idx;
        }
        guide_chars.reverse();

        guide_chars.into_iter().collect()
    }
}

fn tokenize(input: &str) -> Vec<String> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        if i + 1 < chars.len() {
            let two_chars: String = chars[i..=i + 1].iter().collect();
            if ROMAJI_TABLE.contains_key(two_chars.as_str()) {
                tokens.push(two_chars);
                i += 2;
                continue;
            }
        }

        tokens.push(chars[i].to_string());
        i += 1;
    }

    tokens
}

fn count_consecutive_token(tokens: &[String], start_idx: usize, token: &str) -> usize {
    let mut count = 0;
    while tokens.get(start_idx + count).is_some_and(|t| t == token) {
        count += 1;
    }
    count
}

fn resolve_options(token: &str, next_token: Option<&str>) -> Result<Vec<RomajiOption>, String> {
    if token == HATSUON {
        return Ok(hatsuon_options(next_token));
    }
    if let Some(symbols) = symbol_options(token) {
        return Ok(symbols);
    }

    kana_options(token).ok_or_else(|| format!("\"{}\" is not in ROMAJI_TABLE/SYMBOL_TABLE", token))
}

fn resolve_build_options(
    token: &str,
    next_token: Option<&str>,
) -> Result<Vec<BuildOption>, String> {
    let options = resolve_options(token, next_token)?;
    let progress_markers = small_kana_progress_markers(token);

    Ok(options
        .into_iter()
        .map(|option| {
            let mut build_option = BuildOption::from_romaji_option(option);
            if let Some(&(marker_input_chars, marker_completed_chars)) =
                progress_markers.get(&build_option.romaji)
            {
                build_option
                    .progress_markers
                    .push((marker_input_chars, marker_completed_chars));
            }
            build_option
        })
        .collect())
}

fn kana_options(token: &str) -> Option<Vec<RomajiOption>> {
    let mut options = Vec::new();
    if let Some(table_options) = ROMAJI_TABLE.get(token) {
        options.extend(table_options.clone());
    }
    if let Some(mut composed_options) = small_kana_composed_options(token) {
        for opt in &mut composed_options {
            opt.priority = opt.priority.saturating_add(COMPOSED_OPTION_PRIORITY_BIAS);
        }
        options.extend(composed_options);
    }
    if options.is_empty() {
        return None;
    }

    options.sort_by_key(|opt| opt.priority);
    let mut deduped = Vec::new();
    let mut seen = HashSet::new();
    for option in options {
        if seen.insert(option.romaji.clone()) {
            deduped.push(option);
        }
    }
    Some(deduped)
}

fn symbol_options(token: &str) -> Option<Vec<RomajiOption>> {
    SYMBOL_TABLE.get(token).cloned()
}

fn sokuon_options_with_markers(
    sokuon_count: usize,
    next_token: Option<&str>,
) -> Option<Vec<BuildOption>> {
    if sokuon_count == 0 {
        return None;
    }

    let Some(next_token) = next_token else {
        let mut options = expand_sokuon_prefixes(&sokuon_prefix_options(""), sokuon_count);
        options.sort_by_key(|opt| opt.priority);
        return Some(
            options
                .into_iter()
                .map(BuildOption::from_romaji_option)
                .collect(),
        );
    };
    let mut next_options = kana_options(next_token)?;
    next_options.sort_by_key(|opt| opt.priority);

    let mut options: Vec<BuildOption> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let sokuon_completed_chars = sokuon_count;
    for next_opt in &next_options {
        let prefix_options = sokuon_prefix_options(next_opt.romaji.as_str());
        let prefix_combinations = expand_sokuon_prefixes(&prefix_options, sokuon_count);

        for prefix in prefix_combinations {
            let mut romaji = String::with_capacity(prefix.romaji.len() + next_opt.romaji.len());
            romaji.push_str(&prefix.romaji);
            romaji.push_str(&next_opt.romaji);
            if seen.insert(romaji.clone()) {
                options.push(BuildOption {
                    romaji,
                    priority: next_opt.priority.saturating_add(prefix.priority),
                    progress_markers: vec![(prefix.romaji.chars().count(), sokuon_completed_chars)],
                });
            }
        }
    }

    if options.is_empty() {
        None
    } else {
        options.sort_by_key(|opt| opt.priority);
        Some(options)
    }
}

fn is_sokuon_repeatable(c: char) -> bool {
    c.is_ascii_lowercase() && !matches!(c, 'a' | 'i' | 'u' | 'e' | 'o' | 'n')
}

fn sokuon_prefix_options(next_romaji: &str) -> Vec<RomajiOption> {
    let mut options = Vec::new();
    let mut seen = HashSet::new();

    if let Some(head) = next_romaji.chars().next()
        && is_sokuon_repeatable(head)
    {
        push_unique_option(
            &mut options,
            &mut seen,
            RomajiOption::from_string(head.to_string(), 0),
        );
    }

    if next_romaji.starts_with("ch") {
        push_unique_option(&mut options, &mut seen, RomajiOption::new("t", 1));
    }

    for (small_tsu, priority) in [("ltu", 10u8), ("xtu", 11), ("ltsu", 12), ("xtsu", 13)] {
        push_unique_option(
            &mut options,
            &mut seen,
            RomajiOption::new(small_tsu, priority),
        );
    }

    options.sort_by_key(|opt| opt.priority);
    options
}

fn expand_sokuon_prefixes(
    prefix_options: &[RomajiOption],
    sokuon_count: usize,
) -> Vec<RomajiOption> {
    let mut combinations = vec![RomajiOption::from_string(String::new(), 0)];

    for _ in 0..sokuon_count {
        let mut next = Vec::new();
        for base in &combinations {
            for prefix in prefix_options {
                let mut romaji = String::with_capacity(base.romaji.len() + prefix.romaji.len());
                romaji.push_str(&base.romaji);
                romaji.push_str(&prefix.romaji);
                next.push(RomajiOption::from_string(
                    romaji,
                    base.priority.saturating_add(prefix.priority),
                ));
            }
        }

        if next.len() > MAX_COMBINATIONS {
            next.sort_by_key(|opt| opt.priority);
            next.truncate(MAX_COMBINATIONS);
        }

        combinations = next;
    }

    combinations
}

fn hatsuon_options(next_token: Option<&str>) -> Vec<RomajiOption> {
    let mut options = Vec::new();
    if next_token.is_some_and(allows_single_n_before_token) {
        options.push(RomajiOption::new("n", 0));
    }
    options.push(RomajiOption::new("nn", 1));
    options.push(RomajiOption::new("xn", 2));
    options.push(RomajiOption::new("n'", 3));
    options
}

fn allows_single_n_before_token(next_token: &str) -> bool {
    if symbol_options(next_token).is_some() {
        return true;
    }

    let Some(options) = kana_options(next_token) else {
        return true;
    };

    !options.iter().any(|opt| {
        matches!(
            opt.romaji.chars().next(),
            Some('a' | 'i' | 'u' | 'e' | 'o' | 'y' | 'n')
        )
    })
}

fn small_kana_composed_options(token: &str) -> Option<Vec<RomajiOption>> {
    let (base, small) = split_small_kana_token(token)?;
    let mut base_options = ROMAJI_TABLE.get(base)?.clone();
    base_options.sort_by_key(|opt| opt.priority);
    let mut small_options = ROMAJI_TABLE.get(small)?.clone();
    small_options.sort_by_key(|opt| opt.priority);

    let mut options = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for base_opt in &base_options {
        for small_opt in &small_options {
            if !small_opt.romaji.starts_with('x') && !small_opt.romaji.starts_with('l') {
                continue;
            }
            let mut romaji = String::with_capacity(base_opt.romaji.len() + small_opt.romaji.len());
            romaji.push_str(&base_opt.romaji);
            romaji.push_str(&small_opt.romaji);
            if seen.insert(romaji.clone()) {
                options.push(RomajiOption::from_string(
                    romaji,
                    base_opt.priority.saturating_add(small_opt.priority),
                ));
            }
        }
    }

    if options.is_empty() {
        None
    } else {
        Some(options)
    }
}

fn small_kana_progress_markers(token: &str) -> HashMap<String, (usize, usize)> {
    let Some((base, small)) = split_small_kana_token(token) else {
        return HashMap::new();
    };

    let Some(base_options) = ROMAJI_TABLE.get(base) else {
        return HashMap::new();
    };
    let Some(small_options) = ROMAJI_TABLE.get(small) else {
        return HashMap::new();
    };

    let mut markers = HashMap::new();
    let base_completed_chars = base.chars().count();
    for base_opt in base_options {
        for small_opt in small_options {
            if !small_opt.romaji.starts_with('x') && !small_opt.romaji.starts_with('l') {
                continue;
            }

            let mut romaji = String::with_capacity(base_opt.romaji.len() + small_opt.romaji.len());
            romaji.push_str(&base_opt.romaji);
            romaji.push_str(&small_opt.romaji);
            markers.insert(
                romaji,
                (base_opt.romaji.chars().count(), base_completed_chars),
            );
        }
    }

    markers
}

fn split_small_kana_token(token: &str) -> Option<(&str, &str)> {
    let (idx, c) = token.char_indices().last()?;
    if idx == 0 || !SMALL_KANAS.contains(&c) {
        return None;
    }
    Some((&token[..idx], &token[idx..]))
}

fn push_unique_option(
    options: &mut Vec<RomajiOption>,
    seen: &mut HashSet<String>,
    option: RomajiOption,
) {
    if seen.insert(option.romaji.clone()) {
        options.push(option);
    }
}

fn byte_index_at_char_count(text: &str, char_count: usize) -> usize {
    if char_count == 0 {
        return 0;
    }
    text.char_indices()
        .nth(char_count)
        .map(|(idx, _)| idx)
        .unwrap_or(text.len())
}

#[cfg(test)]
mod tests {
    use super::TypingEngine;
    use crate::result::EngineInputResult;

    use std::time::Instant;

    #[test]
    fn typing_engine_new_performance() {
        let start = Instant::now();
        let _engine = TypingEngine::new(
            &"じゅげむじゅげむごこうのすりきれかいじゃりすいぎょのすいぎょうまつうんらいまつふうらいまつくうねるところにすむところやぶらこうじのぶらこうじぱいぽぱいぽぱいぽのしゅーりんがんしゅーりんがんのぐーりんだいぐーりんだいのぽんぽこぴーのぽんぽこなーのちょうきゅうめいのちょうすけ".repeat(10),
        )
        .unwrap();
        let duration = start.elapsed();
        println!("TypingEngine::new took {:?} for a long input", duration);
        assert!(duration.as_secs() < 1, "TypingEngine::new is too slow");
    }

    #[test]
    fn allows_single_n_for_non_terminal_hatsuon() {
        let mut engine = TypingEngine::new("かんか").unwrap();
        for c in ['k', 'a', 'n', 'k'] {
            assert!(matches!(engine.input(c), EngineInputResult::Accepted));
        }
        assert!(matches!(engine.input('a'), EngineInputResult::Completed));
    }

    #[test]
    fn disallows_single_n_for_terminal_hatsuon() {
        let mut engine = TypingEngine::new("かん").unwrap();
        for c in ['k', 'a'] {
            assert!(matches!(engine.input(c), EngineInputResult::Accepted));
        }
        assert!(matches!(engine.input('n'), EngineInputResult::Accepted));
        assert!(matches!(engine.input('n'), EngineInputResult::Completed));
    }

    #[test]
    fn allows_other_common_hatsuon_inputs() {
        let mut engine_nn = TypingEngine::new("かん").unwrap();
        for c in ['k', 'a', 'n'] {
            assert!(matches!(engine_nn.input(c), EngineInputResult::Accepted));
        }
        assert!(matches!(engine_nn.input('n'), EngineInputResult::Completed));

        let mut engine_xn = TypingEngine::new("かん").unwrap();
        for c in ['k', 'a', 'x'] {
            assert!(matches!(engine_xn.input(c), EngineInputResult::Accepted));
        }
        assert!(matches!(engine_xn.input('n'), EngineInputResult::Completed));

        let mut engine_n_apostrophe = TypingEngine::new("かん").unwrap();
        for c in ['k', 'a', 'n'] {
            assert!(matches!(
                engine_n_apostrophe.input(c),
                EngineInputResult::Accepted
            ));
        }
        assert!(matches!(
            engine_n_apostrophe.input('\''),
            EngineInputResult::Completed
        ));
    }

    #[test]
    fn allows_double_consonant_for_sokuon() {
        let mut engine = TypingEngine::new("かっか").unwrap();
        for c in ['k', 'a', 'k', 'k'] {
            assert!(matches!(engine.input(c), EngineInputResult::Accepted));
        }
        assert!(matches!(engine.input('a'), EngineInputResult::Completed));
    }

    #[test]
    fn allows_ltu_for_sokuon() {
        let mut engine = TypingEngine::new("かっか").unwrap();
        for c in ['k', 'a', 'l', 't', 'u', 'k'] {
            assert!(matches!(engine.input(c), EngineInputResult::Accepted));
        }
        assert!(matches!(engine.input('a'), EngineInputResult::Completed));
    }

    #[test]
    fn allows_tch_series_for_sokuon_before_chi_row() {
        for (kana, romaji) in [
            ("っちゃ", "tcha"),
            ("っち", "tchi"),
            ("っちゅ", "tchu"),
            ("っちぇ", "tche"),
            ("っちょ", "tcho"),
        ] {
            let mut engine = TypingEngine::new(kana).unwrap();
            let chars: Vec<char> = romaji.chars().collect();
            for c in &chars[..chars.len() - 1] {
                assert!(
                    matches!(engine.input(*c), EngineInputResult::Accepted),
                    "failed input for {kana} with {romaji}"
                );
            }
            assert!(
                matches!(
                    engine.input(chars[chars.len() - 1]),
                    EngineInputResult::Completed
                ),
                "did not complete for {kana} with {romaji}"
            );
        }
    }

    #[test]
    fn allows_consecutive_sokuon_with_double_consonants() {
        let mut engine = TypingEngine::new("っっか").unwrap();
        for c in "kkk".chars() {
            assert!(matches!(engine.input(c), EngineInputResult::Accepted));
        }
        assert!(matches!(engine.input('a'), EngineInputResult::Completed));
    }

    #[test]
    fn allows_consecutive_sokuon_with_mixed_inputs() {
        let mut engine = TypingEngine::new("っっか").unwrap();
        let chars: Vec<char> = "ltukka".chars().collect();
        for c in &chars[..chars.len() - 1] {
            assert!(matches!(engine.input(*c), EngineInputResult::Accepted));
        }
        assert!(matches!(
            engine.input(chars[chars.len() - 1]),
            EngineInputResult::Completed
        ));
    }

    #[test]
    fn allows_terminal_sokuon() {
        let mut engine = TypingEngine::new("っ").unwrap();
        let chars: Vec<char> = "ltu".chars().collect();
        for c in &chars[..chars.len() - 1] {
            assert!(matches!(engine.input(*c), EngineInputResult::Accepted));
        }
        assert!(matches!(
            engine.input(chars[chars.len() - 1]),
            EngineInputResult::Completed
        ));
    }

    #[test]
    fn allows_repeated_terminal_sokuon() {
        let mut engine = TypingEngine::new("っっ").unwrap();
        let chars: Vec<char> = "ltuxtu".chars().collect();
        for c in &chars[..chars.len() - 1] {
            assert!(matches!(engine.input(*c), EngineInputResult::Accepted));
        }
        assert!(matches!(
            engine.input(chars[chars.len() - 1]),
            EngineInputResult::Completed
        ));
    }

    #[test]
    fn allows_long_consecutive_sokuon_without_panic() {
        let _engine = TypingEngine::new("っっっっっか").unwrap();
    }

    #[test]
    fn allows_symbol_inputs() {
        let mut engine = TypingEngine::new("あ、い。う！え？").unwrap();
        let chars: Vec<char> = "a,i.u!e?".chars().collect();
        for c in &chars[..chars.len() - 1] {
            assert!(matches!(engine.input(*c), EngineInputResult::Accepted));
        }
        assert!(matches!(
            engine.input(chars[chars.len() - 1]),
            EngineInputResult::Completed
        ));
    }

    #[test]
    fn allows_dash_for_choonpu() {
        let mut engine = TypingEngine::new("らーめん").unwrap();
        let chars: Vec<char> = "ra-menn".chars().collect();
        for c in &chars[..chars.len() - 1] {
            assert!(matches!(engine.input(*c), EngineInputResult::Accepted));
        }
        assert!(matches!(
            engine.input(chars[chars.len() - 1]),
            EngineInputResult::Completed
        ));
    }

    #[test]
    fn disallows_vowel_repeat_for_choonpu() {
        let mut engine = TypingEngine::new("らーめん").unwrap();
        for c in "ra".chars() {
            assert!(matches!(engine.input(c), EngineInputResult::Accepted));
        }
        assert!(matches!(engine.input('a'), EngineInputResult::Rejected));
    }

    #[test]
    fn disallows_mixed_sokuon_and_following_option() {
        let mut engine = TypingEngine::new("はっしゃ").unwrap();
        for c in "has".chars() {
            assert!(matches!(engine.input(c), EngineInputResult::Accepted));
        }
        assert!(matches!(engine.input('c'), EngineInputResult::Rejected));
    }

    #[test]
    fn allows_consistent_c_path_for_sokuon_before_sha() {
        let mut engine = TypingEngine::new("はっしゃ").unwrap();
        let chars: Vec<char> = "haccixya".chars().collect();
        for c in &chars[..chars.len() - 1] {
            assert!(matches!(engine.input(*c), EngineInputResult::Accepted));
        }
        assert!(matches!(
            engine.input(chars[chars.len() - 1]),
            EngineInputResult::Completed
        ));
    }

    #[test]
    fn supports_wide_coverage_sentence() {
        let mut engine = TypingEngine::new("にとをおうものいっとをもえず").unwrap();
        let chars: Vec<char> = "nitowooumonoittowomoezu".chars().collect();
        for c in &chars[..chars.len() - 1] {
            assert!(matches!(engine.input(*c), EngineInputResult::Accepted));
        }
        assert!(matches!(
            engine.input(chars[chars.len() - 1]),
            EngineInputResult::Completed
        ));
    }

    #[test]
    fn allows_l_small_kana_composition() {
        let mut engine = TypingEngine::new("きゅ").unwrap();
        let chars: Vec<char> = "kilyu".chars().collect();
        for c in &chars[..chars.len() - 1] {
            assert!(matches!(engine.input(*c), EngineInputResult::Accepted));
        }
        assert!(matches!(
            engine.input(chars[chars.len() - 1]),
            EngineInputResult::Completed
        ));
    }

    #[test]
    fn guide_picks_shortest_route() {
        let engine = TypingEngine::new("つ").unwrap();
        assert_eq!(engine.guide(), "tu");
    }

    #[test]
    fn guide_prefers_direct_yoon_after_repeated_sokuon() {
        let engine = TypingEngine::new("みっっっっちゃる").unwrap();
        assert_eq!(engine.guide(), "mitttttyaru");
    }

    #[test]
    fn guide_prefers_direct_chi_row_yoon_after_repeated_sokuon() {
        for (kana, expected) in [
            ("みっっっっちゃる", "mitttttyaru"),
            ("みっっっっちゅる", "mitttttyuru"),
            ("みっっっっちぇる", "mitttttyeru"),
            ("みっっっっちょる", "mitttttyoru"),
        ] {
            let engine = TypingEngine::new(kana).unwrap();
            assert_eq!(engine.guide(), expected, "unexpected guide for {kana}");
        }
    }

    #[test]
    fn guide_after_shared_prefix_prefers_shorter_branch() {
        let mut engine = TypingEngine::new("みっっっっちゃる").unwrap();
        for c in "mittttt".chars() {
            assert!(matches!(
                engine.input(c),
                EngineInputResult::Accepted | EngineInputResult::Completed
            ));
        }
        assert_eq!(engine.guide(), "yaru");
    }

    #[test]
    fn reports_completed_reading_progress_for_basic_input() {
        let mut engine = TypingEngine::new("かき").unwrap();
        assert_eq!(engine.completed_char_range(), (0, 0));
        assert_eq!(engine.completed_reading(), "");
        assert_eq!(engine.furthest_completed_reading(), "");

        assert!(matches!(engine.input('k'), EngineInputResult::Accepted));
        assert_eq!(engine.completed_char_range(), (0, 0));

        assert!(matches!(engine.input('a'), EngineInputResult::Accepted));
        assert_eq!(engine.completed_char_range(), (1, 1));
        assert_eq!(engine.completed_reading(), "か");
        assert_eq!(engine.furthest_completed_reading(), "か");

        assert!(matches!(engine.input('k'), EngineInputResult::Accepted));
        assert_eq!(engine.completed_char_range(), (1, 1));

        assert!(matches!(engine.input('i'), EngineInputResult::Completed));
        assert_eq!(engine.completed_char_range(), (2, 2));
        assert_eq!(engine.completed_reading(), "かき");
        assert_eq!(engine.furthest_completed_reading(), "かき");
    }

    #[test]
    fn reports_progress_range_for_ambiguous_hatsuon_state() {
        let mut engine = TypingEngine::new("かんか").unwrap();
        for c in ['k', 'a'] {
            assert!(matches!(engine.input(c), EngineInputResult::Accepted));
        }
        assert_eq!(engine.completed_char_range(), (1, 1));

        assert!(matches!(engine.input('n'), EngineInputResult::Accepted));
        assert_eq!(engine.completed_char_range(), (1, 2));
        assert_eq!(engine.completed_reading(), "か");
        assert_eq!(engine.furthest_completed_reading(), "かん");
    }

    #[test]
    fn reports_sokuon_progress_after_ltu_before_following_kana_completion() {
        let mut engine = TypingEngine::new("がっこう").unwrap();
        for c in ['g', 'a', 'l', 't', 'u'] {
            assert!(matches!(engine.input(c), EngineInputResult::Accepted));
        }
        assert_eq!(engine.completed_char_range(), (2, 2));
        assert_eq!(engine.completed_reading(), "がっ");
    }

    #[test]
    fn reports_yoon_progress_after_base_kana_for_composed_input() {
        let mut engine = TypingEngine::new("きゃ").unwrap();
        for c in ['k', 'i'] {
            assert!(matches!(engine.input(c), EngineInputResult::Accepted));
        }
        assert_eq!(engine.completed_char_range(), (1, 1));
        assert_eq!(engine.completed_reading(), "き");

        for c in ['x', 'y'] {
            assert!(matches!(engine.input(c), EngineInputResult::Accepted));
        }
        assert!(matches!(engine.input('a'), EngineInputResult::Completed));
        assert_eq!(engine.completed_char_range(), (2, 2));
    }

    #[test]
    fn keeps_yoon_progress_after_l_in_kilyou_path() {
        let mut engine = TypingEngine::new("きょう").unwrap();
        for c in ['k', 'i'] {
            assert!(matches!(engine.input(c), EngineInputResult::Accepted));
        }
        assert_eq!(engine.completed_char_range(), (1, 1));

        assert!(matches!(engine.input('l'), EngineInputResult::Accepted));
        assert_eq!(engine.completed_char_range(), (1, 1));
        assert_eq!(engine.completed_reading(), "き");
    }

    #[test]
    fn disallows_single_n_before_n_row_kana() {
        let mut engine = TypingEngine::new("かんに").unwrap();
        for c in ['k', 'a', 'n'] {
            assert!(matches!(engine.input(c), EngineInputResult::Accepted));
        }

        assert!(matches!(engine.input('i'), EngineInputResult::Rejected));
        assert!(matches!(engine.input('n'), EngineInputResult::Accepted));
        assert!(matches!(engine.input('n'), EngineInputResult::Accepted));
        assert!(matches!(engine.input('i'), EngineInputResult::Completed));
    }
}
