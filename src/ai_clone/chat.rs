use anyhow::anyhow;
use anyhow::Result;

// TODO move this to somewhere else / pull in from config
const SCREENSHOT_SOURCE: &str = "begin-base";
const SCENE: &str = "AIAssets";
const SOURCE: &str = "bogan";

// Argument parsing should be closer to reading in the chat
pub fn parse_args(
    splitmsg: &Vec<String>,
    screenshot_source: String,
) -> Result<(String, Option<f32>)> {
    // TODO: Extract this to a constant or config

    let strength = splitmsg.get(1).ok_or(anyhow!("Nothing to modify!"))?;
    let parsed_strength = strength.parse::<f32>();

    let (prompt_offset, strength) = match parsed_strength {
        Ok(f) => (2, Some(f)),
        Err(_) => (1, None),
    };

    let prompt = splitmsg
        .iter()
        .skip(prompt_offset)
        .map(AsRef::as_ref)
        .collect::<Vec<&str>>()
        .join(" ");

    let prompt = if screenshot_source == "begin-base" {
        format!("{}. on a bright chroma key green screen background", prompt)
    } else {
        prompt
    };

    Ok((prompt, strength))
}
