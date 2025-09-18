use utoipa::OpenApi;
use forest_gate::feature::system::{__path_health_check, __path_version};

#[derive(OpenApi)]
    #[openapi(
        tags(
            (name = "Auth API", description="RUST Actix-web and sqlx API")
        ),
        paths(
            health_check,
            version,
        )
    )]
pub struct ApiDoc;