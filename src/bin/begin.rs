use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::time::Duration;
use std::time::SystemTime;
use std::fs::File;
use std::io::BufReader;
use rodio::*;
use renet::*;
use std::io::{BufWriter, Write};
use anyhow::Result;
use renet::transport::ClientAuthentication;
use server::audio;
use server::handlers;
use server::uberduck;
use subd_db::get_db_pool;
use twitch_irc::ClientConfig;
use twitch_irc::SecureTCPTransport;
use twitch_irc::TwitchIRCClient;
use twitch_irc::login::StaticLoginCredentials;
use twitch_chat::send_message;
use renet::transport::NetcodeClientTransport;
use elevenlabs_api::{
  tts::{TtsApi, TtsBody},
  *,
};
use serde_json::json;
use serde::{Deserialize,Serialize};
use std::fs;
use rand::{thread_rng, seq::SliceRandom};
use warp::{http::StatusCode, Filter, Rejection, Reply, reply, reply::json, Buf};
use std::convert::Infallible;
use std::collections::HashMap;
use obws::Client as OBSClient;
use std::sync::{Arc, Mutex};
use subd_types::UberDuckRequest;
use subd_types::Event;


#[derive(Deserialize, Debug)]
struct Voice {
    voice_id: String,
    name: String,
}

#[derive(Deserialize, Debug)]
struct VoiceList {
    voices: Vec<Voice>,
}


fn find_random_voice() -> String {
    let data = fs::read_to_string("voices.json").expect("Unable to read file");
    
    let voice_list: VoiceList = serde_json::from_str(&data).expect("JSON was not well-formatted");
    
    let mut rng = thread_rng();
    let random_voice = voice_list.voices.choose(&mut rng).expect("List of voices is empty");

    println!("Random Voice ID: {}, Name: {}", random_voice.voice_id, random_voice.name);
    return random_voice.voice_id.clone()
}


fn get_chat_config() -> ClientConfig<StaticLoginCredentials> {
    let twitch_username = subd_types::consts::get_twitch_bot_username();
    ClientConfig::new_simple(StaticLoginCredentials::new(
        twitch_username,
        Some(subd_types::consts::get_twitch_bot_oauth()),
    ))
}

// =====================================================================


