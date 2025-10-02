mod config;
mod features;
mod infrastructure;
mod swagger;
mod utils;

use std::env;
use std::sync::Arc;

use crate::features::audits::AuditService;
use crate::features::users::UserService;
use crate::utils::error::Error;
use crate::utils::token_service::TokenService;
use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use config::traits::Env;
use features::admin::AdminService;
use features::clients::EmailClient;
use features::onboarding::OnboardingService;
use features::system::ConfigService;
// use forest_gate::seeding;
use infrastructure::persistence::{db, redis};
use swagger::ApiDoc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::features::clients::{MaxMindClient, OpenRouterClient, OrMessage};
use crate::features::onboarding::types::AppState;
use crate::features::onboarding::utils::RateLimiter;

// use crate::features::ws::ws_upgrade;
use crate::utils::crypto::ClientHMAC;
use tokio::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)
        .with_line_number(true)
        .init();

    // let msgs = vec![
    //     OrMessage {
    //         role: "system".into(),
    //         content: "You are a helpful assistant.".into(),
    //     },
    //     OrMessage {
    //         role: "user".into(),
    //         content: "Say hello!".into(),
    //     },
    // ];
    // Create client from env and call
    let openrouter_client = OpenRouterClient::from_env().expect("openrouter client error.");
    // let text = openrouter_client
    //     .chat_text(&msgs, Some("deepseek/deepseek-chat-v3.1:free"), Some(0.2), Some(200))
    //     .await;
    // println!("AI: {}", text.expect("expected some response"));

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

    // seeding::run(&db_pool).await.expect("seeding failed");

    // region services
    let hmac_client = make_hmac_from_env();
    let onboarding_service =
        OnboardingService::new(hmac_client, db_pool.clone(), redis_pool.clone());
    // ATTENTION!!!
    // CONFIG SET notify-keyspace-events Ex
    // CONFIG GET notify-keyspace-events
    // THIS MUST BE EXECUTED BEFORE MAKING SURE YOU COULD FLUSH THE EVENTS FROM REDIS
    // OR JUST ADD TO redis.conf:
    // notify-keyspace-events Ex
    let audit_service = AuditService::new(redis_pool.clone());
    audit_service.spawn_inactivity_flusher(
        &redis_settings.redis_url,
        &pg_settings.database_url,
        openrouter_client.clone(),
    );
    let config_service = Arc::new(ConfigService::new(db_pool.clone(), redis_pool.clone()));
    let issuer = env::var("AUTH_ISSUER").unwrap_or_else(|_| "my-issuer".into());
    let audience = env::var("AUTH_AUDIENCE").unwrap_or_else(|_| "my-audience".into());
    let token_service = Arc::new(
        TokenService::new(config_service.clone(), issuer, audience).expect("init TokenService"),
    );

    let maxmind_client = Arc::new(MaxMindClient::from_env_or_default().expect("load maxmind dbs"));

    let user_service = UserService::new(
        db_pool.clone(),
        token_service.clone(),
        config_service.clone(),
        maxmind_client.clone(),
    );
    let admin_service = AdminService::new(db_pool.clone());
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
            .app_data(web::Data::new(openrouter_client.clone()))
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(redis_pool.clone()))
            .app_data(web::Data::new(config_service.clone()))
            .app_data(web::Data::new(onboarding_service.clone()))
            .app_data(web::Data::new(audit_service.clone()))
            .app_data(web::Data::new(user_service.clone()))
            .app_data(web::Data::new(admin_service.clone()))
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
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
                    .service(features::users::login)
                    .service(features::admin::users)
                    .service(features::audits::audit_init)
                    .service(features::audits::audit_batch),
            )
        // .service(
        //     web::scope("/ws")
        //         .route("", web::get().to(ws_upgrade))
        //         .route("/", web::get().to(ws_upgrade)),
        // )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
