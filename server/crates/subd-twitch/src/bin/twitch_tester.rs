use anyhow::Result;
use twitch_api2::types::UserId;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello, world");

    println!(
        "Is Mod: {}",
        subd_twitch::is_moderator(&UserId::from("57632769")).await?,
    );

    Ok(())
}
