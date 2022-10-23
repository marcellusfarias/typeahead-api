use std::sync::Arc;
use std::sync::Mutex;

mod app_error;
mod handlers;
mod trie;
// PARAMS: suggestion_number, port, host

// DB
// Pub methods
// 1. Initializes the tree preserving the case sensitive and using Arc<Mutex>, so we don't need to store it in a file stream.
// 2. Search prefix
// 3. Increase popularity
// Private methods:
// 1. Insert word

// HANDLERS
// 1. Receive GET /typeahead/{prefix} and returns an array of objects each one having the name and times (popularity) properties.
//      Name should always preserve the original case sensitive. Return SUGGESTION_NUMBER names.
// 2. POST /typeahead
//      It receives a JSON object with a name as the request body
//      (example: { "name": "Joanna" }), increases the popularity for that name in 1, and returns a 201 status code with an object with name and times properties considering the new state.
//      if no name is found, return 400

// APP_ERROR
//  1. Map all possible errors.
//  2. Converts TrieError to ReqwestError.

// APP_CONFIG
//  1. Check if needed for reading the env vars as params.

// MAIN
//  Read parameters, call Trie constructor and create webserver sharing the Trie as data.

// QUESTIONS:
// 1. Think we should store the tree on the disk or load the entire dataset into memory. Ask about file size?

fn main() {
    // let trie: Arc<Mutex<trie::Trie>> = trie::Trie::new(10);
    // let x = trie.lock().unwrap();
}
