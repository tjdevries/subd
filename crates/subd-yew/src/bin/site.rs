use std::cmp::max;
use std::collections::VecDeque;

use chrono::{self, Utc};
use subd_types::{Event as SubdEvent, LunchBytesStatus};
use subd_yew::components::lunchbytes::{self, status};
use subd_yew::components::raffle::RaffleComponent;
use subd_yew::components::sub_notification::SubNotification;
use subd_yew::components::themesong_downloader::ThemesongDownloader;
use subd_yew::components::themesong_player::ThemesongPlayer;
use twitch_irc::message::{Badge, Emote, PrivmsgMessage};
use yew::prelude::*;
use yew_hooks::{use_list, use_web_socket};

const SHOULD_DEFAULT_MESSAGES: bool = false;

// use_reducer or use_reducer_eq
//  Probably what we want to end up using to dispatch over Event
// Might not need to though

// https://static-cdn.jtvnw.net/emoticons/v2/<id>/<format>/<theme_mode>/<scale>
fn make_emote_url(emote: &Emote) -> String {
    format!(
        "https://static-cdn.jtvnw.net/emoticons/v2/{}/default/dark/2.0",
        emote.id
    )
}

fn render_message(message: &PrivmsgMessage) -> Html {
    let color = message
        .name_color
        .clone()
        .unwrap_or(twitch_irc::message::RGBColor { r: 0, g: 0, b: 0 });

    let is_moderator = message
        .badges
        .iter()
        .find(|badge| badge.name == "moderator")
        .is_some();
    let mut class_name = "subd-message".to_string();
    if is_moderator {
        class_name = format!("{} {}", class_name, "subd-message-moderator");
    }

    let contents = message.message_text.clone();
    let mut pieces: Vec<Html> = vec![];

    if message.emotes.is_empty() {
        pieces.push(html! {
            <p>
             { contents }
            </p>
        });
    } else {
        let mut contents = contents.chars();
        let mut last_emote_finish = 0;

        let mut emotes = VecDeque::from(message.emotes.clone());
        while let Some(emote) = emotes.pop_front() {
            // Get the missing text contents
            let segment = contents
                .by_ref()
                .take(emote.char_range.start - last_emote_finish)
                .collect::<String>();
            if !segment.is_empty() {
                pieces.push(html! { <span> { segment } </span> });
            }

            pieces.push(
                html! { <img src={make_emote_url(&emote)} alt={"emote"} /> },
            );

            last_emote_finish = emote.char_range.end;

            // This is just stupid skip cause i'm stupid and tired today
            _ = contents
                .by_ref()
                .take(emote.char_range.len())
                .collect::<String>();
        }

        let remaining = contents.collect::<String>();
        if !remaining.is_empty() {
            pieces.push(html! { <span> { remaining } </span> });
        }
        // pieces.push(html! { <p> { "You think i'll just show your emotes??" } </p> });
    }

    let color_str = format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b);
    html! {
        <div class={ class_name }>
            <span style={ format!("color:{}", color_str) }>
                { message.sender.name.clone() }
            </span>
            { ": " }
            { pieces }
        </div>
    }
}

fn default_messages() -> Vec<PrivmsgMessage> {
    // if !SHOULD_DEFAULT_MESSAGES {
    //     return vec![];
    // }

    // let channel_username = subd_types::consts::get_twitch_broadcaster_username();
    let channel_username = "teej_dv".to_string();
    vec![
        PrivmsgMessage {
            channel_login: channel_username.clone().into(),
            channel_id: "_".into(),
            message_text: "Wow, Sorry for my bad behavior. I'll shape up now"
                .into(),
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
            source: twitch_irc::message::IRCMessage::new_simple(
                "this is a lie".into(),
                vec![],
            ),
        },
        PrivmsgMessage {
            channel_login: channel_username.clone().into(),
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
            source: twitch_irc::message::IRCMessage::new_simple(
                "this is a lie".into(),
                vec![],
            ),
        },
        PrivmsgMessage {
            channel_login: channel_username.clone().into(),
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
            source: twitch_irc::message::IRCMessage::new_simple(
                "this is a lie".into(),
                vec![],
            ),
        },
    ]
}

// fn print_type_of<T>(_: &T) -> String {
//     format!("{}", std::any::type_name::<T>())
// }

#[derive(Properties, PartialEq)]
pub struct Props {
    pub ws_address: String,
}

