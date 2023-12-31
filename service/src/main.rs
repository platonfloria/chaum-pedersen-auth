use std::{collections::HashMap, sync::Arc, time::Duration};

use http::header::HeaderName;
use eyre::Result;
use k256::{elliptic_curve::{PrimeField, point::DecompressPoint, subtle::Choice, generic_array::GenericArray}, AffinePoint, Scalar};
use num_bigint::BigUint;
use protocol::{ChaumPedersen, ChaumPedersenK256};
use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};
use tonic_web::GrpcWebLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use uuid::Uuid;

mod pb2 {
    tonic::include_proto!("zkp_auth");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("descriptor");
}



#[derive(Debug)]
enum Credentials {
    Exp((BigUint, BigUint)),
    K256((AffinePoint, AffinePoint)),
}


#[derive(Debug)]
struct Session {
    id: Option<Uuid>,
    user: String,
    r: Credentials,
    c: BigUint,
}


#[derive(Debug)]
struct User {
    name: String,
    y: Credentials,
}


pub struct API {
    users: Arc<Mutex<HashMap<String, User>>>,
    sessions: Arc<Mutex<HashMap<Uuid, Session>>>,
    protocol: ChaumPedersen,
    protocol_k256: ChaumPedersenK256,
}

impl API {
    fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            protocol: ChaumPedersen::new(
                std::env::var("P").expect("P env var must be set.").parse().expect("P is not an integer"),
                std::env::var("Q").expect("Q env var must be set.").parse().expect("Q is not an integer"),
                std::env::var("G").expect("G env var must be set.").parse().expect("G is not an integer"),
                std::env::var("H").expect("H env var must be set.").parse().expect("H is not an integer"),
            ),
            protocol_k256: ChaumPedersenK256::new(
                std::env::var("K256_H_OFFSET").expect("K256_H_OFFSET env var must be set.").parse().expect("K256_H_OFFSET is not an integer"),
            ),
        }
    }
}

#[tonic::async_trait]
impl pb2::auth_server::Auth for API {
    async fn register(&self, request: Request<pb2::RegisterRequest>) -> Result<Response<pb2::RegisterResponse>, Status> {
        let request = request.get_ref();
        let y1 = BigUint::from_bytes_be(&request.y1);
        let y2 = BigUint::from_bytes_be(&request.y2);
        let mut users = self.users.lock().await;
        if users.contains_key(&request.user) {
            Err(Status::already_exists("user already is registered"))
        } else {
            log::info!("register {} with (y1={}, y2={})", request.user, y1, y2);
            users.insert(request.user.clone(), User {
                name: request.user.clone(),
                y: Credentials::Exp((y1, y2)),
            });
            Ok(Response::new(pb2::RegisterResponse {}))
        }
    }

    async fn create_authentication_challenge(&self, request: Request<pb2::AuthenticationChallengeRequest>) -> Result<Response<pb2::AuthenticationChallengeResponse>, Status> {
        let request = request.get_ref();
        let r1 = BigUint::from_bytes_be(&request.r1);
        let r2 = BigUint::from_bytes_be(&request.r2);
        if let Some(user) = self.users.lock().await.get_mut(&request.user) {
            log::info!("create_authentication_challenge for user {} with (r1={}, r2={})", user.name, r1, r2);
            let auth_id = Uuid::new_v4();
            let c = self.protocol.challenge();
            self.sessions.lock().await.insert(auth_id, Session {
                id: None,
                user: user.name.clone(),
                r: Credentials::Exp((r1, r2)),
                c: c.clone(),
            });
            Ok(Response::new(pb2::AuthenticationChallengeResponse {
                auth_id: auth_id.to_string(),
                c: c.to_bytes_be(),
            }))
        } else {
            Err(Status::not_found("user not found"))
        }
    }

    async fn verify_authentication(&self, request: Request<pb2::AuthenticationAnswerRequest>) -> Result<Response<pb2::AuthenticationAnswerResponse>, Status> {
        let request = request.get_ref();
        let s = BigUint::from_bytes_be(&request.s);
        log::info!("verify_authentication {} with (s={})", request.auth_id, s);
        if let Some(session) = self.sessions.lock().await.get_mut(&Uuid::parse_str(&request.auth_id).expect("invalid auth id")) {
            let user = &self.users.lock().await[&session.user];
            if let (Credentials::Exp((y1, y2)), Credentials::Exp((r1, r2))) = (&user.y, &session.r) {
                if self.protocol.verify(y1, y2, r1, r2, &session.c, &s) {
                    let session_id = Uuid::new_v4();
                    session.id = Some(session_id);
                    Ok(Response::new(pb2::AuthenticationAnswerResponse {
                        session_id: session_id.to_string(),
                    }))
                } else {
                    Err(Status::unauthenticated("invalid password"))
                }
            } else {
                Err(Status::unauthenticated("invalid protocol"))
            }
        } else {
            Err(Status::not_found("auth not found"))
        }
    }


