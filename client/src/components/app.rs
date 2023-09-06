use leptonic::prelude::*;
use leptos::*;

use crate::components::{
    exp_login::ExpLogin,
    k256_login::K256Login,
};


#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let message = create_rw_signal::<Option<String>>(cx, None);
    provide_context(cx, message);

    view! { cx,
        <Root default_theme=LeptonicTheme::default()>
            <Stack orientation=StackOrientation::Vertical spacing=Size::Em(0.6)>
                "Login using exponential protocol version"
                <ExpLogin/>
                <Separator/>
                "Login using elliptic curve protocol version"
                <K256Login/>
            </Stack>
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
