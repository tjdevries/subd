use std::collections::HashSet;

use gloo_timers::callback::Timeout;
use subd_types::RaffleStatus;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub raffle_status: RaffleStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    SetActive,
    StartHide,
    SetHide,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Active,
    Show,
    Hide,
}

pub struct RaffleComponent {
    state: State,
    timeout: Option<Timeout>,
    existing_names: HashSet<String>,
}

impl RaffleComponent {
    fn get_timeout(ctx: &Context<Self>, msg: Msg) -> Timeout {
        let link = ctx.link().clone();
        Timeout::new(2000, move || link.send_message(msg))
    }
}

impl Component for RaffleComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        log::info!("Calling Create");

        Self {
            state: match ctx.props().raffle_status {
                RaffleStatus::Disabled => State::Hide,
                _ => State::Show,
            },
            timeout: None,
            existing_names: HashSet::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::info!("Calling update: {:?}", msg);

        match msg {
            Msg::StartHide => {
                self.state = State::Show;
                self.timeout = Some(Self::get_timeout(ctx, Msg::SetHide));
                return false;
            }
            Msg::SetHide => {
                self.state = State::Hide;
                self.timeout = None;
                return true;
            }
            Msg::SetActive => {
                self.state = State::Active;
                self.timeout = None;
                return false;
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        log::info!("Current State: {:?}", self.state);

        // TODO: Twitch chat colors for entry would be sweet.
        match &ctx.props().raffle_status {
            subd_types::RaffleStatus::Disabled => {
                if self.state != State::Hide {
                    ctx.link().send_message(Msg::StartHide);

                    html! {
                        <div class={"subd-notification animate__animated animate__bounceInDown"}>
                            { "End of Raffle. Better luck next time." }
                        </div>
                    }
                } else {
                    html! {
                        <div class={"subd-notification animate__animated animate__bounceOutLeft"}>
                            { "End of Raffle. Better luck next time." }
                        </div>
                    }
                }
            }
            subd_types::RaffleStatus::Ongoing { title, entries } => {
                ctx.link().send_message(Msg::SetActive);

                let list = entries
                    .iter()
                    .map(|(user, _)| html! { <li> { user } </li> })
                    .collect::<Html>();

                html! {
                    <div class="raffle">
                        <h1> { title } </h1>
                        <ul>
                            { list }
                        </ul>
                    </div>
                }
            }
            subd_types::RaffleStatus::Winner { users } => {
                ctx.link().send_message(Msg::SetActive);

                html! {
                    <div class="raffle-winners"> { format!("Winners! {:?}", users) } </div>
                }
            }
        }
    }
}
