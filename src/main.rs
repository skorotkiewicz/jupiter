mod agent;
mod auth;
mod db;
mod models;
mod routes;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, middleware};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "jupiter.db".to_string());

    log::info!("🪐 Jupiter starting on {}:{}", host, port);
    log::info!("📂 Database: {}", db_path);

    let database = db::Database::new(&db_path).expect("Failed to initialize database");
    let db_data = web::Data::new(database);

    let llm_agent = agent::LlmAgent::new();
    let agent_data = web::Data::new(llm_agent);

    log::info!("🤖 LLM Agent initialized");
    log::info!("🚀 Server ready at http://{}:{}", host, port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(db_data.clone())
            .app_data(agent_data.clone())
            .app_data(web::JsonConfig::default().limit(1024 * 1024))
            // Auth routes
            .route("/v1/auth/register", web::post().to(auth::register))
            .route("/v1/auth/login", web::post().to(auth::login))
            .route("/v1/auth/profile", web::get().to(auth::get_profile))
            .route("/v1/auth/profile", web::put().to(auth::update_profile))
            // Chat with personal agent
            .route("/v1/chat", web::get().to(routes::get_chat_history))
            .route("/v1/chat", web::post().to(routes::send_message))
            // Agent profile (what agent knows)
            .route("/v1/agent/profile", web::get().to(routes::get_agent_profile))
            .route("/v1/agent/profile/update", web::post().to(routes::trigger_profile_update))
            // Matching
            .route("/v1/matching/trigger", web::post().to(routes::trigger_matching))
            .route("/v1/matches", web::get().to(routes::get_matches))
            // Notifications
            .route("/v1/notifications", web::get().to(routes::get_notifications))
            .route("/v1/notifications/unread", web::get().to(routes::get_unread_count))
            .route("/v1/notifications/{id}/read", web::post().to(routes::mark_notification_read))
            // Direct messages
            .route("/v1/messages/{match_id}", web::get().to(routes::get_direct_messages))
            .route("/v1/messages/{match_id}", web::post().to(routes::send_direct_message))
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
