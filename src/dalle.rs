use anyhow::Result;
use reqwest;
use std::env;
use std::fs::File;
use std::io::Write;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ImageResponse {
    created: i64,  // Assuming 'created' is a Unix timestamp
    data: Vec<ImageData>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ImageData {
    url: String,
}


pub async fn dalle_time(contents: String, username: String) -> Result<(), reqwest::Error> {
    let api_key = env::var("OPENAI_API_KEY").unwrap();

// TODO: This is for saving to the file
    // which we aren't doing yet
    let _truncated_prompt = contents.chars().take(80).collect::<String>();
    let client = reqwest::Client::new();

    // let size = "1792x1024";
    // let other_size = "1024x1792";
    
    // Not sure 
    // TODO: Update these
    let response = client
        .post("https://api.openai.com/v1/images/generations")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "prompt": contents,
            "n": 4,
            // "size": size,
            // "size": "1080x1080",
            // "size": "1792x1024",
            "size": "1024x1024",
        }))
        .send()
        .await?;
    
    let text = response.text().await?;

    let image_response: Result<ImageResponse, _> = serde_json::from_str(&text);

    match image_response {
        Ok(response) => {
            for (index, image_data) in response.data.iter().enumerate() {

                println!("Image URL: {} | ", image_data.url.clone());
                let image_data = reqwest::get(image_data.url.clone()).await?.bytes().await?.to_vec();
                
                // Features to Save:
                // username
                // resolution
                // prompt
                // timestamp
                // let username = "default";
                
                // Is there a subsecond
                let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
                let unique_identifier = format!("{}_{}_{}", timestamp, index, username);
                println!("Contents: {}", contents);
                let archive_file = format!("./archive/{}.png", unique_identifier);
                let mut file = File::create(archive_file).unwrap();
                file.write_all(&image_data).unwrap();
                
                let mut csv_file = File::create("output.csv").unwrap();
                writeln!(csv_file, "{},{}", unique_identifier, contents).unwrap();

                
                let filename = format!("./tmp/dalle-{}.png", index+1);
                let mut file = File::create(filename).unwrap();
                file.write_all(&image_data).unwrap();
            }
        },
        Err(e) => {
            eprintln!("Error deserializing response: {}", e);
        }
    }
    
    Ok(())
} 
