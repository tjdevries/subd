use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde_json::Value;

// TODO: update to actually validate twitch messages are from Twitch
// use hmac::Hmac;
// use sha2::Sha256;
// type HmacSha256 = Hmac<Sha256>;

async fn eventsub_get(req: HttpRequest) -> impl Responder {
    println!("Request Info: {:?}", req);
    HttpResponse::Ok().json("")
}

pub async fn eventsub_post(
    req: HttpRequest,
    body: web::Bytes,
) -> impl Responder {
    // let secret = get_secret();
    // let message = get_hmac_message(&req, &body);
    // let mut mac = HmacSha256::new_varkey(secret.as_bytes()).expect("HMAC can take key of any size");
    // mac.update(message.as_bytes());
    // let hmac_hex = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

    //if verify_message(&hmac_hex, req.headers().get("Twitch-Eventsub-Message-Signature").unwrap().to_str().unwrap()) {
    println!("signatures match");
    let notification: Value = match serde_json::from_slice(&body) {
        Ok(value) => value,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match req
        .headers()
        .get("Twitch-Eventsub-Message-Type")
        .and_then(|header| header.to_str().ok())
    {
        Some("notification") => {
            // Process the event's data
            if let Some(event_type) =
                notification["subscription"]["type"].as_str()
            {
                println!("Event type: {}", event_type);
            }
            HttpResponse::NoContent().finish()
        }
        Some("webhook_callback_verification") => {
            if let Some(challenge) = notification["challenge"].as_str() {
                HttpResponse::Ok()
                    .content_type("text/plain")
                    .body(challenge.to_string())
            } else {
                HttpResponse::BadRequest().finish()
            }
        }
        Some("revocation") => {
            println!("notifications revoked!");
            HttpResponse::NoContent().finish()
        }
        _ => HttpResponse::BadRequest().finish(),
    }
}

// fn get_secret() -> String {
//     "hjv0yajkagdn90bha8x7btj07zu54h".to_string() // Replace with your actual secret
// }

// fn get_hmac_message(req: &HttpRequest, body: &web::Bytes) -> String {
//     let id = req.headers().get("Twitch-Eventsub-Message-Id").unwrap().to_str().unwrap();
//     let timestamp = req.headers().get("Twitch-Eventsub-Message-Timestamp").unwrap().to_str().unwrap();
//     format!("{}{}{}", id, timestamp, str::from_utf8(body).unwrap())
// }
//
// fn verify_message(hmac_hex: &str, verify_signature: &str) -> bool {
//     hmac_hex == verify_signature
// }

// #[actix_rt::main]
pub async fn kickoff_webhook() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/eventsub", web::get().to(eventsub_get))
            .route("/eventsub", web::post().to(eventsub_post))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
