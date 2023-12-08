use anyhow::Result;
use chrono::Utc;
use reqwest;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug)]
struct ImageResponse {
    created: i64, // Assuming 'created' is a Unix timestamp
    data: Vec<ImageData>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ImageData {
    url: String,
}

pub async fn dalle_time(
    contents: String,
    username: String,
    amount: i32,
) -> Result<(), reqwest::Error> {
    let api_key = env::var("OPENAI_API_KEY").unwrap();

    // TODO: This was supposed to be for saving to the file
    // which we aren't doing yet
    let _truncated_prompt = contents.chars().take(80).collect::<String>();
    let client = reqwest::Client::new();

    let size = "1024x1024";
    // TODO: read from the database
    let model = "dall-e-3";

    let response = client
        .post("https://api.openai.com/v1/images/generations")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "prompt": contents,
            "n": amount,
            "model": model,
            "size": size,
        }))
        .send()
        .await?;

    let dalle_response_text = response.text().await?;

    let mut csv_file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true) // This will create the file if it doesn't exist
        .open("output.csv")
        .unwrap();

    let image_response: Result<ImageResponse, _> =
        serde_json::from_str(&dalle_response_text);
    match image_response {
        Ok(response) => {
            for (index, image_data) in response.data.iter().enumerate() {
                println!("Image URL: {} | ", image_data.url.clone());
                let image_data = reqwest::get(image_data.url.clone())
                    .await?
                    .bytes()
                    .await?
                    .to_vec();

                // "id": 9612607,
                // request for AI_image_filename
                let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
                let unique_identifier =
                    format!("{}_{}_{}", timestamp, index, username);
                let archive_file =
                    format!("./archive/{}.png", unique_identifier);

                let mut file = File::create(archive_file).unwrap();
                file.write_all(&image_data).unwrap();

                writeln!(csv_file, "{},{}", unique_identifier, contents)
                    .unwrap();

                let filename = format!("./tmp/dalle-{}.png", index + 1);
                let mut file = File::create(filename).unwrap();
                file.write_all(&image_data).unwrap();
            }
        }
        Err(e) => {
            eprintln!("Error deserializing response: {}", e);
        }
    }

    Ok(())
}

// let prompt = "A majestic dog sitting on a throne, wearing a crown.";
// let image_name = "majestic_dog.png";
//
// if let Err(e) = generate_image(prompt, image_name).await {
//     eprintln!("Error: {}", e);
// }

pub async fn generate_image(
    prompt: String,
    username: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = env::var("STABLE_DIFFUSION_URL")
        .map_err(|_| "STABLE_DIFFUSION_URL environment variable not set")?;

    let client = Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&json!({"prompt": prompt}))
        .send()
        .await?;

    let image_data = response.bytes().await?;

    // We aren't currently able to generate more than image
    let index = 1;
    // TODO: move this to a function
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let unique_identifier = format!("{}_{}_{}", timestamp, index, username);
    let archive_file = format!("./archive/{}.png", unique_identifier);

    let mut file = File::create(archive_file).unwrap();
    file.write_all(&image_data).unwrap();

    let filename = format!("./tmp/dalle-{}.png", index);
    let mut file = File::create(filename).unwrap();
    file.write_all(&image_data).unwrap();
    Ok(())
}
