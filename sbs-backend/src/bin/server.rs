//! REST API Service for Spelling Bee Solver.
//!
//! Endpoints:
//! - POST /solve: Accepts JSON config, returns word list (or enriched entries with validator).
//! - GET /health: Status check.

use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use sbs::{create_validator, Config, Dictionary, Solver};
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

    if config.letters.is_none() || config.present.is_none() {
        return HttpResponse::BadRequest().body("Missing letters or present char");
    }

    let validator_kind = config.validator.clone();
    let api_key = config.api_key.clone();
    let validator_url = config.validator_url.clone();

    let solver = Solver::new(config);

    match solver.solve(&data.dictionary) {
        Ok(words) => {
            let mut sorted: Vec<String> = words.into_iter().collect();
            sorted.sort();

            // If a validator is specified, enrich results with definitions and URLs
            if let Some(kind) = validator_kind {
                let validator =
                    match create_validator(&kind, api_key.as_deref(), validator_url.as_deref()) {
                        Ok(v) => v,
                        Err(e) => {
                            return HttpResponse::BadRequest().body(e.to_string());
                        }
                    };

                let summary = validator.validate_words(&sorted);
                log::info!(
                    "Validated: {} candidates, {} confirmed by {}",
                    summary.candidates,
                    summary.validated,
                    kind.display_name()
                );
                HttpResponse::Ok().json(summary)
            } else {
                HttpResponse::Ok().json(sorted)
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

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
            .wrap(Cors::permissive())
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
