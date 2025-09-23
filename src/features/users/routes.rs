use std::u64;

use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use validator::Validate;

use crate::features::{
    clients::EmailClient,
    users::{types::UserLoginReq, UserService},
};

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
    if let Err(errors) = payload.validate() {
        return Ok(HttpResponse::BadRequest().json(errors));
    }

    // 1) cookie check (decoded signature = actual email)

    // 2) user creation

    // 3) user_devices - we add a record in here - we get the __Host-device_id cookie value

    // 4) generate jwt and automatically login the user.

    let mut resp = HttpResponse::Ok();
    // resp.cookie(cookie);
    Ok(resp.finish())
}
