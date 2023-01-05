use gloo_timers::callback::Timeout;
use subd_types::ThemesongDownload;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub themesong: ThemesongDownload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    StartTimeout {
        /// Duration in milliseconds
        duration: u32,
    },
    HideContents,
    Reset,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Show,
    Hide,
}

#[derive(Debug)]
pub struct ThemesongDownloader {
    timeout: Option<Timeout>,
    state: State,
}

impl ThemesongDownloader {
    fn get_timeout(ctx: &Context<Self>, timeout: u32) -> Timeout {
        let link = ctx.link().clone();
        Timeout::new(timeout, move || link.send_message(Msg::HideContents))
    }
}

impl Component for ThemesongDownloader {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            timeout: None,
            state: State::Show,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::info!("Calling update: {:?}", msg);
        match msg {
            Msg::StartTimeout { duration } => {
                self.timeout = Some(Self::get_timeout(ctx, duration));
                false
            }
            Msg::Reset => {
                self.timeout = None;
                self.state = State::Show;
                false
            }
            Msg::HideContents => {
                self.timeout = None;
                self.state = State::Hide;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.state == State::Hide {
            ctx.link().send_message(Msg::Reset);
            return html! {};
        }

        let themesong = &ctx.props().themesong;
        match themesong {
            ThemesongDownload::Request { msg } => {
                html! {
                    <div class={"subd-themesong"}>
                        { format!("Requesting themesong for: {}", msg.user_name.clone()) }
                    </div>
                }
            }
            ThemesongDownload::Start { display_name } => {
                ctx.link()
                    .send_message(Msg::StartTimeout { duration: 10000 });
                html! {
                    <div class={"subd-themesong"}>
                        { format!("Downloading themesong for: {}", display_name) }
                    </div>
                }
            }
            ThemesongDownload::Finish {
                display_name,
                success,
            } => {
                if *success {
                    ctx.link()
                        .send_message(Msg::StartTimeout { duration: 8000 });
                    html! {
                        <div class={"subd-themesong"}>
                            { format!("Succesfully downloaded themesong: {}", display_name) }
                        </div>
                    }
                } else {
                    ctx.link()
                        .send_message(Msg::StartTimeout { duration: 5000 });
                    html! {
                        <div class={"subd-themesong"}>
                            { format!("Failed to downloaded themesong: {}", display_name) }
                        </div>
                    }
                }
            }
            ThemesongDownload::Format { sender } => {
                ctx.link()
                    .send_message(Msg::StartTimeout { duration: 5000 });
                html! {
                    <div class={"subd-themesong"}>
                        { format!("{} says: !themesong <url> 00:00.00 00:00.00", sender) }
                    </div>
                }
            }
        }
    }
}
