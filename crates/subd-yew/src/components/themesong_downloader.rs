use subd_types::ThemesongDownload;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub themesong: ThemesongDownload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {}

#[derive(Clone, PartialEq, Properties)]
pub struct ThemesongDownloader {}

impl Component for ThemesongDownloader {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let themesong = &ctx.props().themesong;

        match themesong {
            ThemesongDownload::Request { msg } => {
                html! {
                    <div class={"subd-themesong"}>
                        { format!("Requesting themesong for: {}", msg.sender.name.clone()) }
                    </div>
                }
            }
            ThemesongDownload::Start { display_name } => {
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
                    html! {
                        <div class={"subd-themesong"}>
                            { format!("Succesfully downloaded themesong: {}", display_name) }
                        </div>
                    }
                } else {
                    html! {
                        <div class={"subd-themesong"}>
                            { format!("Failed to downloaded themesong: {}", display_name) }
                        </div>
                    }
                }
            }
        }
    }
}
