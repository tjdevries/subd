use chrono::{self, Utc};
use subd_types::Event as SubdEvent;
use twitch_irc::message::{Badge, PrivmsgMessage};
// use yew::{function_component, html, use_effect_with_deps, Html};
use yew::prelude::*;
use yew_hooks::{use_list, use_web_socket};

// use_reducer or use_reducer_eq
//  Probably what we want to end up using to dispatch over Event
// Might not need to though

fn render_message(message: &PrivmsgMessage) -> Html {
    let color = message
        .name_color
        .clone()
        .unwrap_or(twitch_irc::message::RGBColor { r: 0, g: 0, b: 0 });

    log::info!("{:?}", message.badges);
    let is_moderator = message
        .badges
        .iter()
        .find(|badge| badge.name == "moderator")
        .is_some();
    let mut class_name = "subd-message".to_string();
    if is_moderator {
        class_name = format!("{} {}", class_name, "subd-message-moderator");
    }

    let color_str = format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b);
    html! {
        <div class={ class_name }>
            <p>
            <span style={ format!("color:{}", color_str) }>
                { message.sender.name.clone() }
            </span>
                { ": " }
                { message.message_text.clone() }
            </p>
        </div>
    }
}

fn default_messages() -> Vec<PrivmsgMessage> {
    vec![
        PrivmsgMessage {
            channel_login: "teej_dv".into(),
            channel_id: "_".into(),
            message_text: "Wow, Sorry for my bad behavior. I'll shape up now".into(),
            is_action: false,
            sender: twitch_irc::message::TwitchUserBasics {
                id: "TODO".into(),
                login: "nyxkrage".into(),
                name: "NyxKrage".into(),
            },
            badge_info: vec![],
            badges: vec![Badge {
                name: "moderator".into(),
                version: "1".into(),
            }],
            bits: None,
            name_color: None,
            // TODO: HANDLE EMOTES
            emotes: vec![],
            message_id: "_".into(),
            server_timestamp: Utc::now(),
            source: twitch_irc::message::IRCMessage::new_simple("this is a lie".into(), vec![]),
        },
        PrivmsgMessage {
            channel_login: "teej_dv".into(),
            channel_id: "_".into(),
            message_text: "... wish I was a mod Sadge".into(),
            is_action: false,
            sender: twitch_irc::message::TwitchUserBasics {
                id: "TODO".into(),
                login: "a_daneel".into(),
                name: "a_daneel".into(),
            },
            badge_info: vec![],
            badges: vec![],
            bits: None,
            name_color: None,
            // TODO: HANDLE EMOTES
            emotes: vec![],
            message_id: "_".into(),
            server_timestamp: Utc::now(),
            source: twitch_irc::message::IRCMessage::new_simple("this is a lie".into(), vec![]),
        },
        PrivmsgMessage {
            channel_login: "teej_dv".into(),
            channel_id: "_".into(),
            message_text: "Oh hey I'm Oetzi and I'm super nice!".into(),
            is_action: false,
            sender: twitch_irc::message::TwitchUserBasics {
                id: "TODO".into(),
                login: "oetziofficial".into(),
                name: "OetziOfficial".into(),
            },
            badge_info: vec![],
            badges: vec![],
            bits: None,
            name_color: None,
            // TODO: HANDLE EMOTES
            emotes: vec![],
            message_id: "_".into(),
            server_timestamp: Utc::now(),
            source: twitch_irc::message::IRCMessage::new_simple("this is a lie".into(), vec![]),
        },
    ]
}

#[function_component(UseReducer)]
fn reducer() -> Html {
    let history = use_list(default_messages());
    let subcount = use_state(|| 0);

    let ws = use_web_socket("ws://192.168.4.97:9001".to_string());

    {
        let history = history.clone();
        let ws = ws.clone();
        let subcount = subcount.clone();

        // Receive message by depending on `ws.message`.
        use_effect_with_deps(
            move |message| {
                if let Some(message) = &**message {
                    let event: SubdEvent =
                        serde_json::from_str(message).expect("got a twitch message");

                    match event {
                        SubdEvent::TwitchChatMessage(twitch_msg) => history.push(twitch_msg),
                        SubdEvent::TwitchSubscriptionCount(count) => subcount.set(count),
                        _ => {}
                    }
                }
                || ()
            },
            ws.message,
        );
    }

    // TODO: Thiks is still not that good tho
    // TODO: Theme songs
    //          https://www.myinstants.com/media/sounds/movie_1.mp3

    html! {
        <div class={ "subd" }>
            <div class={"subd-goal"}>
                <p>{ format!("{} / 420", *subcount) }</p>
            </div>
            <div class={"subd-chat"}>
            {
                {
                    let x = 10;

                    history.current().iter().rev().take(x).map(|message| {
                        render_message(message)
                    }).collect::<Html>()
                }
            }
            </div>
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<UseReducer>();
}
