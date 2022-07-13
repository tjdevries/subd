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
                        {format!("Playing {} themesong!", display_name)}
                    </div>
                }
            }

            ThemesongPlay::Finish { .. } => {
                html! {}
            }
        }
    }
}
