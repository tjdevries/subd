use anyhow::Result;
use rodio::Decoder;
use std::fs::File;
use std::io::BufReader;
use subd_types;
use subd_types::AiScenesRequest;
use twitch_chat::client::send_message;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

// This should be in the move_trailer command
pub async fn trigger_movie_trailer(
    ai_scene_req: &AiScenesRequest,
    locked_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    local_audio_path: String,
) -> Result<()> {
    if let Some(music_bg) = &ai_scene_req.music_bg {
        let _ = send_message(&locked_client, music_bg.clone()).await;
    }

    // We are supressing a whole bunch of alsa message
    // let backup =
    //     redirect::redirect_stderr().expect("Failed to redirect stderr");

    let (_stream, stream_handle) =
        subd_audio::get_output_stream("pulse").expect("stream handle");
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    let file = BufReader::new(File::open(local_audio_path)?);
    sink.append(Decoder::new(BufReader::new(file))?);
    sink.sleep_until_end();

    // redirect::restore_stderr(backup);

    return Ok(());
}
