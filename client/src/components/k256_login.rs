use k256::{
    Scalar,
    elliptic_curve::{point::AffineCoordinates, PrimeField, generic_array::GenericArray},
};
use leptonic::prelude::*;
use leptos::*;
use protocol::ChaumPedersenK256;
use tonic::Request;
use tonic_web_wasm_client::Client;

mod pb2 {
    tonic::include_proto!("zkp_auth");
}


#[component]
pub fn K256Login(cx: Scope) -> impl IntoView {
    let client = pb2::auth_client::AuthClient::new(Client::new(format!(
        "http://{}:{}",
        std::env!("SERVICE_HOST").trim_matches('"'),
        std::env!("SERVICE_PORT").trim_matches('"'),
    )));
    let protocol = ChaumPedersenK256::new(
        std::env!("K256_H_OFFSET").parse().expect("K256_H_OFFSET is not an integer"),
    );

    let username = create_rw_signal::<String>(cx, "".into());
    let password = create_rw_signal::<String>(cx, "".into());
    let session = create_rw_signal(cx, None);
    let message = create_rw_signal::<Option<String>>(cx, None);

    let login = create_action(cx, {
        let client = client.clone();
        let protocol = protocol.clone();
        move |_| {
            let mut client = client.clone();
            let protocol = protocol.clone();
            async move {
                let username = username.get_untracked();
                let password = password.get_untracked();

                let (k, r1, r2) = protocol.commit();
                let request = Request::new(pb2::K256AuthenticationChallengeRequest {
                    user: username,
                    r1: Some(pb2::Point { x: r1.x().as_slice().into(), is_y_odd: r1.y_is_odd().into() }),
                    r2: Some(pb2::Point { x: r2.x().as_slice().into(), is_y_odd: r2.y_is_odd().into() }),
                });
                match client.k256_create_authentication_challenge(request).await {
                    Ok(response) => {
                        let response = response.into_inner();
                        log::info!("RESPONSE={:?}", response);
                        let auth_id = response.auth_id;
                        let c = Scalar::from_repr(GenericArray::clone_from_slice(response.c.as_slice())).unwrap();
                        let s = protocol.solve(&password, &k, &c);
                        let request = Request::new(pb2::K256AuthenticationAnswerRequest {
                            auth_id,
                            s: s.to_repr().to_vec(),
                        });
                        match client.k256_verify_authentication(request).await {
                            Ok(response) => {
                                let response = response.into_inner();
                                log::info!("RESPONSE={:?}", response);
                                session.set(Some(response.session_id));
                            }
                            Err(status) => {
                                message.set(Some(format!("Invalid credentials: {:?}", status.message())));
                            }
                        }
                    }
                    Err(status) => {
                        message.set(Some(format!("Invalid credentials: {:?}", status.message())));
                    }
                };
            }
        }
    });

    let logout = move || session.set(None);

    let register = create_action(cx, {
        move |_| {
            let mut client = client.clone();
            let protocol = protocol.clone();
            async move {
                let username = username.get_untracked();
                let password = password.get_untracked();
                let (y1, y2) = protocol.register(&password);
                let request = Request::new(pb2::K256RegisterRequest {
                    user: username,
                    y1: Some(pb2::Point { x: y1.x().as_slice().into(), is_y_odd: y1.y_is_odd().into() }),
                    y2: Some(pb2::Point { x: y2.x().as_slice().into(), is_y_odd: y2.y_is_odd().into() }),
                });
                match client.k256_register(request).await {
                    Ok(response) => {
                        let response = response.into_inner();
                        log::info!("RESPONSE={:?}", response);
                        message.set(Some("Registration successful".into()));
                        login.dispatch(());
                    }
                    Err(status) => {
                        message.set(Some(format!("Error: {:?}", status.message())));
                    }
                }
            }
        }
    });


    view! { cx,
        <Root default_theme=LeptonicTheme::default()>
            <Show
                when=move || session.get().is_none()
                fallback=move |cx| view! { cx,
                    <Button on_click=move |_| logout()>"Logout"</Button>
                }
            >
                <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.6)>
                    <Stack orientation=StackOrientation::Vertical spacing=Size::Em(0.6)>
                        <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.6)>
                            "Username:"
                            <TextInput get=username set=username.write_only()/>
                        </Stack>
                        <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.6)>
                            "Password:"
                            <PasswordInput get=password set=password.write_only()/>
                        </Stack>
                    </Stack>
                    <Stack orientation=StackOrientation::Vertical spacing=Size::Em(0.6)>
                        <Button on_click=move |_| register.dispatch(())>"Register"</Button>
                        <Button on_click=move |_| login.dispatch(())>"Login"</Button>
                    </Stack>
                </Stack>
            </Show>
            <Separator/>
            <Modal show_when=MaybeSignal::derive(cx, move || message.get().is_some())>
                <ModalBody>{move || message.get().unwrap_or("".into())}</ModalBody>
                <ModalFooter>
                    <ButtonWrapper>
                        <Button on_click=move |_| message.set(None) color=ButtonColor::Secondary>"Ok"</Button>
                    </ButtonWrapper>
                </ModalFooter>
            </Modal>
        </Root>
    }
}
