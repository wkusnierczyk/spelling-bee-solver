//! REST API Service for Spelling Bee Solver.
//!
//! Endpoints:
//! - POST /solve: Accepts JSON config, returns word list.
//! - GET /health: Status check.

use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use sbs::{Config, Dictionary, Solver};
use std::env;
use std::sync::Arc;

/// Shared application state
struct AppState {
    dictionary: Arc<Dictionary>,
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[post("/solve")]
async fn solve_puzzle(data: web::Data<AppState>, config_json: web::Json<Config>) -> impl Responder {
    let config = config_json.into_inner();

    // We run the solver logic. Since it's CPU bound, for very large dictionaries
    // or heavy load, we might use web::block, but Trie traversal is usually fast enough (ms).

    // Validate basics
    if config.letters.is_none() || config.present.is_none() {
        return HttpResponse::BadRequest().body("Missing letters or present char");
    }

    let solver = Solver::new(config);

    match solver.solve(&data.dictionary) {
        Ok(words) => {
            let mut sorted: Vec<String> = words.into_iter().collect();
            sorted.sort();
            HttpResponse::Ok().json(sorted)
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Load dictionary path from env or default
    // Note: In a real deployment, we might pass this via CLI args to the server binary too.
    let dict_path = env::var("SBS_DICT").unwrap_or_else(|_| "data/dictionary.txt".to_string());

    log::info!("Loading dictionary from: {}", dict_path);
    let dictionary = match Dictionary::from_file(&dict_path) {
        Ok(d) => Arc::new(d),
        Err(e) => {
            log::error!("Failed to load dictionary: {}", e);
            std::process::exit(1);
        }
    };

    log::info!("Starting server at http://0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive()) // Allow requests from GUI
            .app_data(web::Data::new(AppState {
                dictionary: dictionary.clone(),
            }))
            .service(health)
            .service(solve_puzzle)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
