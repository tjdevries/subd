use anyhow::Result;
use rodio::Decoder;
use rodio::Sink;
use std::fs::File;
use std::io::BufReader;
use subd_types::AiScenesRequest;
use twitch_chat::client::send_message;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub async fn trigger_movie_trailer(
    sink: &Sink,
    locked_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    ai_scene_req: &AiScenesRequest,
    local_audio_path: String,
) -> Result<()> {
    if let Some(music_bg) = &ai_scene_req.music_bg {
        let _ = send_message(locked_client, music_bg.clone()).await;
    }

    let file = BufReader::new(File::open(local_audio_path)?);
    sink.append(Decoder::new(BufReader::new(file))?);
    sink.sleep_until_end();

    Ok(())
}
