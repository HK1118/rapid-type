use crate::result::EngineInputResult;
use crate::romaji_table::ROMAJI_TABLE;

#[derive(Debug, Clone, Copy)]
struct Transition {
    input: char,
    target: usize,
}

#[derive(Debug, Default, Clone)]
struct Node {
    transitions: Vec<Transition>,
    is_terminal: bool,
}

pub struct TypingEngine {
    nodes: Vec<Node>,
    current_node: usize,
}

impl TypingEngine {
    pub fn new(reading: &str) -> Self {
        let mut nodes = vec![Node::default()];
        let mut prev_node_idx = 0;

        let tokens: Vec<char> = reading.chars().collect();

        let mut i = 0;
        while i < tokens.len() {
            let mut romaji_options = ROMAJI_TABLE
                .get(tokens[i].to_string().as_str())
                .unwrap()
                .clone();

            romaji_options.sort_by_key(|option| option.priority);

            let romaji = romaji_options.first().unwrap().romaji.clone();

            for c in romaji.chars() {
                let next_node_idx = nodes.len();
                nodes.push(Node::default());

                // 前のノードから今の文字への遷移を追加
                nodes[prev_node_idx].transitions.push(Transition {
                    input: c,
                    target: next_node_idx,
                });

                // 次の文字のためにノード番号を更新
                prev_node_idx = next_node_idx;
            }
            i += 1;
        }
        nodes[prev_node_idx].is_terminal = true;

        Self {
            nodes,
            current_node: 0,
        }
    }

    pub fn input(&mut self, key: char) -> EngineInputResult {
        let current = &self.nodes[self.current_node];

        for transition in &current.transitions {
            if transition.input == key {
                self.current_node = transition.target;
                // targetが終端ノードか確認
                if self.nodes[self.current_node].is_terminal {
                    return EngineInputResult::Completed;
                }
                return EngineInputResult::Accepted;
            }
        }

        if current.is_terminal {
            EngineInputResult::AlreadyCompleted
        } else {
            EngineInputResult::Rejected
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typing_engine() {
        let mut engine = TypingEngine::new("きゃたぴら");

        assert_eq!(engine.input('k'), EngineInputResult::Accepted);
        assert_eq!(engine.input('r'), EngineInputResult::Rejected);
        assert_eq!(engine.input('i'), EngineInputResult::Accepted);
        assert_eq!(engine.input('x'), EngineInputResult::Accepted);
        assert_eq!(engine.input('y'), EngineInputResult::Accepted);
        assert_eq!(engine.input('a'), EngineInputResult::Accepted);
        assert_eq!(engine.input('t'), EngineInputResult::Accepted);
        assert_eq!(engine.input('a'), EngineInputResult::Accepted);
        assert_eq!(engine.input('g'), EngineInputResult::Rejected);
        assert_eq!(engine.input('p'), EngineInputResult::Accepted);
        assert_eq!(engine.input('i'), EngineInputResult::Accepted);
        assert_eq!(engine.input('r'), EngineInputResult::Accepted);
        assert_eq!(engine.input('a'), EngineInputResult::Completed);
        assert_eq!(engine.input('x'), EngineInputResult::AlreadyCompleted);
    }
}
