use anyhow::Result;

pub async fn breakup_text(contents: String) -> Result<String> {
    let mut seal_text = contents.clone();
    let spaces: Vec<_> = contents.match_indices(" ").collect();
    let line_length_modifier = 20;
    let mut line_length_limit = 20;
    for val in spaces.iter() {
        if val.0 > line_length_limit {
            seal_text.replace_range(val.0..=val.0, "\n");
            line_length_limit = line_length_limit + line_length_modifier;
        }
    }
    Ok(seal_text)
}
