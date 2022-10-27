use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

// all methodos should return a RESULT. Need to remove unwraps from them
trait ITrie {
    fn initialize(file_content: &str, suggestion_number: u8) -> Trie;
    fn insert_word(&mut self, word: String, popularity: u16);
    fn increase_popularity(&mut self, word: String);
    fn search_with_prefix(&self, prefix: String) -> Vec<(String, u16)>;
}

#[derive(Debug, Clone)]
pub struct Trie {
    pub root: Box<TrieNode>,
    pub suggestion_number: u8,
}

impl Trie {
    fn search_all_words_from_node(
        node: &Box<TrieNode>,
        current_word: String,
        vec: &mut Vec<(String, u16)>,
    ) {
        for (letter, child_node) in &node.children {
            let mut new_word = current_word.clone();
            new_word.push(*letter);

            if let Some(value) = child_node.value {
                vec.push((new_word.clone(), value));
            }

            Trie::search_all_words_from_node(child_node, new_word, vec);
        }
    }
}

fn handle_case_sensitive(prefix: String) -> String {
    if prefix.is_empty() {
        return prefix;
    }

    let mut handled_prefix = String::new();
    for char in prefix.chars() {
        handled_prefix.push(char.to_ascii_lowercase());
    }

    let mut v: Vec<char> = handled_prefix.chars().collect();
    v[0] = v[0].to_uppercase().nth(0).unwrap();
    let handled_prefix: String = v.into_iter().collect();

    handled_prefix
}

impl ITrie for Trie {
    fn initialize(file_content: &str, suggestion_number: u8) -> Trie {
        let mut trie = Trie {
            root: Box::new(TrieNode::new(' ', None)),
            suggestion_number,
        };

        let values: HashMap<String, u16> = serde_json::from_str(file_content).unwrap();

        for (word, popularity) in values {
            trie.insert_word(word, popularity);
        }

        trie
    }

    fn insert_word(&mut self, word: String, popularity: u16) {
        let mut node = &mut self.root;
        let len = word.len();
        let mut i = 0;

        for char in word.chars() {
            if !node.children.contains_key(&char) {
                let value = if i == len - 1 { Some(popularity) } else { None };
                let new_node = Box::new(TrieNode::new(char.clone(), value));
                node.children.insert(char, new_node);
            }

            node = node.children.get_mut(&char).unwrap();

            if i == len - 1 {
                node.value = Some(popularity)
            }

            i = i + 1;
        }
    }

    fn increase_popularity(&mut self, word: String) {
        let mut node = &mut self.root;

        for char in word.chars() {
            node = node.children.get_mut(&char).unwrap();
        }

        node.value = Some(node.value.unwrap_or(0) + 1);
    }

    //[ok] order by value and return SUGGESTION_NUMBER items
    //[ok] name in ascending order if they have equal popularity
    //[test] always leaving the exact match (a name that is exactly the received prefix) at the beginning if there is one
    //[ok] If the prefix segment of the path is not given or it's empty, it returns the SUGGESTION_NUMBER names with the highest popularity.
    //[ok] handle case sensitive
    fn search_with_prefix(&self, prefix: String) -> Vec<(String, u16)> {
        let mut node = &self.root;
        let suggestion_number: usize = self.suggestion_number.into();
        let prefix = handle_case_sensitive(prefix);

        for char in prefix.chars() {
            node = node.children.get(&char).unwrap();
        }

        let mut words_match_prefix: Vec<(String, u16)> = Vec::new();
        if let Some(value) = node.value {
            words_match_prefix.push((prefix.clone(), value));
        }

        Trie::search_all_words_from_node(node, prefix, &mut words_match_prefix);
        words_match_prefix.sort_by(|(name_one, score_one), (name_two, score_two)| {
            score_two.cmp(score_one).then(name_one.cmp(name_two))
        });
        words_match_prefix.truncate(suggestion_number);

        words_match_prefix
    }
}

#[derive(Debug, Clone)]
pub struct TrieNode {
    pub children: HashMap<char, Box<TrieNode>>,
    pub letter: char,
    pub value: Option<u16>,
}

