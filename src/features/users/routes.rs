use actix_web::{post, web, HttpRequest, Responder};
use validator::Validate;

use crate::features::users::{types::UserLoginReq, UserService};

#[utoipa::path(
    post,
    path="/users/login",
    tag="users",
    responses(
        (status = 200, description = "Prepare user for authentication"),
        (status = 403, description = "Forbidden"),
        (status = 429, description = "Too many requests"),
    )
)]
#[post("/users/login")]
pub async fn login(
    req: HttpRequest,
    payload: web::Json<UserLoginReq>,
    user_service: web::Data<UserService>,
) -> actix_web::Result<impl Responder> {
    println!("{:?}", payload);
    if let Err(errors) = payload.validate() {
        return Ok(actix_web::HttpResponse::BadRequest().json(errors));
    }
    user_service.login(&req, &payload).await
}
