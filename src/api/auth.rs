use crate::api::models::ApiResponse;
use base64::{engine::general_purpose::STANDARD as base64, Engine};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::{Responder, Response};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseAuth {
    pub db_name: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct AuthError {
    // Message that will be displayed when there is a failure in authentication
    pub message: String,
}

impl<'r> Responder<'r, 'static> for AuthError {
    fn respond_to(self, request: &'r Request<'_>) -> Result<Response<'static>, Status> {
        let response = ApiResponse::<()>::error(self.message);
        Ok(Json(response).respond_to(request)?)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for DatabaseAuth {
    type Error = AuthError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_header = match request.headers().get_one("Authorization") {
            Some(header) => header,
            None => {
                return Outcome::Error((
                    Status::Unauthorized,
                    AuthError {
                        message: "Authorization header not provided".to_string(),
                    },
                ))
            }
        };

        if !auth_header.starts_with("Basic ") {
            return Outcome::Error((
                Status::Unauthorized,
                AuthError {
                    message: "Invalid authorization type. Expected Basic".to_string(),
                },
            ));
        }

        let credentials = match base64.decode(auth_header.trim_start_matches("Basic ")) {
            Ok(decoded) => match String::from_utf8(decoded) {
                Ok(string) => string,
                Err(_) => {
                    return Outcome::Error((
                        Status::Unauthorized,
                        AuthError {
                            message: "Invalid base64 encoding".to_string(),
                        },
                    ))
                }
            },
            Err(_) => {
                return Outcome::Error((
                    Status::Unauthorized,
                    AuthError {
                        message: "Invalid base64 encoding".to_string(),
                    },
                ))
            }
        };

        let parts: Vec<&str> = credentials.split(':').collect();
        if parts.len() != 3 {
            return Outcome::Error((
                Status::Unauthorized,
                AuthError {
                    message: "Invalid credentials format. Expected 'database:username:password'"
                        .to_string(),
                },
            ));
        }

        Outcome::Success(DatabaseAuth {
            db_name: parts[0].to_string(),
            username: parts[1].to_string(),
            password: parts[2].to_string(),
        })
    }
}
