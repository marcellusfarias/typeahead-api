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

// This function is needed according to this: https://github.com/actix/actix-web/issues/1054
#[allow(clippy::unused_async)]
#[get("/typeahead/")]
async fn get_words_match_empty_prefix(
    shared_trie: web::Data<Arc<Mutex<Trie>>>,
) -> Result<HttpResponse, AppError> {
    // Trace the message body received as this is the only way found to be able to log the request body and hence figure out any json issues before trying to parse it
    info!("prefix is empty");

    let trie = shared_trie.lock().unwrap();
    let result = trie.get_typeahead_words("".to_string())?;

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

#[cfg(test)]
mod tests {
    use http;
    use httptest::{matchers::*, responders::*, Expectation, Server};
    use crate::trie::{ITrie, Trie};
    use std::sync::Arc;
    use std::sync::Mutex;
    use crate::handlers::{get_words_match_prefix, increase_popularity, get_words_match_empty_prefix};
    use actix_web::{body::Body, test, web::Bytes, App};
    use actix_web::http::StatusCode;

    fn get_default_trie() -> Trie {
        let file_content =
        "{\"A-b\": 23, \"Aar\":361,\"Aari\":151,\"Aba\":608,\"Abag\":704, \"Abe\": 300, \"Ba\": 5, \"Bah\": 5, \"Be\": 50, \"Bc\": 50}";
        Trie::initialize(&file_content, 5).unwrap()
    }

    #[actix_rt::test]
    async fn t_get_words_match_prefix_prefix_not_included() {
        let trie = get_default_trie();
        let shared_trie: Arc<Mutex<Trie>> = Arc::new(Mutex::new(trie));
        
        let app = App::new().data(shared_trie).service(get_words_match_prefix);
        let mut app = test::init_service(app).await;

        let req = test::TestRequest::get().uri("/typeahead/A").to_request();
        println!("req: {:?}", req);

        let mut resp = test::call_service(&mut app, req).await;
        println!("response: {:?}", &resp);

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.take_body().as_ref().unwrap(),
            &Body::from_slice(b"[{\"name\":\"Abag\",\"times\":704},{\"name\":\"Aba\",\"times\":608},{\"name\":\"Aar\",\"times\":361},{\"name\":\"Abe\",\"times\":300},{\"name\":\"Aari\",\"times\":151}]")
        );
    }

    #[actix_rt::test]
    async fn t_get_words_match_prefix_exact_match_prefix() {
        let trie = get_default_trie();
        let shared_trie: Arc<Mutex<Trie>> = Arc::new(Mutex::new(trie));
        
        let app = App::new().data(shared_trie).service(get_words_match_prefix);
        let mut app = test::init_service(app).await;

        let req = test::TestRequest::get().uri("/typeahead/Aba").to_request();
        println!("req: {:?}", req);

        let mut resp = test::call_service(&mut app, req).await;
        println!("response: {:?}", &resp);

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.take_body().as_ref().unwrap(),
            &Body::from_slice(b"[{\"name\":\"Aba\",\"times\":608},{\"name\":\"Abag\",\"times\":704}]")
        );
    }

    #[actix_rt::test]
    async fn t_get_words_match_prefix_words_with_same_popularity() {
        let mut trie = get_default_trie();
        trie.suggestion_number = 2;
        let shared_trie: Arc<Mutex<Trie>> = Arc::new(Mutex::new(trie));
        
        let app = App::new().data(shared_trie).service(get_words_match_prefix);
        let mut app = test::init_service(app).await;

        let req = test::TestRequest::get().uri("/typeahead/B").to_request();
        println!("req: {:?}", req);

        let mut resp = test::call_service(&mut app, req).await;
        println!("response: {:?}", &resp);

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.take_body().as_ref().unwrap(),
            &Body::from_slice(b"[{\"name\":\"Bc\",\"times\":50},{\"name\":\"Be\",\"times\":50}]")
        );
    }

    #[actix_rt::test]
    async fn t_get_words_match_prefix_case_insensitive() {
        let mut trie = get_default_trie();
        trie.suggestion_number = 2;
        let shared_trie: Arc<Mutex<Trie>> = Arc::new(Mutex::new(trie));
        
        let app = App::new().data(shared_trie).service(get_words_match_prefix);
        let mut app = test::init_service(app).await;

        let req = test::TestRequest::get().uri("/typeahead/AA").to_request();
        println!("req: {:?}", req);

        let mut resp = test::call_service(&mut app, req).await;
        println!("response: {:?}", &resp);

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.take_body().as_ref().unwrap(),
            &Body::from_slice(b"[{\"name\":\"Aar\",\"times\":361},{\"name\":\"Aari\",\"times\":151}]")
        );
    }

    #[actix_rt::test]
    async fn t_get_words_match_prefix_no_words_match_prefix() {
        let mut trie = get_default_trie();
        trie.suggestion_number = 2;
        let shared_trie: Arc<Mutex<Trie>> = Arc::new(Mutex::new(trie));
        
        let app = App::new().data(shared_trie).service(get_words_match_prefix);
        let mut app = test::init_service(app).await;

        let req = test::TestRequest::get().uri("/typeahead/Brazil").to_request();
        println!("req: {:?}", req);

        let mut resp = test::call_service(&mut app, req).await;
        println!("response: {:?}", &resp);

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.take_body().as_ref().unwrap(),
            &Body::from_slice(b"[]")
        );
    }

    #[actix_rt::test]
    async fn t_get_words_match_prefix_return_only_prefix_and_special_character() {
        let mut trie = get_default_trie();
        trie.suggestion_number = 2;
        let shared_trie: Arc<Mutex<Trie>> = Arc::new(Mutex::new(trie));
        
        let app = App::new().data(shared_trie).service(get_words_match_prefix);
        let mut app = test::init_service(app).await;

        let req = test::TestRequest::get().uri("/typeahead/A-b").to_request();
        println!("req: {:?}", req);

        let mut resp = test::call_service(&mut app, req).await;
        println!("response: {:?}", &resp);

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.take_body().as_ref().unwrap(),
            &Body::from_slice(b"[{\"name\":\"A-b\",\"times\":23}]")
        );
    }

    #[actix_rt::test]
    async fn t_get_words_match_prefix_prefix_empty() {
        let trie = get_default_trie();
        let shared_trie: Arc<Mutex<Trie>> = Arc::new(Mutex::new(trie));
        
        let app = App::new().data(shared_trie).service(get_words_match_empty_prefix);
        let mut app = test::init_service(app).await;

        let req = test::TestRequest::get().uri("/typeahead/").to_request();
        println!("req: {:?}", req);

        let mut resp = test::call_service(&mut app, req).await;
        println!("response: {:?}", &resp);

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.take_body().as_ref().unwrap(),
            &Body::from_slice(b"[{\"name\":\"Abag\",\"times\":704},{\"name\":\"Aba\",\"times\":608},{\"name\":\"Aar\",\"times\":361},{\"name\":\"Abe\",\"times\":300},{\"name\":\"Aari\",\"times\":151}]")
        );
    }

    #[actix_rt::test]
    async fn t_handler_increase_popularity_word_exists() {
        let trie = get_default_trie();
        let shared_trie: Arc<Mutex<Trie>> = Arc::new(Mutex::new(trie));
        
        let app = App::new().data(shared_trie).service(increase_popularity);
        let mut app = test::init_service(app).await;

        let data = Bytes::from("{\"name\": \"Aar\"}");
        let req = test::TestRequest::post().uri("/typeahead").set_payload(data).to_request();
        println!("req: {:?}", req);

        let mut resp = test::call_service(&mut app, req).await;
        println!("response: {:?}", &resp);

        assert_eq!(resp.status(), StatusCode::CREATED);
        assert_eq!(
            resp.take_body().as_ref().unwrap(),
            &Body::from_slice(b"{\"name\":\"Aar\",\"times\":362}")
        );
    }

    #[actix_rt::test]
    async fn t_handler_increase_popularity_word_exists_case_sensitive() {
        let trie = get_default_trie();
        let shared_trie: Arc<Mutex<Trie>> = Arc::new(Mutex::new(trie));
        
        let app = App::new().data(shared_trie).service(increase_popularity);
        let mut app = test::init_service(app).await;

        let data = Bytes::from("{\"name\": \"AaR\"}");
        let req = test::TestRequest::post().uri("/typeahead").set_payload(data).to_request();
        println!("req: {:?}", req);

        let mut resp = test::call_service(&mut app, req).await;
        println!("response: {:?}", &resp);

        assert_eq!(resp.status(), StatusCode::CREATED);
        assert_eq!(
            resp.take_body().as_ref().unwrap(),
            &Body::from_slice(b"{\"name\":\"Aar\",\"times\":362}")
        );
    }

    #[actix_rt::test]
    async fn t_handler_increase_popularity_word_does_not_exists() {
        let trie = get_default_trie();
        let shared_trie: Arc<Mutex<Trie>> = Arc::new(Mutex::new(trie));
        
        let app = App::new().data(shared_trie).service(increase_popularity);
        let mut app = test::init_service(app).await;

        let data = Bytes::from("{\"name\": \"Abcd\"}");
        let req = test::TestRequest::post().uri("/typeahead").set_payload(data).to_request();
        println!("req: {:?}", req);

        let resp = test::call_service(&mut app, req).await;
        println!("response: {:?}", &resp);

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
