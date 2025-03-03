use crate::api::models::ApiResponse;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response};
use rocket::serde::json::Json;

pub struct UnauthorizedError;

impl<'r> Responder<'r, 'static> for UnauthorizedError {
    fn respond_to(self, _: &'r Request<'_>) -> Result<Response<'static>, Status> {
        let response = ApiResponse::<()>::error("Unauthorized");
        Ok(Json(response).respond_to(_)?)
    }
}

pub fn unauthorized() -> UnauthorizedError {
    UnauthorizedError
}