    async fn k256_register(&self, request: Request<pb2::K256RegisterRequest>) -> Result<Response<pb2::K256RegisterResponse>, Status> {
        let request = request.get_ref();
        if let (Some(y1), Some(y2)) = (&request.y1, &request.y2) {
            if let (Some(y1), Some(y2)) = (
                AffinePoint::decompress(y1.x.as_slice().into(), Choice::from(y1.is_y_odd as u8)).into(),
                AffinePoint::decompress(y2.x.as_slice().into(), Choice::from(y2.is_y_odd as u8)).into(),
            ) {
                let mut users = self.users.lock().await;
                if users.contains_key(&request.user) {
                    Err(Status::already_exists("user already is registered"))
                } else {
                    log::info!("register {} with (y1={:?}, y2={:?})", request.user, y1, y2);
                    users.insert(request.user.clone(), User {
                        name: request.user.clone(),
                        y: Credentials::K256((y1, y2)),
                    });
                    Ok(Response::new(pb2::K256RegisterResponse {}))
                }
            } else {
                Err(Status::invalid_argument("y1 point or y2 point is invalid"))
            }
        } else {
            Err(Status::invalid_argument("y1 or y2 is missing"))
        }
    }

    async fn k256_create_authentication_challenge(&self, request: Request<pb2::K256AuthenticationChallengeRequest>) -> Result<Response<pb2::K256AuthenticationChallengeResponse>, Status> {
        let request = request.get_ref();
        if let (Some(r1), Some(r2)) = (&request.r1, &request.r2) {
            if let (Some(r1), Some(r2)) = (
                AffinePoint::decompress(r1.x.as_slice().into(), Choice::from(r1.is_y_odd as u8)).into(),
                AffinePoint::decompress(r2.x.as_slice().into(), Choice::from(r2.is_y_odd as u8)).into(),
            ) {
                if let Some(user) = self.users.lock().await.get_mut(&request.user) {
                    log::info!("create_authentication_challenge for user {} with (r1={:?}, r2={:?})", user.name, r1, r2);
                    let auth_id = Uuid::new_v4();
                    let c = self.protocol_k256.challenge();
                    self.sessions.lock().await.insert(auth_id, Session {
                        id: None,
                        user: user.name.clone(),
                        r: Credentials::K256((r1, r2)),
                        c: BigUint::from_bytes_be(c.to_repr().as_slice()),
                    });
                    Ok(Response::new(pb2::K256AuthenticationChallengeResponse {
                        auth_id: auth_id.to_string(),
                        c: c.to_repr().to_vec(),
                    }))
                } else {
                    Err(Status::not_found("user not found"))
                }
            } else {
                Err(Status::invalid_argument("r1 point or r2 point is invalid"))
            }
        } else {
            Err(Status::invalid_argument("r1 or r2 is missing"))
        }
    }

    async fn k256_verify_authentication(&self, request: Request<pb2::K256AuthenticationAnswerRequest>) -> Result<Response<pb2::K256AuthenticationAnswerResponse>, Status> {
        let request = request.get_ref();
        if let Some(s) = Scalar::from_repr(GenericArray::clone_from_slice(request.s.as_slice())).into() {
            if let Some(session) = self.sessions.lock().await.get_mut(&Uuid::parse_str(&request.auth_id).expect("invalid auth id")) {
                let user = &self.users.lock().await[&session.user];
                let mut c: [u8; 32] = [0u8; 32];
                for (i, v) in session.c.to_bytes_be().iter().rev().enumerate() {
                    c[31 - i] = *v;
                }
                if let (
                    Credentials::K256((y1, y2)),
                    Credentials::K256((r1, r2)),
                    Some(c)
                ) = (&user.y, &session.r, Scalar::from_repr(c.into()).into()) {
                    log::info!("verify_authentication {} with (s={:?})", request.auth_id, s);
                    if self.protocol_k256.verify(y1, y2, r1, r2, &c, &s) {
                        let session_id = Uuid::new_v4();
                        session.id = Some(session_id);
                        Ok(Response::new(pb2::K256AuthenticationAnswerResponse {
                            session_id: session_id.to_string(),
                        }))
                    } else {
                        Err(Status::unauthenticated("invalid password"))
                    }
                } else {
                    Err(Status::unauthenticated("invalid protocol"))
                }
            } else {
                Err(Status::not_found("auth not found"))
            }
        } else {
            Err(Status::unauthenticated("invalid password"))
        }
    }
}


const DEFAULT_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
const DEFAULT_EXPOSED_HEADERS: [&str; 3] =
    ["grpc-status", "grpc-message", "grpc-status-details-bin"];
const DEFAULT_ALLOW_HEADERS: [&str; 4] =
    ["x-grpc-web", "content-type", "x-user-agent", "grpc-timeout"];


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    env_logger::init();

    let addr = "[::0]:50051".parse()?;
    let api = API::new();

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<pb2::auth_server::AuthServer<API>>()
        .await;

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(pb2::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    Server::builder()
        .accept_http1(true)
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::mirror_request())
                .allow_credentials(true)
                .max_age(DEFAULT_MAX_AGE)
                .expose_headers(
                    DEFAULT_EXPOSED_HEADERS
                        .iter()
                        .cloned()
                        .map(HeaderName::from_static)
                        .collect::<Vec<HeaderName>>(),
                )
                .allow_headers(
                    DEFAULT_ALLOW_HEADERS
                        .iter()
                        .cloned()
                        .map(HeaderName::from_static)
                        .collect::<Vec<HeaderName>>(),
                ),
        )
        .layer(GrpcWebLayer::new())
        .add_service(pb2::auth_server::AuthServer::new(api))
        .add_service(health_service)
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
