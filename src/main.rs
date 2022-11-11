use crate::trie::{ITrie, Trie};
use actix_web::{get, middleware, App, HttpResponse, HttpServer, web};
use log::info;
use std::fs;
use std::sync::Arc;
use std::sync::Mutex;

mod app_error;
mod config;
mod handlers;
mod trie;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("log4rs.yml", log4rs::config::Deserializers::default()).unwrap();

    info!("Starting service...");

    let config =
        config::Config::from_env().expect("Could not load configuration from environment!");

    let file_content = fs::read_to_string(config.file_name).expect("JSON file not found");

    let trie = Trie::initialize(&file_content, config.suggestion_number).unwrap();
    let shared_trie: Arc<Mutex<Trie>> = Arc::new(Mutex::new(trie));

    // let bind_address: SocketAddr = format!("{}:{}", config.host, config.port)
    //     .parse()
    //     .expect("Unable to parse socket address");

    let server_address = format!("{}:{}", config.host, config.port);
    info!("Starting server at {}", server_address);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(shared_trie.clone()))
            .wrap(middleware::Logger::default())            
            .service(handlers::get_words_match_prefix)
            .service(handlers::get_words_match_empty_prefix)
            .service(handlers::get_words_match_empty_prefix_with_last_slash)
            .service(handlers::increase_popularity)
            .service(health_check)
        // .service(whatsapp_hook)
    })
    .bind(server_address)?
    .run()
    .await
}

#[allow(clippy::unused_async)]
#[get("/health")]
async fn health_check() -> HttpResponse {
    info!("Service is healthy and accepting requests");
    HttpResponse::Ok().json("Service is healthy")
}
