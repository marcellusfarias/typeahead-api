use crate::app_error::AppError;
use std::collections::HashMap;

// all methodos should return a RESULT. Need to remove unwraps from them
trait ITrie {
    fn initialize(file_content: &str, suggestion_number: u8) -> Trie;
    fn insert_word(&mut self, word: String, popularity: u16);
    fn increase_popularity(&mut self, word: String) -> Result<WordData, AppError>;
    fn get_typeahead_words(&self, prefix: String) -> Vec<WordData>;
}

#[derive(Debug, Clone)]
pub struct Trie {
    pub root: Box<TrieNode>,
    pub suggestion_number: u8,
}

impl Trie {
    fn get_words_with_same_prefix(prefix_node: &Box<TrieNode>, result_vec: &mut Vec<WordData>) {
        for (_letter, child_node) in &prefix_node.children {
            if let Some(word_data) = child_node.word_data.clone() {
                result_vec.push(word_data);
            }

            Trie::get_words_with_same_prefix(child_node, result_vec);
        }
    }
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
        let lowercase_word = word.clone().to_ascii_lowercase();

        for char in lowercase_word.chars() {
            if !node.children.contains_key(&char) {
                let new_node = Box::new(TrieNode::new(char.clone(), None));
                node.children.insert(char, new_node);
            }

            node = node.children.get_mut(&char).unwrap();
        }

        node.word_data = Some(WordData::new(word, popularity));
    }

    fn increase_popularity(&mut self, word: String) -> Result<WordData, AppError> {
        let mut node = &mut self.root;
        let lowercase_word = word.clone().to_ascii_lowercase();

        for char in lowercase_word.chars() {
            node = node
                .children
                .get_mut(&char)
                .ok_or(AppError::WordDoesNotExist)?;
        }

        let mut updated_word_data = node.word_data.clone().ok_or(AppError::WordDoesNotExist)?;
        updated_word_data.popularity = updated_word_data.popularity + 1;
        node.word_data = Some(updated_word_data.clone());

        Ok(updated_word_data)
    }

    //[ok] order by value and return SUGGESTION_NUMBER items
    //[ok] name in ascending order if they have equal popularity
    //[ok] always leaving the exact match (a name that is exactly the received prefix) at the beginning if there is one
    //[ok] If the prefix segment of the path is not given or it's empty, it returns the SUGGESTION_NUMBER names with the highest popularity.
    //[ok] handle case sensitive
    fn get_typeahead_words(&self, prefix: String) -> Vec<WordData> {
        let mut node = &self.root;
        let prefix = prefix.to_ascii_lowercase();

        for char in prefix.chars() {
            node = node.children.get(&char).unwrap();
        }

        let mut words_with_same_prefix: Vec<WordData> = Vec::new();
        let prefix_word_data: Option<WordData> = node.word_data.clone();

        Trie::get_words_with_same_prefix(node, &mut words_with_same_prefix);

        //order by popularity desc and then by word asc
        words_with_same_prefix.sort_by(|word_data_one, word_data_two| {
            word_data_two
                .popularity
                .cmp(&word_data_one.popularity)
                .then(word_data_one.word.cmp(&word_data_two.word))
        });

        //insert word that match prefix at first position
        if let Some(word_data) = prefix_word_data {
            words_with_same_prefix.splice(0..0, vec![word_data].iter().cloned());
        }

        //return only SUGGESTION_NUMBER items
        words_with_same_prefix.truncate(self.suggestion_number.into());

        words_with_same_prefix
    }
}

//storing the word in the node so we can work with lowercase all over the way avoiding case insensitive problems.
//assuming we can't have 2 same words but with different casing. E.g., Rose-Marie and Rose-marie
#[derive(Debug, Clone, PartialEq)]
pub struct WordData {
    pub word: String,
    pub popularity: u16,
}

impl WordData {
    pub fn new(word: String, popularity: u16) -> WordData {
        WordData { word, popularity }
    }
}

#[derive(Debug, Clone)]
pub struct TrieNode {
    pub children: HashMap<char, Box<TrieNode>>,
    pub letter: char,
    pub word_data: Option<WordData>,
}

