use std::{
    hash::{Hash, Hasher},
    collections::hash_map::DefaultHasher
};

use eyre::Result;
use num_bigint::BigUint;
use tonic::Request;
use tonic_web_wasm_client::Client;

mod pb2 {
    tonic::include_proto!("zkp_auth");
}


#[derive(Clone)]
pub struct ChaumPedersen {
    p: BigUint,
    q: BigUint,
    g: BigUint,
    h: BigUint,
    client: pb2::auth_client::AuthClient<Client>,
}

impl ChaumPedersen {
    pub fn new() -> Self {
        Self {
            p: std::env!("P").parse().expect("P is not an integer"),
            q: std::env!("Q").parse().expect("Q is not an integer"),
            g: std::env!("G").parse().expect("G is not an integer"),
            h: std::env!("H").parse().expect("H is not an integer"),
            client: pb2::auth_client::AuthClient::new(Client::new(format!(
                "http://{}:{}",
                std::env!("SERVICE_HOST").trim_matches('"'),
                std::env!("SERVICE_PORT").trim_matches('"'),
            ))),
        }
    }

    pub async fn register(&mut self, username: String, password: String) -> Result<()>  {
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        let x = hasher.finish().into();

        let y1 = self.g.modpow(&x, &self.p).to_u64_digits()[0];
        let y2 = self.h.modpow(&x, &self.p).to_u64_digits()[0];
        let request = Request::new(pb2::RegisterRequest {
                user: username,
                y1,
                y2,
            },
        );
        let response = self.client.register(request).await?.into_inner();
        log::info!("RESPONSE={:?}", response);
        Ok(())
    }

    pub async fn commit(&mut self, username: String) -> Result<(BigUint, String, BigUint)> {
        let k = rand::random::<u64>().into();
        let r1 = self.g.modpow(&k, &self.p).to_u64_digits()[0];
        let r2 = self.h.modpow(&k, &self.p).to_u64_digits()[0];
        let request = Request::new(pb2::AuthenticationChallengeRequest {
                user: username,
                r1,
                r2,
            },
        );
        let response = self.client.create_authentication_challenge(request).await?.into_inner();
        log::info!("RESPONSE={:?}", response);
        Ok((k, response.auth_id, response.c.into()))
    }

    pub async fn verify(&mut self, password: String, k: BigUint, auth_id: String, c: BigUint) -> Result<String> {
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        let x: BigUint = hasher.finish().into();

        let s = if &k >= &self.q {
            k - (c * x) % &self.q
        } else {
            &self.q + k - (c * x) % &self.q
        }.to_u64_digits()[0];
        let request = Request::new(pb2::AuthenticationAnswerRequest {
                auth_id,
                s,
            },
        );
        let response = self.client.verify_authentication(request).await?.into_inner();
        log::info!("RESPONSE={:?}", response);
        Ok(response.session_id)
    }
}
