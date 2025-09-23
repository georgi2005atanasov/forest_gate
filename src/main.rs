mod config;
mod features;
mod infrastructure;
mod swagger;
mod utils;

use std::env;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use config::traits::Env;
use features::clients::EmailClient;
use features::onboarding::OnboardingService;
use features::system::ConfigService;
use infrastructure::persistence::{db, redis};
use swagger::ApiDoc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::features::onboarding::types::AppState;
use crate::features::onboarding::utils::RateLimiter;
use crate::features::users::UserService;
use crate::utils::crypto::ClientHMAC;
use tokio::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    // region settings
    let email_client = EmailClient::from_env().expect("email client config");
    let pg_settings = config::DbSettings::from_env().expect("Failed to load settings");
    let redis_settings = config::RedisSettings::from_env().expect("Failed to load settings");
    // endregion settings

    // region persistense
    let db_pool = db::create_pool(&pg_settings.database_url)
        .await
        .expect("Failed to create database pool");
    let redis_pool =
        redis::create_pool(&redis_settings.redis_url).expect("Failed to create Redis pool");
    // endregion persistence

    // region services
    let hmac_client = make_hmac_from_env();
    let config_service = ConfigService::new(db_pool.clone(), redis_pool.clone());
    let onboarding_service =
        OnboardingService::new(hmac_client, db_pool.clone(), redis_pool.clone());
    let user_service = UserService::new(db_pool.clone(), redis_pool.clone());
    // endregion services

    // region rate Limiting
    let limiter = RateLimiter::new(redis_pool.clone());
    let app_state = web::Data::new(AppState {
        limiter: Mutex::new(limiter),
    });
    // endregion rate Limiting

    fn make_hmac_from_env() -> ClientHMAC {
        let hex_key = env::var("VISITOR_HMAC_KEY")
            .expect("VISITOR_HMAC_KEY must be set (hex, e.g. `openssl rand -hex 32`)");
        ClientHMAC::from_hex_key(&hex_key).expect("invalid VISITOR_HMAC_KEY hex")
    }

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(email_client.clone()))
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(redis_pool.clone()))
            .app_data(web::Data::new(config_service.clone()))
            .app_data(web::Data::new(onboarding_service.clone()))
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:5173")
                    .allowed_methods(vec!["GET", "POST", "PUT", "OPTIONS"])
                    .allow_any_header()
                    .supports_credentials(),
            ) // should be changed for production!!!
            .service(
                SwaggerUi::new("/swagger/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            .service(
                web::scope("")
                    .service(features::system::health)
                    .service(features::system::version)
                    .service(features::system::config)
                    .service(features::system::update_config)
                    .service(features::onboarding::preparation)
                    .service(features::onboarding::with_email)
                    .service(features::onboarding::otp_verification),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