impl TrieNode {
    pub fn new(letter: char, word_data: Option<WordData>) -> TrieNode {
        TrieNode {
            children: HashMap::new(),
            letter,
            word_data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compare_tries(root_a: &Box<TrieNode>, root_b: &Box<TrieNode>) -> bool {
        //two comparisons because order can change.
        println!("Comparing b to a");
        let is_b_equal_a = recursively_compare_tries(root_a, root_b);

        println!("Comparing a to b");
        let is_a_equal_b = recursively_compare_tries(root_b, root_a);

        is_b_equal_a && is_a_equal_b
    }

    fn recursively_compare_tries(node_a: &Box<TrieNode>, node_b: &Box<TrieNode>) -> bool {
        let mut return_value = true;

        if node_a.letter != node_b.letter || node_a.word_data != node_b.word_data {
            println!("node_a: {:?}, node_b: {:?}", node_a, node_b);
            return false;
        }

        for child_a in &node_a.children {
            let child_b = node_b.children.get(&child_a.0);

            if child_b.is_none() || return_value == false {
                println!("child_a: {:?}, child_b: {:?}", child_a, child_b);
                return false;
            } else {
                return_value = recursively_compare_tries(&child_a.1, child_b.unwrap());
            }
        }

        return_value
    }

    fn print_trie(node: &Box<TrieNode>, mut i: u8) {
        println!("[{}] {}-{:?}", i, node.letter, node.word_data);
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
            .insert('a', Box::new(TrieNode::new('a', None)));

        // (A) second level
        node = node.children.get_mut(&'a').unwrap();
        node.children
            .insert('a', Box::new(TrieNode::new('a', None)));
        node.children
            .insert('b', Box::new(TrieNode::new('b', None)));

        // (A) third level
        node = node.children.get_mut(&'a').unwrap();
        node.children.insert(
            'r',
            Box::new(TrieNode::new(
                'r',
                Some(WordData::new("Aar".to_string(), 361)),
            )),
        );

        // (A) fourth level
        node = node.children.get_mut(&'r').unwrap();
        node.children.insert(
            'i',
            Box::new(TrieNode::new(
                'i',
                Some(WordData::new("Aari".to_string(), 151)),
            )),
        );

        // (A) third level
        node = &mut expected_trie.root;
        node = node.children.get_mut(&'a').unwrap();
        node = node.children.get_mut(&'b').unwrap();
        node.children.insert(
            'a',
            Box::new(TrieNode::new(
                'a',
                Some(WordData::new("Aba".to_string(), 608)),
            )),
        );

        // (A) fourth level
        node = node.children.get_mut(&'a').unwrap();
        node.children.insert(
            'g',
            Box::new(TrieNode::new(
                'g',
                Some(WordData::new("Abag".to_string(), 704)),
            )),
        );

        // (A) third level
        node = &mut expected_trie.root;
        node = node.children.get_mut(&'a').unwrap();
        node = node.children.get_mut(&'b').unwrap();
        node.children.insert(
            'e',
            Box::new(TrieNode::new(
                'e',
                Some(WordData::new("Abe".to_string(), 300)),
            )),
        );

        // (B) first level
        node = &mut expected_trie.root;
        node.children
            .insert('b', Box::new(TrieNode::new('b', None)));

        // (B) second level
        node = node.children.get_mut(&'b').unwrap();
        node.children.insert(
            'a',
            Box::new(TrieNode::new('a', Some(WordData::new("Ba".to_string(), 5)))),
        );
        node.children.insert(
            'e',
            Box::new(TrieNode::new(
                'e',
                Some(WordData::new("Be".to_string(), 50)),
            )),
        );
        node.children.insert(
            'c',
            Box::new(TrieNode::new(
                'c',
                Some(WordData::new("Bc".to_string(), 50)),
            )),
        );

        // (B) third level
        node = node.children.get_mut(&'a').unwrap();
        node.children.insert(
            'h',
            Box::new(TrieNode::new(
                'h',
                Some(WordData::new("Bah".to_string(), 5)),
            )),
        );

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
            .insert('a', Box::new(TrieNode::new('a', None)));

        // (A) second level
        node = node.children.get_mut(&'a').unwrap();
        node.children
            .insert('a', Box::new(TrieNode::new('a', None)));
        node.children
            .insert('b', Box::new(TrieNode::new('b', None)));

        // (A) third level
        node = node.children.get_mut(&'a').unwrap();
        node.children.insert(
            'r',
            Box::new(TrieNode::new(
                'r',
                Some(WordData::new("Aar".to_string(), 361)),
            )),
        );

        // (A) fourth level
        node = node.children.get_mut(&'r').unwrap();
        node.children.insert(
            'i',
            Box::new(TrieNode::new(
                'i',
                Some(WordData::new("Aari".to_string(), 151)),
            )),
        );

        // (A) third level
        node = &mut expected_trie.root;
        node = node.children.get_mut(&'a').unwrap();
        node = node.children.get_mut(&'b').unwrap();
        node.children.insert(
            'a',
            Box::new(TrieNode::new(
                'a',
                Some(WordData::new("Aba".to_string(), 608)),
            )),
        );

        // (A) fourth level
        node = node.children.get_mut(&'a').unwrap();
        node.children.insert(
            'g',
            Box::new(TrieNode::new(
                'g',
                Some(WordData::new("Abag".to_string(), 704)),
            )),
        );

        // (A) third level
        node = &mut expected_trie.root;
        node = node.children.get_mut(&'a').unwrap();
        node = node.children.get_mut(&'b').unwrap();
        node.children.insert(
            'e',
            Box::new(TrieNode::new(
                'e',
                Some(WordData::new("Abe".to_string(), 301)),
            )),
        );

        // (B) first level
        node = &mut expected_trie.root;
        node.children
            .insert('b', Box::new(TrieNode::new('b', None)));

        // (B) second level
        node = node.children.get_mut(&'b').unwrap();
        node.children.insert(
            'a',
            Box::new(TrieNode::new('a', Some(WordData::new("Ba".to_string(), 5)))),
        );
        node.children.insert(
            'e',
            Box::new(TrieNode::new(
                'e',
                Some(WordData::new("Be".to_string(), 50)),
            )),
        );
        node.children.insert(
            'c',
            Box::new(TrieNode::new(
                'c',
                Some(WordData::new("Bc".to_string(), 50)),
            )),
        );

        // (B) third level
        node = node.children.get_mut(&'a').unwrap();
        node.children.insert(
            'h',
            Box::new(TrieNode::new(
                'h',
                Some(WordData::new("Bah".to_string(), 5)),
            )),
        );

        expected_trie
    }

    #[test]
    fn t_initialize() {
        let file_content =
            "{\"Aar\":361,\"Aari\":151,\"Aba\":608,\"Abag\":704, \"Abe\": 300, \"Ba\": 5, \"Bah\": 5, \"Be\": 50, \"Bc\": 50}";
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
        "{\"Aar\":361,\"Aari\":151,\"Aba\":608,\"Abag\":704, \"Abe\": 300, \"Ba\": 5, \"Bah\": 5, \"Be\": 50, \"Bc\": 50}";
        let mut trie = Trie::initialize(file_content, 10);
        let _result = trie.increase_popularity("Abe".to_string());

        let expected_trie = increase_popularity_testing_trie();

        // print_trie(&trie.root, 0);
        // print_trie(&expected_trie.root, 0);

        assert!(compare_tries(&trie.root, &expected_trie.root));
    }

    #[test]
    fn t_search_with_prefix_prefix_not_included() {
        let trie = initialize_testing_trie();

        let words = trie.get_typeahead_words("Ab".to_string());

        let expected_words: Vec<WordData> = vec![
            WordData::new("Abag".to_string(), 704),
            WordData::new("Aba".to_string(), 608),
            WordData::new("Abe".to_string(), 300),
        ];

        assert_eq!(expected_words, words);
    }

    #[test]
    fn t_search_with_prefix_exact_match_prefix() {
        let trie = initialize_testing_trie();

        let words = trie.get_typeahead_words("Aba".to_string());

        let expected_words: Vec<WordData> = vec![
            WordData::new("Aba".to_string(), 608),
            WordData::new("Abag".to_string(), 704),
        ];

        assert_eq!(expected_words, words);
    }

    #[test]
    fn t_search_with_prefix_take_two() {
        let mut trie = initialize_testing_trie();
        trie.suggestion_number = 2;
        let words = trie.get_typeahead_words("Ab".to_string());

        let expected_words: Vec<WordData> = vec![
            WordData::new("Abag".to_string(), 704),
            WordData::new("Aba".to_string(), 608),
        ];

        assert_eq!(expected_words, words);
    }

    #[test]
    fn t_search_with_prefix_empty_prefix() {
        let mut trie = initialize_testing_trie();
        trie.suggestion_number = 3;
        let words = trie.get_typeahead_words("".to_string());

        let expected_words: Vec<WordData> = vec![
            WordData::new("Abag".to_string(), 704),
            WordData::new("Aba".to_string(), 608),
            WordData::new("Aar".to_string(), 361),
        ];

        assert_eq!(expected_words, words);
    }

    #[test]
    fn t_search_with_prefix_same_value() {
        let mut trie = initialize_testing_trie();
        trie.suggestion_number = 2;
        let words = trie.get_typeahead_words("B".to_string());

        let expected_words: Vec<WordData> = vec![
            WordData::new("Bc".to_string(), 50),
            WordData::new("Be".to_string(), 50),
        ];

        assert_eq!(expected_words, words);
    }

    #[test]
    fn t_search_with_prefix_case_insensitive() {
        let mut trie = initialize_testing_trie();
        trie.suggestion_number = 2;
        let words = trie.get_typeahead_words("b".to_string());

        let expected_words: Vec<WordData> = vec![
            WordData::new("Bc".to_string(), 50),
            WordData::new("Be".to_string(), 50),
        ];

        assert_eq!(expected_words, words);

        let words = trie.get_typeahead_words("AA".to_string());

        let expected_words: Vec<WordData> = vec![
            WordData::new("Aar".to_string(), 361),
            WordData::new("Aari".to_string(), 151),
        ];

        assert_eq!(expected_words, words);
    }

    #[test]
    fn t_search_with_prefix_testing_ordering() {
        let mut trie = initialize_testing_trie();
        trie.suggestion_number = 4;
        let words = trie.get_typeahead_words("b".to_string());

        let expected_words: Vec<WordData> = vec![
            WordData::new("Bc".to_string(), 50),
            WordData::new("Be".to_string(), 50),
            WordData::new("Ba".to_string(), 5),
            WordData::new("Bah".to_string(), 5),
        ];

        assert_eq!(expected_words, words);
    }
}
