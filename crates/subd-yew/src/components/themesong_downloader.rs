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

        let mut msg = "".to_string();

        let themesong = &ctx.props().themesong;
        match themesong {
            ThemesongDownload::Request { msg } => {
                msg = format!("Requesting themesong for: {}", msg.sender.name.clone());
            }
            ThemesongDownload::Start { display_name } => {
                ctx.link()
                    .send_message(Msg::StartTimeout { duration: 10000 });
                msg = format!("Downloading themesong for: {}", display_name);
            }
            ThemesongDownload::Finish {
                display_name,
                success,
            } => {
                if *success {
                    ctx.link()
                        .send_message(Msg::StartTimeout { duration: 8000 });
                    msg = format!("Succesfully downloaded themesong: {}", display_name);
                } else {
                    ctx.link()
                        .send_message(Msg::StartTimeout { duration: 5000 });
                    msg = format!("Failed to downloaded themesong: {}", display_name);
                }
            }
            ThemesongDownload::Format { sender } => {
                ctx.link()
                    .send_message(Msg::StartTimeout { duration: 5000 });
                html! {
                        msg =  format!("{} says: !themesong <url> 00:00.00 00:00.00", sender) ;
                }
            }
        }

        html! {
            <div class={"subd-notification animate__animated animate__zoomIn"}>
                <svg version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"
                  viewBox="0 0 172 172">
                    <g fill="none" fill-rule="nonzero" stroke="none" stroke-width="1" stroke-linecap="butt" stroke-linejoin="miter"
                      stroke-miterlimit="10" stroke-dasharray="" stroke-dashoffset="0" font-family="none" font-weight="none"
                      font-size="none" text-anchor="none" style="mix-blend-mode: normal">
                        <path d="M0,172v-172h172v172z" fill="none"></path>
                        <g>
                            <path
                              d="M157.66667,86c0,39.58508 -32.08158,71.66667 -71.66667,71.66667c-39.58508,0 -71.66667,-32.08158 -71.66667,-71.66667c0,-39.58508 32.08158,-71.66667 71.66667,-71.66667c39.58508,0 71.66667,32.08158 71.66667,71.66667z"
                              fill="#f9c74f"></path>
                            <path
                              d="M100.33333,107.5c0,11.87158 -9.632,21.5 -21.5,21.5c-11.87158,0 -21.5,-9.632 -21.5,-21.5c0,-11.868 9.62842,-21.5 21.5,-21.5c11.868,0 21.5,9.632 21.5,21.5z"
                              fill="#f94144"></path>
                            <path d="M89.58333,43v64.5h10.75v-40.84283l17.91667,9.3095v-17.91667z" fill="#f94144"></path>
                        </g>
                    </g>
                </svg>
                {msg}
            </div>
        }
    }
}
