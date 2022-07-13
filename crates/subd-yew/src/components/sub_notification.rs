#![allow(unused_variables)]

use gloo_timers::callback::Timeout;
use subd_types::TwitchSubscriptionEvent;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub subscription: TwitchSubscriptionEvent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    ShowNotification,
    HideNotification,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Show,
    Hide,
}

#[derive(Debug)]
pub struct SubNotification {
    state: State,

    #[allow(unused)]
    timeout: Timeout,
}

impl SubNotification {
    fn get_timeout(ctx: &Context<Self>) -> Timeout {
        let link = ctx.link().clone();
        Timeout::new(2000, move || link.send_message(Msg::HideNotification))
    }
}

impl Component for SubNotification {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            state: State::Show,
            timeout: Self::get_timeout(ctx),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ShowNotification => {
                self.state = State::Show;
                self.timeout = Self::get_timeout(ctx);
            }
            Msg::HideNotification => {
                self.state = State::Hide;
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let sub = ctx.props().subscription.clone();
        let name = sub.display_name();

        let mut class = "animate__zoomIn";
        if self.state != State::Show {
            class = "animate__zoomOut";
        }

        html! {
            <div class={format!("subd-notification subd-notification-big subd-notification-imp animate__animated {}", class)}>
                <svg version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"
                  viewBox="0 0 172 172">
                  <g fill="none" fill-rule="nonzero" stroke="none" stroke-width="1" stroke-linecap="butt" stroke-linejoin="miter"
                    stroke-miterlimit="10" stroke-dasharray="" stroke-dashoffset="0" font-family="none" font-weight="none"
                    font-size="none" text-anchor="none" style="mix-blend-mode: normal">
                    <path d="M0,172v-172h172v172z" fill="none"></path>
                    <g>
                      <path
                        d="M2.65391,86c0,-46.02344 37.32266,-83.34609 83.34609,-83.34609c46.02344,0 83.34609,37.32266 83.34609,83.34609c0,46.02344 -37.32266,83.34609 -83.34609,83.34609c-46.02344,0 -83.34609,-37.32266 -83.34609,-83.34609z"
                        fill="#e5aa17"></path>
                      <path
                        d="M86,32.88828l16.39375,33.19063l36.61719,5.34141l-26.50547,25.83359l6.24844,36.48281l-32.75391,-17.23359l-32.7875,17.23359l6.28203,-36.48281l-26.50547,-25.83359l36.61719,-5.34141z"
                        fill="#ffffff"></path>
                    </g>
                  </g>
                </svg>
                <b>{ name }</b>{ " has subscribed!" }
            </div>
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        log::info!("We just changed the Component");
        ctx.link().send_message(Msg::ShowNotification);
        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {}

    fn destroy(&mut self, ctx: &Context<Self>) {}
}
