mod config;
mod features;
mod infrastructure;
mod swagger;
mod utils;

use std::env;
use std::sync::Arc;

use crate::features::users::UserService;
use crate::utils::token_service::TokenService;
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

use crate::features::clients::MaxMindClient;
use crate::features::onboarding::types::AppState;
use crate::features::onboarding::utils::RateLimiter;
use crate::features::ws::job::flush_worker;
// use crate::features::ws::ws_upgrade;
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

    tokio::spawn(flush_worker(redis_pool.clone()));

    // region services
    let hmac_client = make_hmac_from_env();
    let onboarding_service =
        OnboardingService::new(hmac_client, db_pool.clone(), redis_pool.clone());

    let ec_priv_path = env::var("AUTH_EC_PRIVATE_PEM_PATH").expect("AUTH_EC_PRIVATE_PEM_PATH");
    let ec_pub_path = env::var("AUTH_EC_PUBLIC_PEM_PATH").expect("AUTH_EC_PUBLIC_PEM_PATH");
    let ec_priv = std::fs::read(ec_priv_path).expect("read private key");
    let ec_pub = std::fs::read(ec_pub_path).expect("read public key");
    let issuer = env::var("AUTH_ISSUER").unwrap_or_else(|_| "my-issuer".into());
    let audience = env::var("AUTH_AUDIENCE").unwrap_or_else(|_| "my-audience".into());

    let config_service = Arc::new(ConfigService::new(db_pool.clone(), redis_pool.clone()));

    let token_service = Arc::new(
        TokenService::new(config_service.clone(), &ec_priv, &ec_pub, issuer, audience)
            .expect("init TokenService"),
    );

    let maxmind_client = Arc::new(MaxMindClient::from_env_or_default().expect("load maxmind dbs"));

    let user_service = UserService::new(
        db_pool.clone(),
        token_service.clone(),
        config_service.clone(),
        maxmind_client.clone(),
    );
    // endregion services

    // region rate Limiting
    let limiter = RateLimiter::new(redis_pool.clone());
    let app_state = web::Data::new(AppState {
        limiter: Mutex::new(limiter),
        redis: redis_pool.clone(),
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
            .app_data(app_state.clone())
            .app_data(web::Data::new(email_client.clone()))
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(redis_pool.clone()))
            .app_data(web::Data::new(config_service.clone()))
            .app_data(web::Data::new(onboarding_service.clone()))
            .app_data(web::Data::new(user_service.clone()))
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
                    .service(features::onboarding::otp_verification)
                    .service(features::onboarding::user_details)
                    .service(features::users::login),
            )
        // .route("/ws", actix_web::web::get().to(ws_upgrade))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
