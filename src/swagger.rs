use forest_gate::features::{
    admin::__path_users,
    onboarding::{
        __path_otp_verification, __path_preparation, __path_user_details, __path_with_email,
    },
    system::{__path_config, __path_health, __path_update_config, __path_version},
    users::__path_login,
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
        update_config,
        preparation,
        otp_verification,
        user_details,
        with_email,
        login,
        users
    )
)]
pub struct ApiDoc;
