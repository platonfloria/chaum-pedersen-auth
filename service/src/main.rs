use eyre::Result;
use tonic::{transport::Server, Request, Response, Status};

mod pb2 {
    tonic::include_proto!("zkp_auth");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("descriptor");
}


#[derive(Debug, Default)]
pub struct API {}

#[tonic::async_trait]
impl pb2::auth_server::Auth for API {
    async fn register(&self, request: Request<pb2::RegisterRequest>) -> Result<Response<pb2::RegisterResponse>, Status> {
        println!("register");
        Ok(Response::new(pb2::RegisterResponse {}))
    }

    async fn create_authentication_challenge(&self, request: Request<pb2::AuthenticationChallengeRequest>) -> Result<Response<pb2::AuthenticationChallengeResponse>, Status> {
        println!("create_authentication_challenge");
        Ok(Response::new(pb2::AuthenticationChallengeResponse {
            auth_id: "auth_id".into(),
            c: 1,
        }))
    }

    async fn verify_authentication(&self, request: Request<pb2::AuthenticationAnswerRequest>) -> Result<Response<pb2::AuthenticationAnswerResponse>, Status> {
        println!("verify_authentication");
        Ok(Response::new(pb2::AuthenticationAnswerResponse {
            session_id: "session_id".into(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::0]:50051".parse()?;
    let api = API::default();

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<pb2::auth_server::AuthServer<API>>()
        .await;

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(pb2::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    Server::builder()
        .add_service(pb2::auth_server::AuthServer::new(api))
        .add_service(health_service)
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
