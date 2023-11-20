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

fn test() {
    println!("\n\n\t\t=== Test Time \n\n");
    let mut client = RenetClient::new(ConnectionConfig::default());

    // Setup transport layer
    const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);

    //
    // This :0 is contenous
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let client_id: u64 = 0;
    let authentication = ClientAuthentication::Unsecure {
        server_addr: SERVER_ADDR,
        client_id,
        user_data: None,
        protocol_id: 0,
    };

    let mut transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

    // Your gameplay loop
    let delta_time = Duration::from_millis(16);
    // Receive new messages and update client
    client.update(delta_time);
    transport.update(delta_time, &mut client).unwrap();
    
    if client.is_disconnected() {
        // Receive message from server
        while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
            println!("\t\tReceived message: {:?}", message);
            // Handle received message
        }
        
        // Send message
        client.send_message(DefaultChannel::ReliableOrdered, "client text".as_bytes().to_vec());
    }
 
    // Send packets to server
    transport.send_packets(&mut client);
}

// =====================================================================
// 

// {"subscription":{"id":"bd747c26-73ab-7557-db35-6c227d58f0a5","status":"enabled","type":"channel.subscribe","version":"1","condition":{"broadcaster_user_id":"34493369"},"transport":{"method":"webhook","callback":"null"},"created_at":"2023-11-20T06:27:11.465026932Z","cost":0},"event":{"user_id":"52495161","user_login":"testFromUser","user_name":"testFromUser","broadcaster_user_id":"34493369","broadcaster_user_login":"testBroadcaster","broadcaster_user_name":"testBroadcaster","tier":"1000","is_gift":false}}
#[derive(Deserialize, Serialize)]
struct MyData {
    // TODO: Upate this shit
    // field1: String,
    // field2: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct EventSubRoot {
    subscription: Subscription,
    // event: Event,
    challenge: String,
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
struct Event {
    user_id: String,
    user_login: String,
    user_name: String,
    broadcaster_user_id: String,
    broadcaster_user_login: String,
    broadcaster_user_name: String,
    tier: Option<String>,
    is_gift: Option<bool>,
}

// ✗ Server Said: Request body deserialize error: missing field `tier` at line 1 column 540

async fn get_request() -> Result<impl Reply, Rejection> {
    println!("BACK AGAIN!!!");
    Ok(json(&"GET response"))
}
// Define a struct that can be serialized to JSON
#[derive(Serialize, Deserialize)]
struct EventSubResponse{
    message: String,
    // Add other fields as needed
}

// async fn post_request(body: Root) -> Result<impl Reply, Rejection> {
// async fn post_request(body: Root) -> Result<warp::Reply> {

async fn post_request(words: String) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    println!("We hit post_request");
    Ok(Box::new("world"))
    // return Ok(reply::with_status(json(&"".to_string()), StatusCode::OK));
    

    // // We should be getting messages here w/ headers as well
    // // Twitch-Eventsub-Message-Type: webhook_callback_verification
    //     
    // // Create an instance of MyData and set the message field
    // let my_data = EventSubResponse {
    //     message: "Hello, JSON!".to_string(),
    //     // Set other fields if needed
    // };
    //
    // // Serialize the MyData struct to a JSON string
    // let json_string = serde_json::to_string(&my_data).unwrap();
    //
    // // Create a JSON reply with the serialized JSON string and a status code
    // let json_reply = reply::with_status(json(&json_string), StatusCode::OK);   println!("In Post Request");
    //
    // let challenge = body.get("challenge").unwrap_or_default();
    // // Respond with the challenge as text/plain
    // return warp::reply::with_header(challenge, warp::http::header::CONTENT_TYPE, "text/plain");
    //     
    // // return Ok(reply::with_status(json(&"".to_string()), StatusCode::OK));
    // 
    // // Is this ok?
    // let obs_client = server::obs::create_obs_client().await.unwrap();
    // println!("Received body: {:?}", body);
    //
    // // So we just hit this!!!!!!
    // println!("BACK POST AGAIN!!!: {:?}", body);
    // println!("Type Field {:?} | User: {}", body.subscription.type_field, body.event.user_name);
    //
    // // let obs_client_inner = obs_client.lock().unwrap();
    // 
    // match body.subscription.type_field.as_str() {
    //     "channel.cheer" => {
    //         println!("WER ARE FUDDCKING CHERERINGKV");
    //         
    //         server::obs_source::set_enabled(
    //             "Primary",
    //             "Dalle-Gen-1",
    //             true,
    //             &obs_client,
    //         ).await;
    //         
    //     },
    //     _ => {
    //         println!("WOOO ABOUT TO DIE!!");
    //         // todo!();
    //     }
    //     
    // }
    // 
    // Ok(reply::with_status(json(&"".to_string()), StatusCode::OK))
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

    // =======================================================================
    
