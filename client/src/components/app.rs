use leptonic::prelude::*;
use leptos::*;
use num_bigint::BigUint;
use tonic::Request;
use tonic_web_wasm_client::Client;

mod pb2 {
    tonic::include_proto!("zkp_auth");
}

use crate::protocol::ChaumPedersen;


#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let client = pb2::auth_client::AuthClient::new(Client::new(format!(
        "http://{}:{}",
        std::env!("SERVICE_HOST").trim_matches('"'),
        std::env!("SERVICE_PORT").trim_matches('"'),
    )));
    let protocol = ChaumPedersen::new(
        std::env!("P").parse().expect("P is not an integer"),
        std::env!("Q").parse().expect("Q is not an integer"),
        std::env!("G").parse().expect("G is not an integer"),
        std::env!("H").parse().expect("H is not an integer"),
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
                let request = Request::new(pb2::AuthenticationChallengeRequest {
                    user: username,
                    r1: r1.to_bytes_be(),
                    r2: r2.to_bytes_be(),
                });
                match client.create_authentication_challenge(request).await {
                    Ok(response) => {
                        let response = response.into_inner();
                        log::info!("RESPONSE={:?}", response);
                        let auth_id = response.auth_id;
                        let c = BigUint::from_bytes_be(&response.c);

                        let s = protocol.solve(password, &k, &c);
                        let request = Request::new(pb2::AuthenticationAnswerRequest {
                            auth_id,
                            s: s.to_bytes_be(),
                        });
                        match client.verify_authentication(request).await {
                            Ok(response) => {
                                let response = response.into_inner();
                                log::info!("RESPONSE={:?}", response);
                                session.set(Some(response.session_id));
                            }
                            Err(_) => {
                                message.set(Some("Invalid credentials".into()));
                            }
                        }
                    }
                    Err(_) => {
                        message.set(Some("Invalid credentials".into()));
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
                let (y1, y2) = protocol.register(password);
                let request = Request::new(pb2::RegisterRequest {
                    user: username,
                    y1: y1.to_bytes_be(),
                    y2: y2.to_bytes_be(),
                });
                if let Ok(response) = client.register(request).await {
                    let response = response.into_inner();
                    log::info!("RESPONSE={:?}", response);
                    message.set(Some("Registration successful".into()));
                    login.dispatch(());
                } else {
                    message.set(Some("Error".into()));
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
