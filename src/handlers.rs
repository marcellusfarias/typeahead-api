use crate::app_error::AppError;
use crate::trie::Trie;
use actix_web::{get, post, web, HttpResponse};
use std::sync::Arc;
use std::sync::Mutex;

// #[allow(clippy::unused_async)]
#[get("/typeahead/{prefix}")]
async fn get_words_match_prefix(
    trie: web::Data<Arc<Mutex<Trie>>>,
    prefix: web::Query<String>,
) -> Result<HttpResponse, AppError> {
    // Trace the message body received as this is the only way found to be able to log the request body and hence figure out any json issues before trying to parse it
    // trace!("PAYLOAD: \n{:?}", payload);

    Ok(HttpResponse::Ok().into())
}

#[post("/typeahead")]
async fn increase_popularity(
    trie: web::Data<Arc<Mutex<Trie>>>,
    payload: web::Bytes,
) -> Result<HttpResponse, AppError> {
    // Trace the message body received as this is the only way found to be able to log the request body and hence figure out any json issues before trying to parse it
    // trace!("PAYLOAD: \n{:?}", payload);

    Ok(HttpResponse::Ok().into())
}
