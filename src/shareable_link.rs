use gloo::timers::callback::Timeout;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use yew::web_sys::HtmlInputElement;
use yew::{prelude::*, utils::window};

enum UserMessage {
    CopySuccess,
    CopyFailure,
}

pub struct ShareableLink {
    props: ShareableLinkProps,
    link: ComponentLink<Self>,
    user_message_shown: Option<Timeout>,
    user_message: Option<UserMessage>,
    on_copy_success: Closure<dyn FnMut(JsValue)>,
    on_copy_failure: Closure<dyn FnMut(JsValue)>,
}

pub enum Msg {
    CopyLink,
    ShowSuccess,
    ShowFailure,
    HideMessage,
}

#[derive(Properties, Clone, PartialEq)]
pub struct ShareableLinkProps {
    pub link_to_share: String,
}

impl ShareableLink {
    fn create_hide_message_timeout(&self) -> Timeout {
        let link = self.link.clone();
        Timeout::new(1000, move || {
            link.send_message(Msg::HideMessage);
        })
    }
}

impl Component for ShareableLink {
    type Message = Msg;
    type Properties = ShareableLinkProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let link_clone_for_copy_success = link.clone();
        let link_clone_for_copy_failure = link.clone();
        Self {
            props,
            link,
            user_message: None,
            user_message_shown: None,
            on_copy_success: Closure::wrap(Box::new(move |_| {
                link_clone_for_copy_success.send_message(Msg::ShowSuccess);
            })),
            on_copy_failure: Closure::wrap(Box::new(move |_| {
                link_clone_for_copy_failure.send_message(Msg::ShowFailure);
            })),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::CopyLink => {
                if let Some(clipboard) = window().navigator().clipboard() {
                    let _ = clipboard
                        .write_text(&self.props.link_to_share)
                        .then(&self.on_copy_success)
                        .catch(&self.on_copy_failure);
                }
            }
            Msg::ShowSuccess => {
                if let Some(existing_timeout) = self.user_message_shown.take() {
                    existing_timeout.cancel();
                }
                self.user_message = Some(UserMessage::CopySuccess);
                self.user_message_shown = Some(self.create_hide_message_timeout());
            }
            Msg::ShowFailure => {
                if let Some(existing_timeout) = self.user_message_shown.take() {
                    existing_timeout.cancel();
                }
                self.user_message = Some(UserMessage::CopyFailure);
                self.user_message_shown = Some(self.create_hide_message_timeout());
            }
            Msg::HideMessage => {
                self.user_message_shown = None;
            }
        }

        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props != props {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let (user_message_classes, user_message_text) = if let Some(message) = &self.user_message {
            let hidden_class = if self.user_message_shown.is_some() {
                classes!()
            } else {
                classes!("hidden")
            };
            match message {
                UserMessage::CopySuccess => (classes!(hidden_class, "success"), "Link copied!"),
                UserMessage::CopyFailure => (
                    classes!(hidden_class, "failure"),
                    "Sorry, link wasn’t copied",
                ),
            }
        } else {
            (classes!("hidden"), "")
        };
        html! {
            <div class="share">
                <label for="shareable-link">
                    {"Shareable link"}
                    {
                        html!{
                            <span class=user_message_classes>{user_message_text}</span>
                        }
                    }
                </label>
                <div>
                    <input
                        id="shareable-link"
                        type="text"
                        readonly=true
                        onclick=Callback::from(|event: MouseEvent| {
                            event.target()
                                .and_then(|target| target.dyn_into::<HtmlInputElement>().ok())
                                .map(|input| {
                                    let _ = input.set_selection_range(0, input.value().len() as u32);
                                });
                        })
                        value=self.props.link_to_share.clone()
                    />
                    <button onclick=self.link.callback(|_| Msg::CopyLink)>
                        {"Copy"}
                    </button>
                </div>
            </div>
        }
    }
}
