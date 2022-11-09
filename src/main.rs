use crate::trie::{ITrie, Trie};
use actix_web::{get, middleware, App, HttpResponse, HttpServer};
use log::info;
use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;

mod app_error;
mod config;
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

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("log4rs.yml", log4rs::config::Deserializers::default()).unwrap();

    info!("Starting service...");

    let config =
        crate::config::Config::from_env().expect("Could not load configuration from environment!");

    // let trie: Arc<Mutex<trie::Trie>>;
    let file_content = fs::read_to_string("./names.json").expect("JSON file not found");

    let trie = Trie::initialize(&file_content, config.suggestion_number).unwrap();
    let shared_trie: Arc<Mutex<Trie>> = Arc::new(Mutex::new(trie));

    let bind_address: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("Unable to parse socket address");

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(shared_trie.clone())
            .service(handlers::get_words_match_prefix)
            .service(handlers::increase_popularity)
            .service(health_check)
        // .service(whatsapp_hook)
    })
    .bind(bind_address)?
    .run()
    .await
}

#[allow(clippy::unused_async)]
#[get("/health")]
async fn health_check() -> HttpResponse {
    // info!("Service is health    y and accepting requests");
    HttpResponse::Ok().json("Service is healthy")
}
