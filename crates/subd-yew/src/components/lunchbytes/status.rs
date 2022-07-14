use yew::prelude::*;

#[derive(Clone, Properties, PartialEq)]
pub struct TopicProps {
    pub id: u32,
    pub text: String,
    pub percentage: f32,
}

#[function_component(Topic)]
fn topic(props: &TopicProps) -> Html {
    html! {
        <>
            <span>{props.id}</span>
            <span>{props.text.clone()}</span>
            <div class="subd-bar" style={format!("--percentage: {}%", props.percentage * 100.)}>
                <div></div>
                <span>{format!("{}%", props.percentage * 100.)}</span>
            </div>
        </>
    }
}

pub enum Msg {}

#[derive(Clone, PartialEq, Properties)]
pub struct StatusProps {
    pub enabled: bool,
    pub topics: Vec<TopicProps>,
}

pub struct Status {}

impl Component for Status {
    type Message = Msg;
    type Properties = StatusProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if !ctx.props().enabled {
            return html! {};
        }

        let topics = ctx
            .props()
            .topics
            .clone()
            .into_iter()
            .map(|t| html! { <li> <Topic ..t /> </li> })
            .collect::<Html>();

        html! {
            <div class="subd-topics">
                <h1>{ "Topics List" }</h1>
                <ul>
                    <li class="subd-topics-header">
                        <span class="subd-topics-header-id">ID</span>
                        <span class="subd-topics-header-topic">Topic Name</span>
                        <span class="subd-topics-header-votes">Votes</span>
                    </li>
                    { topics }
                </ul>
            </div>
        }
    }
}
