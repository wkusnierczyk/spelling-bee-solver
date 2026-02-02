//! REST API Service for Spelling Bee Solver.
//!
//! Endpoints:
//! - POST /solve: Accepts JSON config, returns word list (or enriched entries with validator).
//! - POST /solve-stream: Like /solve, but streams SSE progress events during validation.
//! - GET /health: Status check.

use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
#[cfg(feature = "validator")]
use sbs::create_validator;
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

    if config.letters.is_none() || config.present.is_none() {
        return HttpResponse::BadRequest().body("Missing letters or present char");
    }

    #[cfg(feature = "validator")]
    let validator_kind = config.validator.clone();
    #[cfg(feature = "validator")]
    let api_key = config.api_key.clone();
    #[cfg(feature = "validator")]
    let validator_url = config.validator_url.clone();

    let solver = Solver::new(config);

    match solver.solve(&data.dictionary) {
        Ok(words) => {
            let mut sorted: Vec<String> = words.into_iter().collect();
            sorted.sort();

            // If a validator is specified, enrich results with definitions and URLs
            #[cfg(feature = "validator")]
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
                return HttpResponse::Ok().json(summary);
            }

            HttpResponse::Ok().json(sorted)
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// SSE endpoint that streams validation progress.
#[cfg(feature = "validator")]
#[post("/solve-stream")]
async fn solve_stream(data: web::Data<AppState>, config_json: web::Json<Config>) -> impl Responder {
    use futures::stream;
    use tokio::sync::mpsc;

    let config = config_json.into_inner();

    if config.letters.is_none() || config.present.is_none() {
        return HttpResponse::BadRequest().body("Missing letters or present char");
    }

    let validator_kind = config.validator.clone();
    let api_key = config.api_key.clone();
    let validator_url = config.validator_url.clone();
    let dictionary = data.dictionary.clone();

    let (tx, rx) = mpsc::unbounded_channel::<String>();

    // Run solving and validation in a blocking thread
    std::thread::spawn(move || {
        let solver = Solver::new(config);

        let words = match solver.solve(&dictionary) {
            Ok(words) => {
                let mut sorted: Vec<String> = words.into_iter().collect();
                sorted.sort();
                sorted
            }
            Err(e) => {
                let _ = tx.send(format!(
                    "data: {}\n\n",
                    serde_json::json!({"error": e.to_string()})
                ));
                return;
            }
        };

        if let Some(kind) = validator_kind {
            let validator =
                match create_validator(&kind, api_key.as_deref(), validator_url.as_deref()) {
                    Ok(v) => v,
                    Err(e) => {
                        let _ = tx.send(format!(
                            "data: {}\n\n",
                            serde_json::json!({"error": e.to_string()})
                        ));
                        return;
                    }
                };

            let summary = validator.validate_words_with_progress(&words, &|done, total| {
                let _ = tx.send(format!(
                    "data: {}\n\n",
                    serde_json::json!({"progress": {"done": done, "total": total}})
                ));
            });

            log::info!(
                "Validated: {} candidates, {} confirmed by {}",
                summary.candidates,
                summary.validated,
                kind.display_name()
            );

            let _ = tx.send(format!(
                "data: {}\n\n",
                serde_json::json!({"result": summary})
            ));
        } else {
            let _ = tx.send(format!(
                "data: {}\n\n",
                serde_json::json!({"result": words})
            ));
        }
    });

    let event_stream = stream::unfold(rx, |mut rx| async move {
        rx.recv()
            .await
            .map(|msg| (Ok::<_, actix_web::Error>(web::Bytes::from(msg)), rx))
    });

    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .insert_header(("Cache-Control", "no-cache"))
        .streaming(event_stream)
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
        let mut app = App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(AppState {
                dictionary: dictionary.clone(),
            }))
            .service(health)
            .service(solve_puzzle);

        #[cfg(feature = "validator")]
        {
            app = app.service(solve_stream);
        }

        app
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
