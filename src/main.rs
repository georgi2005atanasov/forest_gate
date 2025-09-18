mod config;
mod swagger;
mod utils;
mod feature;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use swagger::ApiDoc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use config::traits::Env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    // let pg_settings = config::DbSettings::from_env().expect("Failed to load settings");
    // let redis_settings = config::RedisSettings::from_env().expect("Failed to load settings");

    // let db_pool = common::db::create_pool(&pg_settings.postgres_url)
    //     .await
    //     .expect("Failed to create database pool");

    // let redis_pool =
    //     common::redis::create_pool(&redis_settings.redis_url).expect("Failed to create Redis pool");

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            // .app_data(web::Data::new(db_pool.clone()))
            // .app_data(web::Data::new(redis_pool.clone()))
            .wrap(Logger::default())
            .wrap(Cors::permissive()) // Configure as needed
            .service(
                SwaggerUi::new("/swagger/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            .service(
                web::scope("/api/v1")
                    .service(feature::system::health_check)
                    .service(feature::system::version),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
