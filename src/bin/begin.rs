use anyhow::Result;
use clap::Parser;
use elevenlabs_api::{Auth, Elevenlabs};
use serde::{Deserialize, Serialize};
use server::audio;
use server::handlers;
use server::obs::obs::create_obs_client;
use std::collections::HashMap;
use subd_db::get_db_pool;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::ClientConfig;
use twitch_irc::SecureTCPTransport;
use twitch_irc::TwitchIRCClient;

fn get_chat_config() -> ClientConfig<StaticLoginCredentials> {
    let twitch_username = subd_types::consts::get_twitch_bot_username();
    ClientConfig::new_simple(StaticLoginCredentials::new(
        twitch_username,
        Some(subd_types::consts::get_twitch_bot_oauth()),
    ))
}

#[derive(Serialize, Deserialize, Debug)]
struct EventSubRoot {
    subscription: Subscription,
    event: Option<EventSub>,
    challenge: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Subscription {
    id: String,
    status: String,
    #[serde(rename = "type")]
    type_field: String,
    version: String,
    condition: HashMap<String, String>,
    transport: Transport,
    created_at: String,
    cost: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Transport {
    method: String,
    callback: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct EventSub {
    user_id: String,
    user_login: String,
    user_name: String,
    broadcaster_user_id: String,
    broadcaster_user_login: String,
    broadcaster_user_name: String,
    tier: Option<String>,
    is_gift: Option<bool>,
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Whether to Enable All Features
    #[arg(long)]
    enable_all: bool,

    /// Whether to Enable the AI Scenes
    #[arg(long, value_delimiter = ' ', num_args = 1..)]
    enable: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    {
        use rustrict::{add_word, Type};

        // You must take care not to call these when the crate is being
        // used in any other way (to avoid concurrent mutation).
        unsafe {
            add_word(format!("vs{}", "code").as_str(), Type::PROFANE);
            add_word("vsc*de", Type::SAFE);
        }
    }

    // Create 1 Event Loop
    // Push handles onto the loop
    // those handlers are things like twitch-chat, twitch-sub, github-sponsor etc.
    let mut event_loop = events::EventLoop::new();

    let args = Args::parse();

    let features = if args.enable_all {
        vec![
            "implict_soundeffects".to_string(),
            "explicit_soundeffects".to_string(),
            // "tts".to_string(),
            "ai_screenshots".to_string(),
            // "ai_screenshots_timer".to_string(),
            "ai_telephone".to_string(),
            "ai_scenes".to_string(),
            "skybox".to_string(),
            "obs".to_string(),
            "twitch_chat_saving".to_string(),
            "stream_character".to_string(),
            "chat_gpt_response".to_string(),
            "twitch_eventsub".to_string(),
            "dynamic_stream_background".to_string(),
            "channel_rewards".to_string(),
            "ai_songs".to_string(),
            "turbo_bg".to_string(),
        ]
    } else {
        args.enable
    };

    let pool = get_db_pool().await;
    let (_stream, stream_handle) =
        audio::get_output_stream("pulse").expect("stream handle");

    for feature in features {
        match feature.as_ref() {
            "twitch_chat_saving" => {
                println!("Enabling Twitch Chat Saving");
                event_loop.push(twitch_chat::client::TwitchChat::new(
                    pool.clone(),
                    "beginbot".to_string(),
                )?);

                // TODO: Update this description to be more exact
                // Saves the message and extracts out some information
                // for easier routing
                event_loop.push(
                    twitch_chat::handlers::TwitchMessageHandler::new(
                        pool.clone(),
                        twitch_service::Service::new(
                            pool.clone(),
                            user_service::Service::new(pool.clone()).await,
                        )
                        .await,
                    ),
                );

                // TODO: We want to reconsider our feature boundaries
                let obs_client = create_obs_client().await?;
                event_loop.push(handlers::voices_handler::VoicesHandler {
                    pool: pool.clone(),
                    obs_client,
                });
                continue;
            }
            // This might be named Wrong
            "implict_soundeffects" => {
                // TODO: This should be abstracted, Works for Arch Linux
                // Works for Mac
                // let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
                let sink = rodio::Sink::try_new(&stream_handle).unwrap();
                event_loop.push(handlers::sound_handler::SoundHandler {
                    sink,
                    pool: pool.clone(),
                });
            }

            "explicit_soundeffects" => {
                let sink = rodio::Sink::try_new(&stream_handle).unwrap();
                event_loop.push(
                    handlers::sound_handler::ExplicitSoundHandler {
                        sink,
                        pool: pool.clone(),
                    },
                );
            }

            "tts" => {
                println!("Enabling TTS");
                // Elevenlabs/elevenlabs handles voice messages
                let elevenlabs_auth = Auth::from_env().unwrap();
                let elevenlabs = Elevenlabs::new(
                    elevenlabs_auth,
                    "https://api.elevenlabs.io/v1/",
                );
                let sink = rodio::Sink::try_new(&stream_handle).unwrap();
                let obs_client = create_obs_client().await?;
                let twitch_config = get_chat_config();
                let (_, twitch_client) = TwitchIRCClient::<
                    SecureTCPTransport,
                    StaticLoginCredentials,
                >::new(twitch_config);
                event_loop.push(
                    handlers::elevenlabs_handler::ElevenLabsHandler {
                        pool: pool.clone(),
                        twitch_client,
                        sink,
                        obs_client,
                        elevenlabs,
                    },
                );
            }

            "ai_screenshots" => {
                let obs_client = create_obs_client().await?;
                let twitch_config = get_chat_config();
                let (_, twitch_client) = TwitchIRCClient::<
                    SecureTCPTransport,
                    StaticLoginCredentials,
                >::new(twitch_config);
                let sink = rodio::Sink::try_new(&stream_handle).unwrap();
                event_loop.push(
                    handlers::ai_screenshots::AiScreenshotsHandler {
                        obs_client,
                        sink,
                        pool: pool.clone(),
                        twitch_client,
                    },
                );
            }

            "ai_screenshots_timer" => {
                let obs_client = create_obs_client().await?;
                let twitch_config = get_chat_config();
                let (_, twitch_client) = TwitchIRCClient::<
                    SecureTCPTransport,
                    StaticLoginCredentials,
                >::new(twitch_config);
                let sink = rodio::Sink::try_new(&stream_handle).unwrap();
                event_loop.push(
                    handlers::ai_screenshots_timer::AiScreenshotsTimerHandler {
                        obs_client,
                        sink,
                        pool: pool.clone(),
                        twitch_client,
                    },
                );
            }

            "ai_telephone" => {
                let obs_client = create_obs_client().await?;
                let twitch_config = get_chat_config();
                let (_, twitch_client) = TwitchIRCClient::<
                    SecureTCPTransport,
                    StaticLoginCredentials,
                >::new(twitch_config);
                let sink = rodio::Sink::try_new(&stream_handle).unwrap();
                event_loop.push(handlers::ai_telephone::AiTelephoneHandler {
                    obs_client,
                    sink,
                    pool: pool.clone(),
                    twitch_client,
                });
            }

            "ai_scenes" => {
                let elevenlabs_auth = Auth::from_env().unwrap();
                let elevenlabs = Elevenlabs::new(
                    elevenlabs_auth,
                    "https://api.elevenlabs.io/v1/",
                );
                let sink = rodio::Sink::try_new(&stream_handle).unwrap();
                let obs_client = create_obs_client().await?;
                let twitch_config = get_chat_config();
                let (_, twitch_client) = TwitchIRCClient::<
                    SecureTCPTransport,
                    StaticLoginCredentials,
                >::new(twitch_config);
                event_loop.push(handlers::ai_scenes::AiScenesHandler {
                    pool: pool.clone(),
                    twitch_client,
                    sink,
                    obs_client,
                    elevenlabs,
                });

                let obs_client = create_obs_client().await?;
                event_loop.push(
                    handlers::music_scenes_handler::MusicScenesHandler {
                        pool: pool.clone(),
                        obs_client,
                    },
                );
            }

            "channel_rewards" => {
                let obs_client = create_obs_client().await?;
                let twitch_config = get_chat_config();
                let (_, twitch_client) = TwitchIRCClient::<
                    SecureTCPTransport,
                    StaticLoginCredentials,
                >::new(twitch_config);
                event_loop.push(handlers::reward_handler::RewardHandler {
                    obs_client,
                    pool: pool.clone(),
                    twitch_client,
                });
            }

            "skybox" => {
                let obs_client = create_obs_client().await?;
                event_loop.push(handlers::skybox::SkyboxHandler {
                    obs_client,
                    pool: pool.clone(),
                });

                // This checks if Skyboxes are done, every 60 seconds
                let obs_client = create_obs_client().await?;
                event_loop.push(handlers::skybox_status::SkyboxStatusHandler {
                    obs_client,
                    pool: pool.clone(),
                });

                // This checks if Skyboxes are done, every 60 seconds
                let obs_client = create_obs_client().await?;
                let sink = rodio::Sink::try_new(&stream_handle).unwrap();
                let twitch_config = get_chat_config();
                let (_, twitch_client) = TwitchIRCClient::<
                    SecureTCPTransport,
                    StaticLoginCredentials,
                >::new(twitch_config);
                event_loop.push(handlers::skybox::SkyboxRoutingHandler {
                    sink,
                    twitch_client,
                    obs_client,
                    pool: pool.clone(),
                });
            }

            "obs" => {
                // This really is named wrong
                // this handles more than OBS
                // and it's also earlier in the program
                // but it takes an obs_client and pool none-the-less
                let twitch_config = get_chat_config();
                let (_, twitch_client) = TwitchIRCClient::<
                    SecureTCPTransport,
                    StaticLoginCredentials,
                >::new(twitch_config);
                let obs_client = create_obs_client().await?;
                // Do we need to duplicate this?
                let sink = rodio::Sink::try_new(&stream_handle).unwrap();
                event_loop.push(handlers::obs_messages::OBSMessageHandler {
                    obs_client,
                    twitch_client,
                    pool: pool.clone(),
                    sink,
                });

                // OBS Hotkeys are controlled here
                let obs_client = create_obs_client().await?;
                event_loop.push(
                    handlers::trigger_obs_hotkey::TriggerHotkeyHandler {
                        obs_client,
                    },
                );

                // OBS Text is controlled here
                let obs_client = create_obs_client().await?;
                event_loop.push(
                    handlers::transform_obs_test::TransformOBSTextHandler {
                        obs_client,
                    },
                );

                // OBS Sources are controlled here
                let obs_client = create_obs_client().await?;
                event_loop.push(
                    handlers::source_visibility::SourceVisibilityHandler {
                        obs_client,
                    },
                );
                continue;
            }

            "stream_character" => {
                // OBS Stream Characters are controlled here
                let obs_client = create_obs_client().await?;
                event_loop.push(
                    handlers::stream_character_handler::StreamCharacterHandler {
                        obs_client,
                    },
                );
            }

            "chat_gpt_response" => {
                let twitch_config = get_chat_config();
                let (_, twitch_client) = TwitchIRCClient::<
                    SecureTCPTransport,
                    StaticLoginCredentials,
                >::new(twitch_config);
                event_loop.push(
                    handlers::chatgpt_response_handler::ChatGPTResponse {
                        twitch_client,
                    },
                );
            }

            "twitch_eventsub" => {
                println!("Attempting to run Twitch Event Sub");
                // Twitch EventSub Events
                let twitch_config = get_chat_config();
                let (_, twitch_client) = TwitchIRCClient::<
                    SecureTCPTransport,
                    StaticLoginCredentials,
                >::new(twitch_config);
                let obs_client = create_obs_client().await?;
                event_loop.push(
                    handlers::twitch_eventsub_handler::TwitchEventSubHandler {
                        pool: pool.clone(),
                        obs_client,
                        twitch_client,
                    },
                );
            }

            "dynamic_stream_background" => {
                let obs_client = create_obs_client().await?;
                event_loop.push(
                    handlers::stream_background::StreamBackgroundHandler {
                        obs_client,
                    },
                );
            }

            "turbo_bg" => {
                println!("Turbo BG time");
                
                let obs_client = create_obs_client().await?;
                
                
                let twitch_config = get_chat_config();
                let (_, twitch_client) = TwitchIRCClient::<
                    SecureTCPTransport,
                    StaticLoginCredentials,
                >::new(twitch_config);

                let sink = rodio::Sink::try_new(&stream_handle).unwrap();
                // let (sink, mut queue_rx) = rodio::Sink::new_idle();
                // println!("{:?}", queue_rx.next());
                // stream_handle.play_raw(queue_rx)?;
                
                event_loop.push(handlers::fal_handler::FalHandler {
                    // queue_rx: queue_rx,
                    pool: pool.clone(),
                    sink,
                    obs_client,
                    twitch_client,
                });
            }

            "ai_songs" => {
                let obs_client = create_obs_client().await?;
                let sink = rodio::Sink::try_new(&stream_handle).unwrap();
                let twitch_config = get_chat_config();
                let (_, twitch_client) = TwitchIRCClient::<
                    SecureTCPTransport,
                    StaticLoginCredentials,
                >::new(twitch_config);
                event_loop.push(handlers::ai_songs::AISongsHandler {
                    pool: pool.clone(),
                    sink,
                    obs_client,
                    twitch_client,
                });

                let obs_client = create_obs_client().await?;
                let twitch_config = get_chat_config();
                let (_, twitch_client) = TwitchIRCClient::<
                    SecureTCPTransport,
                    StaticLoginCredentials,
                >::new(twitch_config);
                event_loop.push(
                    handlers::ai_songs_downloader::AISongsDownloader {
                        pool: pool.clone(),
                        obs_client,
                        twitch_client,
                    },
                );
            }

            _ => {
                println!("Unknown Feature: {}", feature);
                continue;
            }
        }
    }

    // =======================================================================

    event_loop.run().await?;
    Ok(())
}
