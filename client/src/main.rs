use num_bigint::BigUint;
use eyre::Result;
use tonic::{transport::{Channel, Uri}, Request};

mod pb2 {
    tonic::include_proto!("zkp_auth");
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    env_logger::init();

    let p: BigUint = std::env::var("P").expect("P env var must be set.").parse().expect("P is not an integer");
    let q: BigUint = std::env::var("Q").expect("Q env var must be set.").parse().expect("Q is not an integer");
    let g: BigUint = std::env::var("G").expect("G env var must be set.").parse().expect("G is not an integer");
    let h: BigUint = std::env::var("H").expect("H env var must be set.").parse().expect("H is not an integer");

    let uri = format!(
        "http://{}:{}",
        std::env::var("SERVICE_HOST").expect("SERVICE_HOST env var must be set."),
        std::env::var("SERVICE_PORT").expect("SERVICE_PORT env var must be set.")
    ).parse::<Uri>().unwrap();
    let channel = Channel::builder(uri).connect().await?;
    let mut client = pb2::auth_client::AuthClient::new(channel);

    let username = "username".to_string();
    let x = rand::random::<u64>().into();
    let y1 = g.modpow(&x, &p).to_u64_digits()[0];
    let y2 = h.modpow(&x, &p).to_u64_digits()[0];
    let request = Request::new(pb2::RegisterRequest {
            user: username.clone(),
            y1,
            y2,
        },
    );
    let response = client.register(request).await?.into_inner();
    log::info!("RESPONSE={:?}", response);

    let k = rand::random::<u64>().into();
    let r1 = g.modpow(&k, &p).to_u64_digits()[0];
    let r2 = h.modpow(&k, &p).to_u64_digits()[0];
    let request = Request::new(pb2::AuthenticationChallengeRequest {
            user: username,
            r1,
            r2,
        },
    );
    let response = client.create_authentication_challenge(request).await?.into_inner();
    log::info!("RESPONSE={:?}", response);
    let auth_id = response.auth_id;
    let c = response.c;

    let s = if k >= q {
        k - (c * x) % &q
    } else {
        &q + k - (c * x) % q
    }.to_u64_digits()[0];
    let request = Request::new(pb2::AuthenticationAnswerRequest {
            auth_id,
            s,
        },
    );
    let response = client.verify_authentication(request).await?.into_inner();
    log::info!("RESPONSE={:?}", response);

    Ok(())
}
