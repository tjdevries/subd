pub fn upload_image(msg: UserMessage) -> Result<()> {
    let image_url = msg
        .contents
        .iter()
        .split(" ")
        .skip(1)
        .take(1)
        .collect::<Vec<String>>();

    let output_path = format!("{}.png" msg.user_name);

    let output = Command::new("sh")
        .arg(format!("curl -s {} | rembg i > output.png", image_url))
        .output()
        .expect("failed to executge process");

    let output = Command::new("sh")
        .arg(format!(
            "convert output.png -resize 256x256^ {}",
            output_path
        ))
        .output()
        .expect("failed to edcute proces");

    Ok(())
}

mod tests {
    use subd_types::UserMessage;

    fn upload_image() {
        super::handlers::upload_image(UserMessage {
            user_id: 1,
            user_name: "rockerBOO".to_string(),
            roles: subd_types::Role::TwitchMod,
            platform: subd_types::UserPlatform::Twitch,
            contents: "hello test chat".to_string(),
        })
    }
}