#[function_component(UseReducer)]
fn reducer(props: &Props) -> Html {
    log::info!("ws_address: {}", props.ws_address);

    // This needs to be generic
    let ws = use_web_socket(props.ws_address.clone());

    let history = use_list(default_messages());
    let subcount = use_state(|| 0);

    let new_sub = use_state(|| None);
    let themesong = use_state(|| None);
    let player = use_state(|| None);
    let lb_status = use_state(|| LunchBytesStatus {
        enabled: false,
        topics: vec![],
    });
    let raffle_status = use_state(|| subd_types::RaffleStatus::Disabled);

    {
        let history = history.clone();
        let ws = ws.clone();
        let subcount = subcount.clone();
        let new_sub = new_sub.clone();
        let themesong = themesong.clone();
        let player = player.clone();
        let lb_status = lb_status.clone();
        let raffle_status = raffle_status.clone();

        // Receive message by depending on `ws.message`.
        use_effect_with_deps(
            move |message| {
                if let Some(message) = &**message {
                    let event: SubdEvent = serde_json::from_str(message)
                        .expect("got a twitch message");

                    match event {
                        SubdEvent::TwitchChatMessage(twitch_msg) => {
                            history.push(twitch_msg)
                        }
                        SubdEvent::TwitchSubscriptionCount(count) => {
                            subcount.set(count)
                        }
                        SubdEvent::TwitchSubscription(subscription) => {
                            log::info!(
                                "Got a new subscription: {:?}",
                                subscription
                            );
                            // handle_twitch_sub(subscription)
                            new_sub.set(Some(subscription))
                        }
                        SubdEvent::ThemesongDownload(download) => {
                            let download_type = match download {
                                subd_types::ThemesongDownload::Request {
                                    ..
                                } => "Request",
                                subd_types::ThemesongDownload::Start {
                                    ..
                                } => "Start",
                                subd_types::ThemesongDownload::Finish {
                                    ..
                                } => "Finish",
                                subd_types::ThemesongDownload::Format {
                                    ..
                                } => "Format",
                            };
                            log::info!(
                                "New download request: {:?}",
                                download_type
                            );
                            themesong.set(Some(download))
                        }
                        SubdEvent::ThemesongPlay(play) => {
                            player.set(Some(play))
                        }
                        SubdEvent::LunchBytesStatus(mut lunchbytes_status) => {
                            // Sort by votes
                            lunchbytes_status
                                .topics
                                .sort_by(|a, b| b.votes.cmp(&a.votes));

                            lb_status.set(lunchbytes_status)
                        }
                        SubdEvent::RaffleStatus(raffle_msg) => {
                            raffle_status.set(raffle_msg);
                        }

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

    let notification = match &(*new_sub) {
        Some(sub) => {
            let sub = sub.clone();
            html! { <SubNotification subscription={sub} /> }
        }
        None => html! {},
    };

    let themesong = match &(*themesong) {
        Some(themesong) => {
            let themesong = themesong.clone();
            html! { <ThemesongDownloader themesong={themesong} /> }
        }
        None => html! {},
    };

    let player = match &(*player) {
        Some(play) => {
            let play = play.clone();
            html! { <ThemesongPlayer play={play} />}
        }
        None => html! {},
    };

    let raffle_html =
        html! { <RaffleComponent raffle_status={(*raffle_status).clone()} /> };

    // TODO: Consider using max instead
    // let total_votes = lb_status.topics.iter().map(|t| t.votes).max().unwrap_or(1);
    let total_votes =
        max(lb_status.topics.iter().map(|t| t.votes).sum::<i32>(), 1);
    let status_props = status::StatusProps {
        enabled: lb_status.enabled,
        topics: lb_status
            .topics
            .iter()
            .map(|t| status::TopicProps {
                id: t.id,
                text: t.text.clone(),
                percentage: t.votes as f32 / total_votes as f32,
            })
            .collect(),
    };

    // <div class={"subd-goal"}>
    //     <p>{ format!("{} / 420", *subcount) }</p>
    // </div>

    html! {
        <div class={ "subd" }>
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
            <> { notification } </>
            <> { themesong } </>
            <> { player } </>
            <> { raffle_html } </>
            <> <lunchbytes::status::Status ..status_props/> </>
        </div>
    }
}

fn main() {
    // TODO: This is a bad hack... but for now it works OK.
    //  This is to make it work for me & begin during dev.
    //  We'll figure out a better way for this later.
    let env_file = include_str!("../../../../.env");

    // Get the address!
    let ws_address = env_file
        .lines()
        .find(|l| l.starts_with("SUBD_YEW_WS_ADDRESS"))
        .expect("ws_address")
        .split_once("=")
        .expect("ws_addres =")
        .1
        .trim_matches('"');

    let ws_port = env_file
        .lines()
        .find(|l| l.starts_with("SUBD_YEW_WS_PORT"))
        .unwrap()
        .split_once("=")
        .unwrap()
        .1
        .trim_matches('"');

    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app_with_props::<UseReducer>(Props {
        ws_address: format!("ws://{}:{}", ws_address, ws_port),
    });
}
