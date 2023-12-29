use actix_session::SessionExt;
use actix_web::FromRequest;
use actix_web::{http::StatusCode, HttpResponse};
use derive_more::{Display, Error};

use crate::jwt;

#[derive(Debug, Display, Error)]
enum AuthError {
    #[display(fmt = "Authentication error")]
    AuthError,
}

impl actix_web::error::ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(("HX-Redirect", "/"))
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            AuthError::AuthError => StatusCode::UNAUTHORIZED,
        }
    }
}

fn build_unauthorized_response() -> futures::future::Ready<Result<AuthMiddleware, actix_web::Error>>
{
    futures::future::err(AuthError::AuthError.into())
}

pub struct AuthMiddleware {
    pub user_id: i32,
}

impl FromRequest for AuthMiddleware {
    type Error = actix_web::Error;
    type Future = futures::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let session = req.get_session();

        let token = match session.get::<String>("token") {
            Ok(token) => match token {
                Some(token) => token,
                None => {
                    return build_unauthorized_response();
                }
            },
            Err(_) => {
                return build_unauthorized_response();
            }
        };

        let claims = match jwt::decode_auth_jwt(&token) {
            Ok(decoded) => decoded,
            Err(_) => {
                return build_unauthorized_response();
            }
        };

        futures::future::ok(AuthMiddleware {
            user_id: claims.sub,
        })
    }
}
