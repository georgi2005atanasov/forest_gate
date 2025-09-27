use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::features::{onboarding::types::AppState, ws::types::SessionWs};

pub async fn ws_upgrade(
    req: HttpRequest,
    stream: web::Payload, // <-- add this
    state: web::Data<AppState>,
) -> actix_web::Result<HttpResponse> {
    // let (Some(user_id), Some(device_id), Some(session_id)) = (
        // req.cookie("user_id").map(|c| c.value().to_string()),
        // req.cookie("device_id").map(|c| c.value().to_string()),
        // req.cookie("session_id").map(|c| c.value().to_string()),
    // ) else {
    //     return Ok(HttpResponse::BadRequest().body("Missing identifying cookies"));
    // };
    let user_id = "test-user".to_string();
    let device_id = "test-device".to_string();
    let session_id = "test-session".to_string();

    let actor = SessionWs::new(user_id, device_id, session_id, state.redis.clone());
    ws::start(actor, &req, stream) // <-- pass `stream` instead of `HttpResponse::Ok()`
}
