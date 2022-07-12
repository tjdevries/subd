use subd_types::ThemesongPlay;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub play: ThemesongPlay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {}

#[derive(Clone, PartialEq, Properties)]
pub struct ThemesongPlayer {}

impl Component for ThemesongPlayer {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let themesong = &ctx.props().play;
        match themesong {
            ThemesongPlay::Start { display_name, .. } => {
                html! {
                    <div class={"subd-themesong-play"}>
                        { format!("playing {} themesong", display_name) }
                    </div>
                }
            }

            ThemesongPlay::Finish { .. } => {
                html! {}
            }
        }
    }
}
