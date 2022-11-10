use crate::app_error::AppError;
use crate::trie::ITrie;
use crate::trie::Trie;
use actix_web::{get, post, web, HttpResponse};
use log::info;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use std::sync::Mutex;

#[allow(clippy::unused_async)]
#[get("/typeahead/{prefix}")]
async fn get_words_match_prefix(
    shared_trie: web::Data<Arc<Mutex<Trie>>>,
    prefix: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    // Trace the message body received as this is the only way found to be able to log the request body and hence figure out any json issues before trying to parse it
    info!("prefix: \n{:?}", prefix);

    let trie = shared_trie.lock().unwrap();
    let result = trie.get_typeahead_words(prefix.into_inner())?;

    Ok(HttpResponse::Ok().json(result))
}

#[derive(Deserialize, Serialize)]
struct IncreasePopularityPayload {
    pub name: String,
}

#[allow(clippy::unused_async)]
#[post("/typeahead")]
async fn increase_popularity(
    shared_trie: web::Data<Arc<Mutex<Trie>>>,
    payload: web::Bytes,
) -> Result<HttpResponse, AppError> {
    // Trace the message body received as this is the only way found to be able to log the request body and hence figure out any json issues before trying to parse it
    info!("PAYLOAD: \n{:?}", payload);

    let payload = String::from_utf8(payload.to_vec()).map_err(|_e| AppError::BadRequest)?;
    let deserialized_payload = serde_json::from_str::<IncreasePopularityPayload>(&payload)
        .map_err(|_e| AppError::BadRequest)?;

    let mut trie = shared_trie.lock().unwrap();
    let result = trie.increase_popularity(deserialized_payload.name)?;

    let json = json! ({ "name": result.word, "times": result.popularity });

    Ok(HttpResponse::Created().json(json))
}
