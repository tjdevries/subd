use anyhow::Result;
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::OutputStream;
use rodio::*;

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
