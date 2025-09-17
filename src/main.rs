mod config;
mod controllers;
mod data;
mod middlewares;
mod models;
mod repositories;
mod services;
mod swagger;
mod utils;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use swagger::ApiDoc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let settings = config::app_settings::Settings::from_env().expect("Failed to load settings");

    let db_pool = data::connection::create_pool(&settings.database_url)
        .await
        .expect("Failed to create database pool");

    let redis_pool =
        config::redis::create_pool(&settings.redis_url).expect("Failed to create Redis pool");

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(redis_pool.clone()))
            .wrap(Logger::default())
            .wrap(Cors::permissive()) // Configure as needed
            .service(
                SwaggerUi::new("/swagger/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            .service(
                web::scope("/api/v1")
                    .service(controllers::utils_controller::health_check)
                    .service(controllers::utils_controller::version),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