// {"subscription":{"id":"ddfd7140-1590-1dda-ca56-61aac70f9be1","status":"enabled","type":"channel.follow","version":"1","condition":{"broadcaster_user_id":"10483564"},"transport":{"method":"webhook","callback":"null"},"created_at":"2023-11-20T22:57:37.179519123Z","cost":0},"event":{"user_id":"25578051","user_login":"testFromUser","user_name":"testFromUser","broadcaster_user_id":"10483564","broadcaster_user_login":"10483564","broadcaster_user_name":"testBroadcaster","followed_at":"2023-11-20T22:57:37.179519123Z"}}
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

    // Advice!
    // codyphobe:
    //           For the OBSClient cloning,
    //           could you pass the OBSClient in the constructor when making event_loop,
    //           then pass self.obsclient into each handler's handle method inside
    //           EventLoop#run

    // Create 1 Event Loop
    // Push handles onto the loop
    // those handlers are things like twitch-chat, twitch-sub, github-sponsor etc.
    let mut event_loop = events::EventLoop::new();

    // You can clone this
    // because it's just adding one more connection per clone()???
    //
    // This is useful because you need no lifetimes
    let pool = subd_db::get_db_pool().await;

    // Turns twitch IRC things into our message events
    event_loop.push(twitch_chat::TwitchChat::new(
        pool.clone(),
        "beginbot".to_string(),)?);

    // TODO: Update this description to be more exact
    // Saves the message and extracts out some information
    // for easier routing
    event_loop.push(twitch_chat::TwitchMessageHandler::new(
        pool.clone(),
        twitch_service::Service::new(
            pool.clone(),
            user_service::Service::new(pool.clone()).await,
        )
        .await,
    ));
    
    let twitch_config = get_chat_config();
    let (_, twitch_client) = TwitchIRCClient::<
        SecureTCPTransport,
        StaticLoginCredentials,
    >::new(twitch_config);
    
    // This really is named wrong
    // this handles more than OBS
    // and it's also earlier in the program
    // but it takes an obs_client and pool none-the-less
    let obs_client = server::obs::create_obs_client().await?;
    event_loop.push(handlers::obs_messages::OBSMessageHandler {
        obs_client,
        twitch_client,
        pool: pool.clone(),
    });

    // TODO: This should be abstracted
    // Works for Arch Linux
    let (_stream, stream_handle) =
        audio::get_output_stream("pulse").expect("stream handle");
    
    // Works for Mac
    // let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();

    event_loop.push(handlers::sound_handler::SoundHandler {
        sink,
        pool: pool.clone(),
    });

    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    event_loop.push(handlers::sound_handler::ExplicitSoundHandler {
        sink,
        pool: pool.clone(),
    });

    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    let pool = get_db_pool().await;
    
    let elevenlabs_auth = Auth::from_env().unwrap();
    let elevenlabs = Elevenlabs::new(elevenlabs_auth, "https://api.elevenlabs.io/v1/");

    // Uberduck handles voice messages
    event_loop.push(uberduck::ElevenLabsHandler { pool, sink, elevenlabs });

    // // OBS Hotkeys are controlled here
    let obs_client = server::obs::create_obs_client().await?;
    event_loop.push(handlers::trigger_obs_hotkey::TriggerHotkeyHandler { obs_client });
    //
    // // OBS Text is controlled here
    let obs_client = server::obs::create_obs_client().await?;
    event_loop.push(handlers::transform_obs_test::TransformOBSTextHandler { obs_client });
    //
    // // OBS Sources are controlled here
    let obs_client = server::obs::create_obs_client().await?;
    event_loop.push(handlers::source_visibility::SourceVisibilityHandler { obs_client });
    
    //
    let obs_client = server::obs::create_obs_client().await?;
    event_loop.push(handlers::skybox::SkyboxHandler{ obs_client });
    
    // // OBS Stream Characters are controlled here
    let obs_client = server::obs::create_obs_client().await?;
    event_loop.push(handlers::stream_character_handler::StreamCharacterHandler { obs_client });
    

    // This will also need to be able to access the DB, play sounds, talk to Twitch Chat
    // // OBS Stream Characters are controlled here
    let obs_client = server::obs::create_obs_client().await?;
    event_loop.push(handlers::twitch_eventsub_handler::TwitchEventSubHandler { obs_client });
    
    // =======================================================================
    
    // EVENT SUB EXPERIMENTS
    // let route2 = warp::body::content_length_limit(1024 * 1282)
    // .and(warp::body::json())
    // .map(|simple_map: EventSubRoot| {
    //     // Why are we failing in here?
    //     // It's probably because of differnet shapes of data?
    //
    //     // server::obs_source::set_enabled(
    //     //     "Primary",
    //     //     "Dalle-Gen-1",
    //     //     true,
    //     //     &obs_client,
    //     // ).await;
    //         
    //     println!("simple_map = {:?}", simple_map);
    //     let challenge = match simple_map.challenge {
    //         Some(challenge) => {
    //                 challenge
    //         }
    //         _ =>  {
    //                 // Here we need our own OBS Client
    //                 // and postgres pool
    //                 // So we just have to pass this object around!
    //                 // So now we want to handle this message!
    //                 "".to_string()
    //         }
    //         // return warp::reply::with_status(simple_map.challenge, warp::http::StatusCode::OK)
    //     };
    //     return warp::reply::with_status(challenge, warp::http::StatusCode::OK)
    //     // warp::reply::with_status(simple_map.challenge, warp::http::StatusCode::OK)
    // });
    // 
    // 
    // let route3 = warp::body::content_length_limit(1024 * 1282)
    // .and(warp::body::json()).and_then(post_request);
    // 
    // // Run the Warp server in a separate async task
    // tokio::spawn(async move {
    //     warp::serve(route3).run(([0, 0, 0, 0], 8080)).await;
    // });

    // =======================================================================

    event_loop.run().await?;
    Ok(())
}

pub async fn test() -> Result<()> {
    Ok(())
}

// How would I get a tx in here!
async fn post_request(simple_map: EventSubRoot) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    // println!("We hit post_request");
    // Ok(Box::new("world"))
    // return Ok(reply::with_status(json(&"".to_string()), StatusCode::OK));
    println!("simple_map = {:?}", simple_map);
    let challenge = match simple_map.challenge {
        Some(challenge) => {
                challenge
        }
        _ =>  {
                let obs_client = server::obs::create_obs_client().await.unwrap();
                server::obs_source::set_enabled(
                    "Primary",
                    "Dalle-Gen-1",
                    true,
                    &obs_client,
                ).await;
            
            // let _ = tx.send(Event::UberDuckRequest(UberDuckRequest {
            //         message: "woah there budy".to_string(),
            //         voice_text: "waoh there buddy".to_string(),
            //         username: "beginbot".to_string(),
            // voice: "Ethan".to_string(),
            //         source: None,
            //     }));
                // Here we need our own OBS Client
                // and postgres pool
                // So we just have to pass this object around!
                // So now we want to handle this message!
                "".to_string()
        }
        // return warp::reply::with_status(simple_map.challenge, warp::http::StatusCode::OK)
    };
    
    // return warp::reply::with_status(challenge, warp::http::StatusCode::OK)
    return Ok(Box::new(reply::with_status(challenge, StatusCode::OK)));
}

