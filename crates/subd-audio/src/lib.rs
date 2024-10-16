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
pub async fn play_sound(sink: &Sink, soundname: &str) -> Result<()> {
    let file = match File::open(format!("./MP3s/{}.mp3", soundname)) {
        Ok(file) => BufReader::new(file),
        Err(e) => {
            println!("Error opening file: {}", e);
            return Err(anyhow::anyhow!("Failed to open sound file"));
        }
    };
    let sleep_time = time::Duration::from_millis(1000);
    thread::sleep(sleep_time);
    // To tell me a screen shot is coming
    sink.set_volume(0.3);
    match Decoder::new(BufReader::new(file)) {
        Ok(source) => sink.append(source),
        Err(e) => {
            println!("Error decoding file: {}", e);
            return Err(anyhow::anyhow!("Failed to decode sound file"));
        }
    }
    sink.sleep_until_end();
    Ok(())
}

pub fn get_output_stream_2(
    device_name: &str,
) -> Result<(OutputStream, OutputStreamHandle)> {
    let host = cpal::default_host();
    let devices = host
        .output_devices()
        .map_err(|e| anyhow::anyhow!("Failed to get output devices: {}", e))?;

    for device in devices {
        // Convert cpal::Device to rodio::Device
        let dev = device;
        let dev_name = match dev.name() {
            Ok(name) => name,
            Err(e) => {
                println!("Error getting device name: {}", e);
                continue;
            }
        };
        if dev_name == device_name {
            println!("Device found: {}", dev_name);
            // Return the result directly without unwrapping
            return Ok(OutputStream::try_from_device(&dev)?);
        }
    }

    Err(anyhow::anyhow!("Device not found"))
}
pub fn get_output_stream(
    device_name: &str,
) -> Result<(OutputStream, OutputStreamHandle)> {
    let host = cpal::default_host();
    let devices = host.output_devices()?;

    for device in devices {
        if let Ok(name) = device.name() {
            println!("Device found: {}", name);
            if name == device_name {
                return OutputStream::try_from_device(&device).map_err(|e| {
                    anyhow::anyhow!("Failed to create output stream: {}", e)
                });
            }
        }
    }

    Err(anyhow::anyhow!("Device not found"))
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
    voice: &str,
    local_audio_path: &str,
) -> Result<String> {
    let mut local_audio_path = normalize_tts_file(local_audio_path)?;
    if req.reverb {
        local_audio_path = add_reverb(&local_audio_path)?;
    }

    if let Some(stretch) = &req.stretch {
        local_audio_path = stretch_audio(&local_audio_path, stretch)?;
    }

    if let Some(pitch) = &req.pitch {
        local_audio_path = change_pitch(&local_audio_path, pitch)?;
    }

    if voice == "evil_pokimane" {
        local_audio_path = change_pitch(&local_audio_path, "-200")?;
        local_audio_path = add_reverb(&local_audio_path)?;
    }

    if voice == "satan" {
        local_audio_path = change_pitch(&local_audio_path, "-350")?;
        local_audio_path = add_reverb(&local_audio_path)?;
    }

    if voice == "god" {
        local_audio_path = add_reverb(&local_audio_path)?;
    }

    Ok(local_audio_path.to_string())
}

fn normalize_tts_file(local_audio_path: &str) -> Result<String> {
    let audio_dest_path = add_postfix_to_filepath(local_audio_path, "_norm");
    let ffmpeg_status = Command::new("ffmpeg")
        .args(["-i", local_audio_path, &audio_dest_path])
        .status()
        .expect("Failed to execute ffmpeg");

    if ffmpeg_status.success() {
        Ok(audio_dest_path)
    } else {
        println!("Failed to normalize audio");
        Ok(local_audio_path.to_string())
    }
}

fn stretch_audio(local_audio_path: &str, stretch: &str) -> Result<String> {
    let audio_dest_path = add_postfix_to_filepath(local_audio_path, "_stretch");
    Command::new("sox")
        .args([
            "-t",
            "wav",
            local_audio_path,
            &audio_dest_path,
            "stretch",
            stretch,
        ])
        .status()
        .expect("Failed to execute sox");
    Ok(audio_dest_path)
}

fn change_pitch(local_audio_path: &str, pitch: &str) -> Result<String> {
    let postfix = format!("{}_{}", "_pitch", pitch);
    let audio_dest_path = add_postfix_to_filepath(local_audio_path, &postfix);
    Command::new("sox")
        .args([
            "-t",
            "wav",
            local_audio_path,
            &audio_dest_path,
            "pitch",
            pitch,
        ])
        .status()
        .expect("Failed to execute sox");

    Ok(audio_dest_path)
}

fn add_reverb(local_audio_path: &str) -> Result<String> {
    let audio_dest_path = add_postfix_to_filepath(local_audio_path, "_reverb");
    Command::new("sox")
        .args([
            "-t",
            "wav",
            local_audio_path,
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
fn add_postfix_to_filepath(filepath: &str, postfix: &str) -> String {
    match filepath.rfind('.') {
        Some(index) => {
            let path = filepath[..index].to_string();
            let filename = filepath[index..].to_string();
            format!("{}{}{}", path, postfix, filename)
        }
        None => filepath.to_string(),
    }
}
