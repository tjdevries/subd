use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use elevenlabs_api::{Auth, Elevenlabs};
use obs_service::obs::create_obs_client;
use obws::Client;
use serde::{Deserialize, Serialize};
use server::handlers;
use std::collections::HashMap;
use std::env;
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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    enable_all: bool,

    #[arg(long, value_delimiter = ' ', num_args = 1..)]
    enable: Vec<String>,
}

struct AppResources {
    obs_client: Option<Client>,
    sink: rodio::Sink,
    twitch_client: TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    elevenlabs: Elevenlabs,
}

impl AppResources {
    async fn new(stream_handle: &rodio::OutputStreamHandle) -> Result<Self> {
        let obs_client = create_obs_client().await.ok();
        let sink = rodio::Sink::try_new(stream_handle)?;
        let twitch_config = get_chat_config();
        let (_, twitch_client) = TwitchIRCClient::<
            SecureTCPTransport,
            StaticLoginCredentials,
        >::new(twitch_config);
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

    unsafe {
        rustrict::add_word(
            format!("vs{}", "code").as_str(),
            rustrict::Type::PROFANE,
        );
        rustrict::add_word("vsc*de", rustrict::Type::SAFE);
    }

    let mut event_loop = events::EventLoop::new();
    let args = Args::parse();

    let (_stream, stream_handle) = create_audio_stream()?;
    let pool = get_db_pool().await;
    let features = get_features(&args);

    for feature in features {
        add_feature_to_event_loop(
            &mut event_loop,
            &feature,
            &pool,
            &stream_handle,
        )
        .await?;
    }

    event_loop.run().await
}

fn create_audio_stream(
) -> Result<(rodio::OutputStream, rodio::OutputStreamHandle)> {
    if env::consts::OS == "macos" {
        rodio::OutputStream::try_default().map_err(Into::into)
    } else {
        let fe = subd_utils::redirect_stderr()?;
        let fo = subd_utils::redirect_stdout()?;
        let result = subd_audio::get_output_stream("pulse")
            .map_err(|_| anyhow::anyhow!("Failed to get audio output stream"));
        subd_utils::restore_stderr(fe);
        subd_utils::restore_stdout(fo);
        result
    }
}

fn get_features(args: &Args) -> Vec<String> {
    if args.enable_all {
        vec![
            "implict_soundeffects",
            "explicit_soundeffects",
            "voices",
            "ai_screenshots",
            "ai_telephone",
            "ai_scenes",
            "skybox",
            "obs",
            "twitch_chat_saving",
            "stream_character",
            "chat_gpt_response",
            "twitch_eventsub",
            "dynamic_stream_background",
            "channel_rewards",
            "ai_songs",
            "ai_videos",
            "fal",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    } else {
        args.enable.clone()
    }
}

async fn add_feature_to_event_loop(
    event_loop: &mut events::EventLoop,
    feature: &str,
    pool: &sqlx::PgPool,
    stream_handle: &rodio::OutputStreamHandle,
) -> Result<()> {
    match feature {
        "twitch_chat_saving" => {
            add_twitch_chat_saving(event_loop, pool).await?
        }
        "voices" => add_voices_control(event_loop, pool, stream_handle).await?,
        "implict_soundeffects" => {
            add_implicit_sound_effects(event_loop, pool, stream_handle).await?
        }
        "explicit_soundeffects" => {
            add_explicit_sound_effects(event_loop, pool, stream_handle).await?
        }
        "tts" => add_tts(event_loop, pool, stream_handle).await?,
        "ai_screenshots" => {
            add_ai_screenshots(event_loop, pool, stream_handle).await?
        }
        "ai_screenshots_timer" => {
            add_ai_screenshots_timer(event_loop, pool, stream_handle).await?
        }
        "ai_telephone" => {
            add_ai_telephone(event_loop, pool, stream_handle).await?
        }
        "ai_scenes" => add_ai_scenes(event_loop, pool, stream_handle).await?,
        "channel_rewards" => {
            add_channel_rewards(event_loop, pool, stream_handle).await?
        }
        "skybox" => add_skybox(event_loop, pool, stream_handle).await?,
        "obs" => add_obs(event_loop, pool, stream_handle).await?,
        "stream_character" => {
            add_stream_character(event_loop, stream_handle).await?
        }
        "chat_gpt_response" => {
            add_chat_gpt_response(event_loop, stream_handle).await?
        }
        "twitch_eventsub" => {
            add_twitch_eventsub(event_loop, pool, stream_handle).await?
        }
        "dynamic_stream_background" => {
            add_dynamic_stream_background(event_loop, stream_handle).await?
        }
        "fal" => add_fal(event_loop, pool, stream_handle).await?,
        "ai_videos" => add_ai_videos(event_loop, pool, stream_handle).await?,
        "ai_songs" => add_ai_songs(event_loop, pool, stream_handle).await?,
        _ => println!("Unknown Feature: {}", feature),
    }
    Ok(())
}

async fn add_twitch_chat_saving(
    event_loop: &mut events::EventLoop,
    pool: &sqlx::PgPool,
) -> Result<()> {
    println!("{}", "Enabling Twitch Chat Saving".green());
    event_loop.push(twitch_chat::client::TwitchChat::new(
        pool.clone(),
        "beginbot".to_string(),
    )?);
    event_loop.push(twitch_chat::handlers::TwitchMessageHandler::new(
        pool.clone(),
        twitch_service::Service::new(
            pool.clone(),
            user_service::Service::new(pool.clone()).await,
        )
        .await,
    ));
    Ok(())
}

async fn add_voices_control(
    event_loop: &mut events::EventLoop,
    pool: &sqlx::PgPool,
    stream_handle: &rodio::OutputStreamHandle,
) -> Result<()> {
    println!("{}", "Enabling Voices control".green());
    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(handlers::voices_handler::VoicesHandler {
        pool: pool.clone(),
        obs_client: resources.obs_client.unwrap(),
    });
    Ok(())
}

async fn add_implicit_sound_effects(
    event_loop: &mut events::EventLoop,
    pool: &sqlx::PgPool,
    stream_handle: &rodio::OutputStreamHandle,
) -> Result<()> {
    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(handlers::implicit_sound_handler::ImplicitSoundHandler {
        sink: resources.sink,
        pool: pool.clone(),
    });
    Ok(())
}

async fn add_explicit_sound_effects(
    event_loop: &mut events::EventLoop,
    pool: &sqlx::PgPool,
    stream_handle: &rodio::OutputStreamHandle,
) -> Result<()> {
    println!("{}", "Enabling Explicit Sound Effects".green());
    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(handlers::explicit_sound_handler::ExplicitSoundHandler {
        sink: resources.sink,
        pool: pool.clone(),
    });
    Ok(())
}

async fn add_tts(
    event_loop: &mut events::EventLoop,
    pool: &sqlx::PgPool,
    stream_handle: &rodio::OutputStreamHandle,
) -> Result<()> {
    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(handlers::elevenlabs_handler::ElevenLabsHandler {
        pool: pool.clone(),
        twitch_client: resources.twitch_client,
        sink: resources.sink,
        obs_client: resources.obs_client.unwrap(),
        elevenlabs: resources.elevenlabs,
    });
    Ok(())
}

async fn add_ai_screenshots(
    event_loop: &mut events::EventLoop,
    pool: &sqlx::PgPool,
    stream_handle: &rodio::OutputStreamHandle,
) -> Result<()> {
    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(handlers::ai_screenshots_handler::AiScreenshotsHandler {
        obs_client: resources.obs_client.unwrap(),
        sink: resources.sink,
        pool: pool.clone(),
        twitch_client: resources.twitch_client,
    });
    Ok(())
}

async fn add_ai_screenshots_timer(
    event_loop: &mut events::EventLoop,
    pool: &sqlx::PgPool,
    stream_handle: &rodio::OutputStreamHandle,
) -> Result<()> {
    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(
        handlers::ai_screenshots_timer_handler::AiScreenshotsTimerHandler {
            obs_client: resources.obs_client.unwrap(),
            sink: resources.sink,
            pool: pool.clone(),
            twitch_client: resources.twitch_client,
        },
    );
    Ok(())
}

async fn add_ai_telephone(
    event_loop: &mut events::EventLoop,
    pool: &sqlx::PgPool,
    stream_handle: &rodio::OutputStreamHandle,
) -> Result<()> {
    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(handlers::ai_telephone_handler::AiTelephoneHandler {
        obs_client: resources.obs_client.unwrap(),
        sink: resources.sink,
        pool: pool.clone(),
        twitch_client: resources.twitch_client,
    });
    Ok(())
}

async fn add_ai_scenes(
    event_loop: &mut events::EventLoop,
    pool: &sqlx::PgPool,
    stream_handle: &rodio::OutputStreamHandle,
) -> Result<()> {
    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(handlers::ai_scenes_handler::AiScenesHandler {
        pool: pool.clone(),
        twitch_client: resources.twitch_client,
        sink: resources.sink,
        obs_client: resources.obs_client.unwrap(),
        elevenlabs: resources.elevenlabs,
    });

    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(handlers::music_scenes_handler::MusicScenesHandler {
        pool: pool.clone(),
        obs_client: resources.obs_client.unwrap(),
    });
    Ok(())
}

async fn add_channel_rewards(
    event_loop: &mut events::EventLoop,
    pool: &sqlx::PgPool,
    stream_handle: &rodio::OutputStreamHandle,
) -> Result<()> {
    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(handlers::reward_handler::RewardHandler {
        obs_client: resources.obs_client.unwrap(),
        pool: pool.clone(),
        twitch_client: resources.twitch_client,
    });
    Ok(())
}

async fn add_skybox(
    event_loop: &mut events::EventLoop,
    pool: &sqlx::PgPool,
    stream_handle: &rodio::OutputStreamHandle,
) -> Result<()> {
    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(handlers::skybox_handler::SkyboxHandler {
        obs_client: resources.obs_client.unwrap(),
        pool: pool.clone(),
    });

    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(handlers::skybox_status_handler::SkyboxStatusHandler {
        obs_client: resources.obs_client.unwrap(),
        pool: pool.clone(),
    });

    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(handlers::skybox_handler::SkyboxRoutingHandler {
        sink: resources.sink,
        twitch_client: resources.twitch_client,
        obs_client: resources.obs_client.unwrap(),
        pool: pool.clone(),
    });
    Ok(())
}

async fn add_obs(
    event_loop: &mut events::EventLoop,
    pool: &sqlx::PgPool,
    stream_handle: &rodio::OutputStreamHandle,
) -> Result<()> {
    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(handlers::obs_messages_handler::OBSMessageHandler {
        obs_client: resources.obs_client.unwrap(),
        twitch_client: resources.twitch_client,
        pool: pool.clone(),
        sink: resources.sink,
    });

    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(
        handlers::trigger_obs_hotkey_handler::TriggerHotkeyHandler {
            obs_client: resources.obs_client.unwrap(),
        },
    );

    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(
        handlers::transform_obs_test_handler::TransformOBSTextHandler {
            obs_client: resources.obs_client.unwrap(),
        },
    );

    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(
        handlers::source_visibility_handler::SourceVisibilityHandler {
            obs_client: resources.obs_client.unwrap(),
        },
    );
    Ok(())
}

async fn add_stream_character(
    event_loop: &mut events::EventLoop,
    stream_handle: &rodio::OutputStreamHandle,
) -> Result<()> {
    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(
        handlers::stream_character_handler::StreamCharacterHandler {
            obs_client: resources.obs_client.unwrap(),
        },
    );
    Ok(())
}

async fn add_chat_gpt_response(
    event_loop: &mut events::EventLoop,
    stream_handle: &rodio::OutputStreamHandle,
) -> Result<()> {
    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(handlers::chatgpt_response_handler::ChatGPTResponse {
        twitch_client: resources.twitch_client,
    });
    Ok(())
}

async fn add_twitch_eventsub(
    event_loop: &mut events::EventLoop,
    pool: &sqlx::PgPool,
    stream_handle: &rodio::OutputStreamHandle,
) -> Result<()> {
    println!("{}", "Enabling Twitch Event Sub".green());
    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(handlers::twitch_eventsub_handler::TwitchEventSubHandler {
        pool: pool.clone(),
        obs_client: resources.obs_client.unwrap(),
        twitch_client: resources.twitch_client,
    });
    Ok(())
}

async fn add_dynamic_stream_background(
    event_loop: &mut events::EventLoop,
    stream_handle: &rodio::OutputStreamHandle,
) -> Result<()> {
    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(
        handlers::stream_background_handler::StreamBackgroundHandler {
            obs_client: resources.obs_client.unwrap(),
        },
    );
    Ok(())
}

async fn add_fal(
    event_loop: &mut events::EventLoop,
    pool: &sqlx::PgPool,
    stream_handle: &rodio::OutputStreamHandle,
) -> Result<()> {
    println!("{}", "Enabling FalHandler".green());
    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(handlers::fal_handler::FalHandler {
        pool: pool.clone(),
        sink: resources.sink,
        obs_client: resources.obs_client.unwrap(),
        twitch_client: resources.twitch_client,
    });
    Ok(())
}

async fn add_ai_videos(
    event_loop: &mut events::EventLoop,
    pool: &sqlx::PgPool,
    stream_handle: &rodio::OutputStreamHandle,
) -> Result<()> {
    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(
        handlers::ai_music_video_creator_handler::AIMusicVideoCreatorHandler {
            pool: pool.clone(),
            obs_client: resources.obs_client.unwrap(),
            twitch_client: resources.twitch_client,
        },
    );
    Ok(())
}

async fn add_ai_songs(
    event_loop: &mut events::EventLoop,
    pool: &sqlx::PgPool,
    stream_handle: &rodio::OutputStreamHandle,
) -> Result<()> {
    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(handlers::ai_songs_handler::AISongsHandler {
        pool: pool.clone(),
        sink: resources.sink,
        twitch_client: resources.twitch_client,
    });

    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(handlers::ai_songs_downloader_handler::AISongsDownloader {
        pool: pool.clone(),
        twitch_client: resources.twitch_client,
    });

    let resources = AppResources::new(stream_handle).await?;
    event_loop.push(handlers::ai_songs_vote_handler::AISongsVoteHandler {
        pool: pool.clone(),
        twitch_client: resources.twitch_client,
    });
    Ok(())
}