impl TrieNode {
    pub fn new(letter: char, value: Option<u16>) -> TrieNode {
        TrieNode {
            children: HashMap::new(),
            letter,
            value,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::__private::de;

    use super::*;

    fn compare_tries(root_a: &Box<TrieNode>, root_b: &Box<TrieNode>) -> bool {
        //two comparisons because order can change.
        let is_b_equal_a = recursive_compare_nodes(root_a, root_b);
        let is_a_equal_b = recursive_compare_nodes(root_b, root_a);

        is_b_equal_a && is_a_equal_b
    }

    fn recursive_compare_nodes(node_a: &Box<TrieNode>, node_b: &Box<TrieNode>) -> bool {
        let mut return_value = true;

        if node_a.letter != node_b.letter || node_a.value != node_b.value {
            println!("node_a: {:?}, node_b: {:?}", node_a, node_b);
            return false;
        }

        for child_a in &node_a.children {
            let child_b = node_b.children.get(&child_a.0);

            if child_b.is_none() || return_value == false {
                return false;
            } else {
                return_value = compare_tries(&child_a.1, child_b.unwrap());
            }
        }

        return_value
    }

    fn print_trie(node: &Box<TrieNode>, mut i: u8) {
        println!("[{}] {}-{:?}", i, node.letter, node.value);
        i = i + 1;
        for child in &node.children {
            print_trie(child.1, i);
        }
    }

    fn initialize_testing_trie() -> Trie {
        let mut expected_trie = Trie {
            root: Box::new(TrieNode::new(' ', None)),
            suggestion_number: 10,
        };

        let mut node: &mut Box<TrieNode> = &mut expected_trie.root;

        // (A) first level
        node.children
            .insert('A', Box::new(TrieNode::new('A', None)));

        // (A) second level
        node = node.children.get_mut(&'A').unwrap();
        node.children
            .insert('a', Box::new(TrieNode::new('a', None)));
        node.children
            .insert('b', Box::new(TrieNode::new('b', None)));

        // (A) third level
        node = node.children.get_mut(&'a').unwrap();
        node.children
            .insert('r', Box::new(TrieNode::new('r', Some(361))));

        // (A) fourth level
        node = node.children.get_mut(&'r').unwrap();
        node.children
            .insert('i', Box::new(TrieNode::new('i', Some(151))));

        // (A) third level
        node = &mut expected_trie.root;
        node = node.children.get_mut(&'A').unwrap();
        node = node.children.get_mut(&'b').unwrap();

        node.children
            .insert('a', Box::new(TrieNode::new('a', Some(704))));

        // (A) fourth level
        node = node.children.get_mut(&'a').unwrap();
        node.children
            .insert('g', Box::new(TrieNode::new('g', Some(608))));

        // (A) third level
        node = &mut expected_trie.root;
        node = node.children.get_mut(&'A').unwrap();
        node = node.children.get_mut(&'b').unwrap();
        node.children
            .insert('e', Box::new(TrieNode::new('e', Some(300))));

        // (B) first level
        node = &mut expected_trie.root;
        node.children
            .insert('B', Box::new(TrieNode::new('B', None)));

        // (B) second level
        node = node.children.get_mut(&'B').unwrap();
        node.children
            .insert('a', Box::new(TrieNode::new('a', Some(5))));
        node.children
            .insert('e', Box::new(TrieNode::new('e', Some(50))));
        node.children
            .insert('c', Box::new(TrieNode::new('c', Some(50))));

        expected_trie
    }

    fn increase_popularity_testing_trie() -> Trie {
        let mut expected_trie = Trie {
            root: Box::new(TrieNode::new(' ', None)),
            suggestion_number: 10,
        };

        let mut node: &mut Box<TrieNode> = &mut expected_trie.root;

        // (A) first level
        node.children
            .insert('A', Box::new(TrieNode::new('A', None)));

        // (A) second level
        node = node.children.get_mut(&'A').unwrap();
        node.children
            .insert('a', Box::new(TrieNode::new('a', None)));
        node.children
            .insert('b', Box::new(TrieNode::new('b', None)));

        // (A) third level
        node = node.children.get_mut(&'a').unwrap();
        node.children
            .insert('r', Box::new(TrieNode::new('r', Some(361))));

        // (A) fourth level
        node = node.children.get_mut(&'r').unwrap();
        node.children
            .insert('i', Box::new(TrieNode::new('i', Some(151))));

        // (A) third level
        node = &mut expected_trie.root;
        node = node.children.get_mut(&'A').unwrap();
        node = node.children.get_mut(&'b').unwrap();

        node.children
            .insert('a', Box::new(TrieNode::new('a', Some(704))));

        // (A) fourth level
        node = node.children.get_mut(&'a').unwrap();
        node.children
            .insert('g', Box::new(TrieNode::new('g', Some(608))));

        // (A) third level
        node = &mut expected_trie.root;
        node = node.children.get_mut(&'A').unwrap();
        node = node.children.get_mut(&'b').unwrap();
        node.children
            .insert('e', Box::new(TrieNode::new('e', Some(301))));

        // (B) first level
        node = &mut expected_trie.root;
        node.children
            .insert('B', Box::new(TrieNode::new('B', None)));

        // (B) second level
        node = node.children.get_mut(&'B').unwrap();
        node.children
            .insert('a', Box::new(TrieNode::new('a', Some(5))));
        node.children
            .insert('e', Box::new(TrieNode::new('e', Some(50))));

        expected_trie
    }

    #[test]
    fn t_initialize() {
        let file_content =
            "{\"Aar\":361,\"Aari\":151,\"Aba\":704,\"Abag\":608, \"Abe\": 300, \"Ba\": 5, \"Be\": 50, \"Bc\": 50}";
        let trie = Trie::initialize(file_content, 10);

        let expected_trie = initialize_testing_trie();

        print_trie(&expected_trie.root, 0);
        print_trie(&trie.root, 0);

        assert!(compare_tries(&trie.root, &expected_trie.root));
    }

    #[test]
    fn t_insert_word_into_db() {}

    #[test]
    fn t_increase_popularity() {
        let file_content =
        "{\"Aar\":361,\"Aari\":151,\"Aba\":704,\"Abag\":608, \"Abe\": 300, \"Ba\": 5, \"Be\": 50}";
        let mut trie = Trie::initialize(file_content, 10);
        trie.increase_popularity("Abe".to_string());

        let expected_trie = increase_popularity_testing_trie();

        // print_trie(&trie.root, 0);
        // print_trie(&expected_trie.root, 0);

        assert!(compare_tries(&trie.root, &expected_trie.root));
    }

    #[test]
    fn t_search_with_prefix_prefix_not_included() {
        let trie = initialize_testing_trie();

        let words = trie.search_with_prefix("Ab".to_string());

        let expected_words: Vec<(String, u16)> = vec![
            ("Aba".to_string(), 704),
            ("Abag".to_string(), 608),
            ("Abe".to_string(), 300),
        ];

        assert_eq!(expected_words, words);
    }

    #[test]
    fn t_search_with_prefix_prefix_included() {
        let trie = initialize_testing_trie();

        let words = trie.search_with_prefix("Aba".to_string());

        let expected_words: Vec<(String, u16)> =
            vec![("Aba".to_string(), 704), ("Abag".to_string(), 608)];

        assert_eq!(expected_words, words);
    }

    #[test]
    fn t_search_with_prefix_take_two() {
        let mut trie = initialize_testing_trie();
        trie.suggestion_number = 2;
        let words = trie.search_with_prefix("Ab".to_string());

        let expected_words: Vec<(String, u16)> =
            vec![("Aba".to_string(), 704), ("Abag".to_string(), 608)];

        assert_eq!(expected_words, words);
    }

    #[test]
    fn t_search_with_prefix_empty_prefix() {
        let mut trie = initialize_testing_trie();
        trie.suggestion_number = 3;
        let words = trie.search_with_prefix("".to_string());

        let expected_words: Vec<(String, u16)> = vec![
            ("Aba".to_string(), 704),
            ("Abag".to_string(), 608),
            ("Aar".to_string(), 361),
        ];

        assert_eq!(expected_words, words);
    }

    #[test]
    fn t_search_with_prefix_same_value() {
        let mut trie = initialize_testing_trie();
        trie.suggestion_number = 2;
        let words = trie.search_with_prefix("B".to_string());

        let expected_words: Vec<(String, u16)> =
            vec![("Bc".to_string(), 50), ("Be".to_string(), 50)];

        assert_eq!(expected_words, words);
    }

    #[test]
    fn t_search_with_prefix_case_insensitive() {
        let mut trie = initialize_testing_trie();
        trie.suggestion_number = 2;
        let words = trie.search_with_prefix("b".to_string());

        let expected_words: Vec<(String, u16)> =
            vec![("Bc".to_string(), 50), ("Be".to_string(), 50)];

        assert_eq!(expected_words, words);

        let words = trie.search_with_prefix("AA".to_string());

        let expected_words: Vec<(String, u16)> =
            vec![("Aar".to_string(), 361), ("Aari".to_string(), 151)];

        assert_eq!(expected_words, words);
    }
}
