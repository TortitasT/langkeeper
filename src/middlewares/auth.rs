use actix_session::SessionExt;
use actix_web::FromRequest;

use crate::jwt;

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
                    return futures::future::err(actix_web::error::ErrorUnauthorized(
                        "Unauthorized",
                    ))
                }
            },
            Err(_) => {
                return futures::future::err(actix_web::error::ErrorUnauthorized("Unauthorized"))
            }
        };

        let claims = match jwt::decode_auth_jwt(&token) {
            Ok(decoded) => decoded,
            Err(_) => {
                return futures::future::err(actix_web::error::ErrorUnauthorized("Unauthorized"))
            }
        };

        futures::future::ok(AuthMiddleware {
            user_id: claims.sub,
        })
    }
}
