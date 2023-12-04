use anyhow::Result;
use obws::Client as OBSClient;
use rand::Rng;

// TODO late update this to handle multiple contracts
pub async fn updates_ab_browser(
    obs_client: &OBSClient,
    contract: String,
    lower_bound: i64,
    upper_bound: i64,
) -> Result<()> {
    let random_number = random_in_range(lower_bound, upper_bound);
    let ab_id = random_number.to_string();
    let ab_url =
        format!("https://generator.artblocks.io/{}/{}", contract, ab_id);
    // let ab_url = format!("https://generator.artblocks.io/0x99a9b7c1116f9ceeb1652de04d5969cce509b069/{}", ab_id);
    let browser_settings =
        obws::requests::custom::source_settings::BrowserSource {
            url: ab_url.as_ref(),
            ..Default::default()
        };
    let set_settings = obws::requests::inputs::SetSettings {
        settings: &browser_settings,
        input: "AB-Browser",
        overlay: Some(true),
    };
    let _ = obs_client.inputs().set_settings(set_settings).await;
    Ok(())
}

fn random_in_range(lower: i64, upper: i64) -> i64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(lower..=upper)
}
