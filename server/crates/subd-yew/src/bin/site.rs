use twitch_irc::message::PrivmsgMessage;
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

#[function_component(UseReducer)]
fn reducer() -> Html {
    let history = use_list(vec![
        // YewTwitchMessage {
        //     twitch_login: "NyxKrage".to_string(),
        //     color: Some("#1E90FF".to_string()),
        //     contents: "Wow, TJ is truly my favorite streamer".to_string(),
        // },
        // YewTwitchMessage {
        //     twitch_login: "KD_______T".to_string(),
        //     color: Some("Green".to_string()),
        //     contents: "Nyx, you are so right buddy".to_string(),
        // },
        // YewTwitchMessage {
        //     twitch_login: "seblj".to_string(),
        //     color: Some("LightGreen".to_string()),
        //     contents: "Awesome, its so fun to watch you work on subd".to_string(),
        // },
    ]);

    let ws = use_web_socket("ws://192.168.4.97:9001".to_string());

    {
        let history = history.clone();
        let ws = ws.clone();
        // Receive message by depending on `ws.message`.
        use_effect_with_deps(
            move |message| {
                if let Some(message) = &**message {
                    let twitch_msg: PrivmsgMessage =
                        serde_json::from_str(message).expect("got a twitch message");

                    history.push(twitch_msg);
                }
                || ()
            },
            ws.message,
        );
    }

    // TODO: Thiks is still not that good tho
    // TODO: https://www.myinstants.com/media/sounds/movie_1.mp3
    html! {
         <div class={"teej-is-bad-at-css"}>
             {
                 for history.current().iter().rev().take(20).map(|message| {
                     render_message(message)
                 })
             }
         </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<UseReducer>();
}
