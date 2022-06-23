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
        log::info!("Calling update: {:?}", msg);
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

        if self.state == State::Show {
            html! {
                <div class={"subd-notification animate__animated animate__bounceInDown"}>
                    { name } { " has subscribed!" }
                </div>
            }
        } else {
            html! {
                <div class={"subd-notification animate__animated animate__bounceOutLeft"}>
                    { name } { " has subscribed!" }
                </div>
            }
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
