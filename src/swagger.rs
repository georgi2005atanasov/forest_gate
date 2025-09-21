use forest_gate::feature::system::{
    __path_config, __path_health, __path_update_config, __path_version,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
        tags(
            (name = "Auth API", description="RUST Actix-web and sqlx API")
        ),
        paths(
            health,
            version,
            config,
            update_config
        )
    )]
pub struct ApiDoc;