    let get_route = warp::get()
        .and(warp::path("eventsub"))
        .and_then(get_request);
    
    // let verification_route = warp::path!("eventsub")
    //     .and(warp::header::exact_ignore_case("Twitch-Eventsub-Message-Type", "webhook_callback_verification"))
    //     .and(warp::body::bytes())
    //     .map(|body: Bytes| {
    //         let body_bytes = body.to_vec();
    //
    //         let challenge = String::from_utf8_lossy(&body_bytes);
    //
    //         // Respond with the challenge as text/plain
    //         warp::reply::with_header(
    //             challenge,
    //             warp::http::header::CONTENT_TYPE,
    //             "text/plain",
    //         )
    //     });
    
    // let obs_client = Arc::new(Mutex::new(OBSClient::connect("localhost", 4444, Some("")).await.unwrap()));
    // let obs_client_filter = warp::any().map(move || obs_client.clone());

    // Match when we get `accept: */*` exactly.
    // let accept_challenges = warp::header::exact("Twitch-Eventsub-Message-Type", "webhook_callback_verification");
    
    //
    // let routes = host
    //     .and(accept_stars)
    //     .map(|addr| format!("accepting stars on {}", addr));
    
     // warp::path::param()
    // we need the headers to matched
    let post_route = warp::post()
        .and(warp::path("eventsub"))
        .and(warp::body::form())
        .and_then(post_request);
        // .and(accept_challenges)
        // .and(obs_client_filter)
    

    
      // let verification_route = warp::path!("webhook")
      //   .and(warp::header::exact_ignore_case("Twitch-Eventsub-Message-Type", "webhook_callback_verification"))
      //   .and(warp::body::bytes())
      //   .map(handle_verification);
    
    // let warp_routes = get_route.or(post_route);
    // let routes = warp::post()
    //     .and(warp::path("webhook"))
    //     .and(warp::body::bytes())
    //     .map(|body: warp::hyper::body::Bytes| {
    //         warp::reply::with_status(body, warp::http::StatusCode::OK)
    //     });
    
    // =======================================================================

    
// bytes = b"{\"challenge\":\"942c9a68-e9c5-a654-3cd7-5d3144c3caea\",\"subscription\":{\"id\":\"556226ed-f217-f293-f63f-2209dc48333f\",\"status\":\"webhook_callback_verification_pending\",\"type\":\"channel.cheer\",\"version\":\"1\",\"condition\":{\"broadcaster_user_id\":\"33608266\"},\"transport\":{\"method\":\"webhook\",\"callback\":\"https://7b15-47-151-148-79.ngrok-free.app/eventsub\"},\"created_at\":\"2023-11-20T22:16:36.029649097Z\",\"cost\":0}}"
    
    let route = warp::body::content_length_limit(1024 * 32)
    .and(warp::body::bytes())
    .map(|bytes: warp::hyper::body::Bytes| {
        println!("bytes = {:?}", bytes);
        warp::reply::with_status("", warp::http::StatusCode::OK)
            // This needs to reply is all
    });
    
    let route2 = warp::body::content_length_limit(1024 * 32)
    .and(warp::body::json())
    .map(|simple_map: EventSubRoot| {
        println!("simple_map = {:?}", simple_map);
        warp::reply::with_status(simple_map.challenge, warp::http::StatusCode::OK)
    });
    
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));
    
    // let routes = route.or(hello);

    warp::serve(route2).run(([0, 0, 0, 0], 8080)).await;
    
    // Run the Warp server in a separate async task
    // tokio::spawn(async move {
    //     warp::serve(hello).run(([0, 0, 0, 0], 8080)).await;
    // });

    // =======================================================================

    event_loop.run().await?;
    Ok(())
}

// async fn handle_verification(body: Bytes) -> Result<impl warp::Reply, warp::Rejection> {
//     // Convert the bytes to a string
//     let challenge = String::from_utf8_lossy(&body);
//
//     // Respond with the challenge as text/plain
//     Ok(warp::reply::with_header(
//         challenge,
//         warp::http::header::CONTENT_TYPE,
//         "text/plain",
//     ))
// }

// async fn handle_verification(body: warp::hyper::body::Bytes) -> impl warp::Reply {
//     // Convert the bytes to a string
//     let body_bytes = body.to_vec();
//
//     // Convert the bytes to a string
//     let challenge = String::from_utf8_lossy(&body_bytes);
//     // Respond with the challenge as text/plain
//     warp::reply::with_header(challenge, warp::http::header::CONTENT_TYPE, "text/plain")
// }


async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    if err.is_not_found() {
        Ok(reply::with_status("NOT FOUND", StatusCode::NOT_FOUND))
    } else {
        // log error
        Ok(reply::with_status("INTERNAL SERVER ERROR", StatusCode::INTERNAL_SERVER_ERROR))
    }
}
