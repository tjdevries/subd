use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use elevenlabs_api::{Auth, Elevenlabs};
use obs_service::obs::create_obs_client;
use obws::Client;
use serde::{Deserialize, Serialize};
use server::handlers;
use std::collections::HashMap;
use subd_db::get_db_pool;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::{ClientConfig, SecureTCPTransport, TwitchIRCClient};

fn get_chat_config() -> ClientConfig<StaticLoginCredentials> {
    let twitch_username = subd_types::consts::get_twitch_bot_username();
    ClientConfig::new_simple(StaticLoginCredentials::new(
        twitch_username,
        Some(subd_types::consts::get_twitch_bot_oauth()),
    ))
}

// #[derive(Serialize, Deserialize, Debug)]
// struct EventSubRoot {
//     subscription: Subscription,
//     event: Option<EventSub>,
//     challenge: Option<String>,
// }

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

    /// Whether to Enable Specific Features
    #[arg(long, value_delimiter = ' ', num_args = 1..)]
    enable: Vec<String>,
}

struct AppResources {
    obs_client: Client,
    sink: rodio::Sink,
    twitch_client: TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    elevenlabs: Elevenlabs,
}

impl AppResources {
    /// Creates a new instance of `AppResources` with fresh resources.
    async fn new(stream_handle: &rodio::OutputStreamHandle) -> Result<Self> {
        // Initialize OBS client
        let obs_client = create_obs_client().await?;

        // This is how we play audio
        let sink = rodio::Sink::try_new(stream_handle)?;

        // Initialize Twitch client
        let twitch_config = get_chat_config();
        let (_, twitch_client) = TwitchIRCClient::<
            SecureTCPTransport,
            StaticLoginCredentials,
        >::new(twitch_config);

        // Initialize ElevenLabs client
        let elevenlabs_auth = Auth::from_env().unwrap();
        let elevenlabs =
            Elevenlabs::new(elevenlabs_auth, "https://api.elevenlabs.io/v1/");

        Ok(Self {
            obs_client,
            sink,
            twitch_client,
            elevenlabs,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", "\n=== Starting Subd ===\n".cyan());

    {
        use rustrict::{add_word, Type};

        // You must take care not to call these when the crate is being
        // used in any other way (to avoid concurrent mutation).
        unsafe {
            add_word(format!("vs{}", "code").as_str(), Type::PROFANE);
            add_word("vsc*de", Type::SAFE);
        }
    }

    // Create the event loop
    let mut event_loop = events::EventLoop::new();

    let args = Args::parse();

    // Redirect the stdout to cleanup bootup logs
    let fe = subd_utils::redirect_stderr()?;
    let fo = subd_utils::redirect_stdout()?;
    let (_stream, stream_handle) = subd_audio::get_output_stream("pulse")
        .expect("Failed to get audio output stream");
    subd_utils::restore_stderr(fe);
    subd_utils::restore_stdout(fo);

    // Determine which features to enable
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
            "fal".to_string(),
        ]
    } else {
        args.enable
    };

    // Shared resource
    let pool = get_db_pool().await;

    for feature in features {
        match feature.as_str() {
            "twitch_chat_saving" => {
                println!("{}", "Enabling Twitch Chat Saving".green());
                event_loop.push(twitch_chat::client::TwitchChat::new(
                    pool.clone(),
                    "beginbot".to_string(),
                )?);

                // Saves the message and extracts out some information
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

                // Create new resources for this feature
                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(handlers::voices_handler::VoicesHandler {
                    pool: pool.clone(),
                    obs_client: resources.obs_client,
                });
            }

            "implict_soundeffects" => {
                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(
                    handlers::implicit_sound_handler::ImplicitSoundHandler {
                        sink: resources.sink,
                        pool: pool.clone(),
                    },
                );
            }

            "explicit_soundeffects" => {
                println!("{}", "Enabling Explicit Sound Effects".green());
                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(
                    handlers::explicit_sound_handler::ExplicitSoundHandler {
                        sink: resources.sink,
                        pool: pool.clone(),
                    },
                );
            }

            "tts" => {
                println!("{}", "Enabling TTS".green());
                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(
                    handlers::elevenlabs_handler::ElevenLabsHandler {
                        pool: pool.clone(),
                        twitch_client: resources.twitch_client,
                        sink: resources.sink,
                        obs_client: resources.obs_client,
                        elevenlabs: resources.elevenlabs,
                    },
                );
            }

            "ai_screenshots" => {
                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(
                    handlers::ai_screenshots_handler::AiScreenshotsHandler {
                        obs_client: resources.obs_client,
                        sink: resources.sink,
                        pool: pool.clone(),
                        twitch_client: resources.twitch_client,
                    },
                );
            }

            "ai_screenshots_timer" => {
                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(
                    handlers::ai_screenshots_timer_handler::AiScreenshotsTimerHandler {
                        obs_client: resources.obs_client,
                        sink: resources.sink,
                        pool: pool.clone(),
                        twitch_client: resources.twitch_client,
                    },
                );
            }

            "ai_telephone" => {
                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(
                    handlers::ai_telephone_handler::AiTelephoneHandler {
                        obs_client: resources.obs_client,
                        sink: resources.sink,
                        pool: pool.clone(),
                        twitch_client: resources.twitch_client,
                    },
                );
            }

            "ai_scenes" => {
                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(handlers::ai_scenes_handler::AiScenesHandler {
                    pool: pool.clone(),
                    twitch_client: resources.twitch_client,
                    sink: resources.sink,
                    obs_client: resources.obs_client,
                    elevenlabs: resources.elevenlabs,
                });

                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(
                    handlers::music_scenes_handler::MusicScenesHandler {
                        pool: pool.clone(),
                        obs_client: resources.obs_client,
                    },
                );
            }

            "channel_rewards" => {
                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(handlers::reward_handler::RewardHandler {
                    obs_client: resources.obs_client,
                    pool: pool.clone(),
                    twitch_client: resources.twitch_client,
                });
            }

            "skybox" => {
                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(handlers::skybox_handler::SkyboxHandler {
                    obs_client: resources.obs_client,
                    pool: pool.clone(),
                });

                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(
                    handlers::skybox_status_handler::SkyboxStatusHandler {
                        obs_client: resources.obs_client,
                        pool: pool.clone(),
                    },
                );
                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(
                    handlers::skybox_handler::SkyboxRoutingHandler {
                        sink: resources.sink,
                        twitch_client: resources.twitch_client,
                        obs_client: resources.obs_client,
                        pool: pool.clone(),
                    },
                );
            }

            "obs" => {
                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(
                    handlers::obs_messages_handler::OBSMessageHandler {
                        obs_client: resources.obs_client,
                        twitch_client: resources.twitch_client,
                        pool: pool.clone(),
                        sink: resources.sink,
                    },
                );

                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(
                    handlers::trigger_obs_hotkey_handler::TriggerHotkeyHandler {
                        obs_client: resources.obs_client,
                    },
                );

                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(
                    handlers::transform_obs_test_handler::TransformOBSTextHandler {
                        obs_client: resources.obs_client,
                    },
                );

                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(
                    handlers::source_visibility_handler::SourceVisibilityHandler {
                        obs_client: resources.obs_client,
                    },
                );
            }

            "stream_character" => {
                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(
                    handlers::stream_character_handler::StreamCharacterHandler {
                        obs_client: resources.obs_client,
                    },
                );
            }

            "chat_gpt_response" => {
                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(
                    handlers::chatgpt_response_handler::ChatGPTResponse {
                        twitch_client: resources.twitch_client,
                    },
                );
            }

            "twitch_eventsub" => {
                println!("{}", "Enabling Twitch Event Sub".green());
                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(
                    handlers::twitch_eventsub_handler::TwitchEventSubHandler {
                        pool: pool.clone(),
                        obs_client: resources.obs_client,
                        twitch_client: resources.twitch_client,
                    },
                );
            }

            "dynamic_stream_background" => {
                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(
                    handlers::stream_background_handler::StreamBackgroundHandler {
                        obs_client: resources.obs_client,
                    },
                );
            }

            "fal" => {
                println!("{}", "Enabling FalHandler".green());
                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(handlers::fal_handler::FalHandler {
                    pool: pool.clone(),
                    sink: resources.sink,
                    obs_client: resources.obs_client,
                    twitch_client: resources.twitch_client,
                });
            }

            "ai_songs" => {
                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(handlers::ai_songs_handler::AISongsHandler {
                    pool: pool.clone(),
                    sink: resources.sink,
                    obs_client: resources.obs_client,
                    twitch_client: resources.twitch_client,
                });

                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(
                    handlers::ai_songs_downloader_handler::AISongsDownloader {
                        pool: pool.clone(),
                        obs_client: resources.obs_client,
                        twitch_client: resources.twitch_client,
                    },
                );

                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(
                    handlers::ai_music_video_creator_handler::AIMusicVideoCreatorHandler{
                        pool: pool.clone(),
                        obs_client: resources.obs_client,
                        twitch_client: resources.twitch_client,
                    },
                );

                let resources = AppResources::new(&stream_handle).await?;
                event_loop.push(
                    handlers::ai_songs_vote_handler::AISongsVoteHandler {
                        pool: pool.clone(),
                        obs_client: resources.obs_client,
                        twitch_client: resources.twitch_client,
                    },
                );
            }

            _ => {
                println!("Unknown Feature: {}", feature);
            }
        }
    }

    event_loop.run().await?;
    Ok(())
}
