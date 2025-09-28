use actix_web::{post, web, HttpResponse, Result};
use serde::Serialize;
use utoipa::ToSchema;

use crate::features::users::types::UserDto;

use super::types::AllUsersDto;
use super::AdminService;

#[derive(Serialize, ToSchema)]
struct UsersPage<T> {
    items: Vec<T>,
    total: i64,
}

#[utoipa::path(
    post,
    path = "/admin/app-users",
    tag = "admin",
    request_body = AllUsersDto,
    responses(
        (status = 200, description = "List users with filters and pagination", body = UsersPage<UserDto>)
    )
)]
#[post("/admin/app-users")]
pub async fn users(
    admin_service: web::Data<AdminService>,
    body: Result<web::Json<AllUsersDto>, actix_web::Error>,
) -> Result<HttpResponse> {
    let body = body.map_err(|e| {
        tracing::error!("Invalid request body: {:?}", e);
        e
    })?;

    let (items, total) = admin_service.all(body.into_inner()).await.map_err(|e| {
        tracing::error!("DB error: {:?}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;

    Ok(HttpResponse::Ok().json(UsersPage { items, total }))
}
