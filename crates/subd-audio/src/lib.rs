use anyhow::Result;
use std::process::Command;
use subd_types::AiScenesRequest;

use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::OutputStream;
use rodio::*;
use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::time;

// This also has a pause in it,
// we might want to take that in as a variable
pub async fn play_sound(sink: &Sink, soundname: String) -> Result<()> {
    let file = BufReader::new(
        File::open(format!("./MP3s/{}.mp3", soundname)).unwrap(),
    );
    let sleep_time = time::Duration::from_millis(1000);
    thread::sleep(sleep_time);
    // To tell me a screen shot is coming
    sink.set_volume(0.3);
    sink.append(Decoder::new(BufReader::new(file)).unwrap());
    sink.sleep_until_end();
    Ok(())
}

pub fn get_output_stream(
    device_name: &str,
) -> Result<(OutputStream, OutputStreamHandle)> {
    let host = cpal::default_host();
    let devices = host.output_devices().unwrap();

    for device in devices {
        println!("Device found: {}", device.name().unwrap());
    }

    let devices = host.output_devices().unwrap();
    for device in devices {
        let dev: rodio::Device = device.into();
        let dev_name: String = dev.name().unwrap();
        if dev_name == device_name {
            println!("Device found: {}", dev_name);
            return Ok(OutputStream::try_from_device(&dev).unwrap());
        }
    }

    return Err(anyhow::anyhow!("Device not found"));
}

// ============== //
// Audio Settings //
// ============== //

pub async fn set_audio_status(
    _obs_conn: &obws::Client,
    _name: &str,
    _status: bool,
) -> Result<()> {
    // obs_conn.sources().(name, !status).await?;
    Ok(())
}
pub fn add_voice_modifiers(
    req: &AiScenesRequest,
    voice: String,
    mut local_audio_path: String,
) -> Result<String> {
    if req.reverb {
        local_audio_path = normalize_tts_file(local_audio_path.clone())?;
        local_audio_path = add_reverb(local_audio_path.clone())?;
    }

    match &req.stretch {
        Some(stretch) => {
            local_audio_path =
                normalize_tts_file(local_audio_path.clone()).unwrap();
            local_audio_path =
                stretch_audio(local_audio_path, stretch.to_owned())?;
        }
        None => {}
    }

    match &req.pitch {
        Some(pitch) => {
            local_audio_path = normalize_tts_file(local_audio_path.clone())?;
            local_audio_path =
                change_pitch(local_audio_path, pitch.to_owned())?;
        }
        None => {}
    }

    if voice == "evil_pokimane" {
        local_audio_path = normalize_tts_file(local_audio_path.clone())?;
        local_audio_path = change_pitch(local_audio_path, "-200".to_string())?;
        local_audio_path = add_reverb(local_audio_path.clone())?;
    }

    if voice == "satan" {
        local_audio_path = normalize_tts_file(local_audio_path.clone())?;
        local_audio_path = change_pitch(local_audio_path, "-350".to_string())?;
        local_audio_path = add_reverb(local_audio_path.clone())?;
    }

    if voice == "god" {
        local_audio_path = normalize_tts_file(local_audio_path.clone())?;
        local_audio_path = add_reverb(local_audio_path)?;
    }

    return Ok(local_audio_path);
}

fn normalize_tts_file(local_audio_path: String) -> Result<String> {
    let audio_dest_path =
        add_postfix_to_filepath(local_audio_path.clone(), "_norm".to_string());
    let ffmpeg_status = Command::new("ffmpeg")
        .args(&["-i", &local_audio_path, &audio_dest_path])
        .status()
        .expect("Failed to execute ffmpeg");

    if ffmpeg_status.success() {
        Ok(audio_dest_path)
    } else {
        println!("Failed to normalize audio");
        Ok(local_audio_path)
    }
}

fn stretch_audio(local_audio_path: String, stretch: String) -> Result<String> {
    let audio_dest_path = add_postfix_to_filepath(
        local_audio_path.clone(),
        "_stretch".to_string(),
    );
    Command::new("sox")
        .args(&[
            "-t",
            "wav",
            &local_audio_path,
            &audio_dest_path,
            "stretch",
            &stretch,
        ])
        .status()
        .expect("Failed to execute sox");
    Ok(audio_dest_path)
}

fn change_pitch(local_audio_path: String, pitch: String) -> Result<String> {
    let postfix = format!("{}_{}", "_pitch".to_string(), pitch);
    let audio_dest_path =
        add_postfix_to_filepath(local_audio_path.clone(), postfix);
    Command::new("sox")
        .args(&[
            "-t",
            "wav",
            &local_audio_path,
            &audio_dest_path,
            "pitch",
            &pitch,
        ])
        .status()
        .expect("Failed to execute sox");

    Ok(audio_dest_path)
}

fn add_reverb(local_audio_path: String) -> Result<String> {
    let audio_dest_path = add_postfix_to_filepath(
        local_audio_path.clone(),
        "_reverb".to_string(),
    );
    Command::new("sox")
        .args(&[
            "-t",
            "wav",
            &local_audio_path,
            &audio_dest_path,
            "gain",
            "-2",
            "reverb",
            "70",
            "100",
            "50",
            "100",
            "10",
            "2",
        ])
        .status()
        .expect("Failed to execute sox");
    Ok(audio_dest_path)
}

// this belongs in some sort of filepath utils crate
fn add_postfix_to_filepath(filepath: String, postfix: String) -> String {
    match filepath.rfind('.') {
        Some(index) => {
            let path = filepath[..index].to_string();
            let filename = filepath[index..].to_string();
            format!("{}{}{}", path, postfix, filename)
        }
        None => filepath,
    }
}
