use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;

mod config;
mod controllers;
mod services;
mod repositories;
mod models;
mod middlewares;
mod data;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let settings = config::app_settings::Settings::from_env().expect("Failed to load settings");
    
    // Setup database connection pool
    let db_pool = data::connection::create_pool(&settings.database_url)
        .await
        .expect("Failed to create database pool");
    
    // Setup Redis connection pool
    let redis_pool = config::redis::create_pool(&settings.redis_url)
        .expect("Failed to create Redis pool");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(redis_pool.clone()))
            .wrap(Logger::default())
            .wrap(Cors::permissive()) // Configure as needed
            .service(
                web::scope("/api/v1")
                    .service(controllers::utils_controller::health_check)
                    .service(controllers::utils_controller::version)
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}