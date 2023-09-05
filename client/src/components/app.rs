use leptonic::prelude::*;
use leptos::*;

use crate::protocol::ChaumPedersen;


#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let protocol = ChaumPedersen::new();

    let username = create_rw_signal(cx, "".into());
    let password = create_rw_signal::<String>(cx, "".into());
    let session = create_rw_signal(cx, None);
    let message = create_rw_signal::<Option<String>>(cx, None);

    let login = create_action(cx, {
        let protocol = protocol.clone();
        move |_| {
            let mut protocol = protocol.clone();
            async move {
                let username = username.get_untracked();
                let password = password.get_untracked();
                match protocol.commit(username).await {
                    Ok((k, auth_id, c)) => {
                        match protocol.verify(password, k, auth_id, c).await {
                            Ok(session_id) => {
                                session.set(Some(session_id));
                            }
                            Err(_) => {
                                message.set(Some("Invalid credentials".into()));
                            }
                        }
                    }
                    Err(_) => {
                        message.set(Some("Invalid credentials".into()));
                    },
                }
            }
        }
    });

    let logout = move || session.set(None);

    let register = create_action(cx, {
        move |_| {
            let mut protocol = protocol.clone();
            async move {
                let username = username.get_untracked();
                let password = password.get_untracked();
                if let Err(_) = protocol.register(username, password).await {
                    message.set(Some("Error".into()));
                } else {
                    message.set(Some("Registration successful".into()));
                    login.dispatch(());
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
