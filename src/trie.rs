use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};
use serde_json::json;

const NAMES_FILE_PATH: &str = "./names.json";

trait ITrie {
    fn initialize_data(&mut self, suggestion_number: u8);
    fn insert_word(&mut self, word: String, popularity: u16); // change this to return a RESULT-
    fn increase_popularity(&mut self, word: String);
    fn search_with_prefix(&mut self, prefix: String) -> Vec<(String, u16)>;
}

#[derive(Debug, Clone)]
pub struct Trie {
    pub root: Box<TrieNode>,
}

impl ITrie for Trie {
    fn initialize_data(&mut self, suggestion_number: u8) {
        todo!()
    }

    fn insert_word(&mut self, word: String, popularity: u16) {
        let mut node = &mut self.root;
        let len = word.len();
        let mut i = 0;

        for char in word.chars() {
            if !node.children.contains_key(&char) {
                let new_node = Box::new(TrieNode {
                    children: HashMap::new(),
                    letter: char.clone(),
                    value: if i == len - 1 { Some(popularity) } else { None },
                });

                node.children.insert(char, new_node.clone());
            }

            node = node.children.get_mut(&char).unwrap();
            i = i + 1;
        }
    }

    //return RESULT
    //remove unwrap
    //change for loop
    fn increase_popularity(&mut self, word: String) {
        let mut node = &mut self.root;
        let len = word.len();
        let i = 0;

        for char in word.chars() {
            if i == len - 1 {
                node.value = Some(node.value.unwrap() + 1);
            }

            node = node.children.get_mut(&char).unwrap();
        }
    }

    fn search_with_prefix(&mut self, prefix: String) -> Vec<(String, u16)> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct TrieNode {
    pub children: HashMap<char, Box<TrieNode>>,
    pub letter: char,
    pub value: Option<u16>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_insert_word_into_db() {}

    #[test]
    fn t_update_popularity_word() {}

    #[test]
    fn t_insert_word_into_trie() {
        let mut trie = Trie {
            root: Box::new(TrieNode {
                children: HashMap::new(),
                letter: ' ',
                value: None,
            }),
        };
        let mut node = &mut trie.root;

        let word = "RUST";
        let popularity = 100;
        let len = word.len();
        let mut i = 0;

        for char in word.chars() {
            let value: Option<u16> = if i == len - 1 { Some(popularity) } else { None };

            let new_node = Box::new(TrieNode {
                children: HashMap::new(),
                letter: char.clone(),
                value: value,
            });

            if !node.children.contains_key(&char) {
                node.children.insert(char, new_node.clone());
            }

            node = node.children.get_mut(&char).unwrap();

            i = i + 1;

            println!("[{}] new_node: {:?}", i, new_node);
        }

        let mut node = &mut trie.root;
        let word = "RUDE";
        let popularity = 90;
        let len = word.len();
        let mut i = 0;

        for char in word.chars() {
            let value: Option<u16> = if i == len - 1 { Some(popularity) } else { None };

            let new_node = Box::new(TrieNode {
                children: HashMap::new(),
                letter: char.clone(),
                value: value,
            });

            if !node.children.contains_key(&char) {
                node.children.insert(char, new_node.clone());
            }

            node = node.children.get_mut(&char).unwrap();

            i = i + 1;

            println!("[{}] new_node: {:?}", i, new_node);
        }

        println!("tree: {:?}", trie);
    }
}
