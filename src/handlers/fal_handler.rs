use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use fal_ai;
use obws::Client as OBSClient;
use rodio::*;
use subd_types::{Event, UserMessage};
use tokio::sync::broadcast;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};
use twitch_stream_state;

pub struct FalHandler {
    pub obs_client: OBSClient,
    pub pool: sqlx::PgPool,
    pub sink: Sink,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

#[async_trait]
impl EventHandler for FalHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::UserMessage(msg) => msg,
                _ => continue,
            };

            let splitmsg = msg
                .contents
                .split(' ')
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            match handle_fal_commands(
                &tx,
                &self.obs_client,
                &self.twitch_client,
                &self.pool,
                &self.sink,
                splitmsg,
                msg,
            )
            .await
            {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Error: {err}");
                    continue;
                }
            }
        }
    }
}

pub async fn handle_fal_commands(
    _tx: &broadcast::Sender<Event>,
    _obs_client: &OBSClient,
    _twitch_client: &TwitchIRCClient<
        SecureTCPTransport,
        StaticLoginCredentials,
    >,
    pool: &sqlx::PgPool,
    _sink: &Sink,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<()> {
    let is_mod = msg.roles.is_twitch_mod();
    let _not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";
    let command = splitmsg[0].as_str();

    match command {
        // This sets the theme, that is passed to the image generation prompt
        "!theme" => {
            if _not_beginbot && !is_mod {
                return Ok(());
            }
            let theme = &splitmsg
                .iter()
                .skip(1)
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>()
                .join(" ");
            twitch_stream_state::set_ai_background_theme(pool, theme).await?;
        }

        "!talk" => {
            // Not sure why this is hardcoded here
            println!("\n\nTALK TIME!");
            let image_file_path = "teej_2.jpg";
            let _ =
                fal_ai::create_video_from_image(image_file_path, None).await;
        }

        _ => {
            // TODO: a way to enable or disable Fal on every chat-message
            let word_count = msg.contents.split_whitespace().count();
            if !command.starts_with('!')
                && !command.starts_with('@')
                && word_count > 1
            {
                let prompt = msg.contents;
                let theme =
                    twitch_stream_state::get_ai_background_theme(pool).await?;
                let final_prompt = format!("{} {}", theme, prompt);
                println!("Creating image for prompt: {}", final_prompt);

                // TODO: Can we use another model here???
                // fal_ai::create_turbo_image(&final_prompt).await?;

                // This seems wrong
                let _ =
                    fal_ai::create_and_save_image(&final_prompt, None, None)
                        .await;
            }
        }
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fal_hit() {
        let prompt = "Dog wearing a Top Hat";
        // let prompt = "Realistic Bowser in a dark Fantasy background";
        //let prompt = "Waluigi is portrayed as a mischievous villain and an exaggerated mirror version of Luigi. He bears a large pink nose and crooked moustache. His stature is tall and his frame thin and lanky. He is the rival of Luigi and is the same age. His Super Smash Bros. Melee trophy describes him as hardworking by stating that he spends his time training and perfecting his skills in order to antagonise the Mario brothers. Although he has a scrawny physique, Waluigi is technically skilled and athletic, but sometimes cheats in-game. His design comprises black overalls, a purple long-sleeved shirt, a purple hat with a yellow \"Γ\" symbol";

        // let prompt = "Anime Girl";
        // let prompt = "newest, extremely aesthetic, best quality, 1girl, solo, pink hair, blue eyes, long hair, looking at viewer, smile, black background, holding a sign, the text on the sign says 'Hello'";

        // This works and is hilarious
        //let model = "fal-ai/aura-flow";

        // anime model
        // real bad for some reason if not anime???
        // soooo bad
        // let model = "fal-ai/stable-cascade/sote-diffusion";

        // This is good, real good
        // not always realistic
        // let model = "fal-ai/flux-realism";

        // Pretty Cool
        //let model = "fal-ai/realistic-vision";

        // Pretty Cool
        //let model = "fal-ai/flux-pro/v1.1";

        let model = "fal-ai/kolors";

        // let model = "fal-ai/fast-sdxl";
        // let model = "fal-ai/stable-cascade";
        // let model = "fal-ai/flux/dev";

        let _ = fal_ai::create_and_save_image_for_model(prompt, model).await;
        // Ok now
    }
}
