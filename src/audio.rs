//use anyhow::Result;
//use rodio::cpal::traits::{DeviceTrait, HostTrait};
//use rodio::OutputStream;
//use rodio::*;
//use std::fs::File;
//use std::io::BufReader;
//use std::thread;
//use std::time;
//
//// This also has a pause in it,
//// we might want to take that in as a variable
//pub async fn play_sound(sink: &Sink, soundname: String) -> Result<()> {
//    let file = BufReader::new(
//        File::open(format!("./MP3s/{}.mp3", soundname)).unwrap(),
//    );
//    let sleep_time = time::Duration::from_millis(1000);
//    thread::sleep(sleep_time);
//    // To tell me a screen shot is coming
//    sink.set_volume(0.3);
//    sink.append(Decoder::new(BufReader::new(file)).unwrap());
//    sink.sleep_until_end();
//    Ok(())
//}
//
//pub fn get_output_stream(
//    device_name: &str,
//) -> Result<(OutputStream, OutputStreamHandle)> {
//    let host = cpal::default_host();
//    let devices = host.output_devices().unwrap();
//
//    for device in devices {
//        println!("Device found: {}", device.name().unwrap());
//    }
//
//    let devices = host.output_devices().unwrap();
//    for device in devices {
//        let dev: rodio::Device = device.into();
//        let dev_name: String = dev.name().unwrap();
//        if dev_name == device_name {
//            println!("Device found: {}", dev_name);
//            return Ok(OutputStream::try_from_device(&dev).unwrap());
//        }
//    }
//
//    return Err(anyhow::anyhow!("Device not found"));
//}
//
//// ============== //
//// Audio Settings //
//// ============== //
//
//pub async fn set_audio_status(
//    _obs_conn: &obws::Client,
//    _name: &str,
//    _status: bool,
//) -> Result<()> {
//    // obs_conn.sources().(name, !status).await?;
//    Ok(())
//}
