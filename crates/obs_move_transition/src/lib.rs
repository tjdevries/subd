pub mod duration;
pub mod models;
pub mod move_source;
pub mod move_value;
use anyhow::{Context, Result};
use obws::{requests::sources::SourceId, Client as OBSClient};

use crate::models::Coordinates;

pub async fn find_source(
    scene: impl Into<String> + std::fmt::Debug,
    source: impl Into<String> + std::fmt::Debug,
    filter_name: impl Into<String> + std::fmt::Debug,
    obs_client: &OBSClient,
) -> Result<()> {
    println!(
        "\nFinding Source: {:?} {:?} {:?}",
        scene, source, filter_name
    );
    let ms = move_source::MoveSourceSettings::builder()
        .relative_transform(false)
        .position(models::Coordinates::new(Some(100.0), Some(100.0)))
        .scale(models::Coordinates::new(Some(1.0), Some(1.0)))
        .rot(0.0)
        .build();
    let filter_name = filter_name.into();
    let settings = move_source::MoveSource::new(
        source,
        &filter_name,
        ms,
        duration::EasingDuration::new(300),
    );
    update_filter_and_enable(&scene.into(), &filter_name, settings, obs_client)
        .await
}

pub async fn scale_source(
    scene: impl Into<String>,
    source: impl Into<String>,
    filter_name: impl Into<String>,
    x: f32,
    y: f32,
    obs_client: &OBSClient,
) -> Result<()> {
    let duration = duration::EasingDuration::builder()
        .duration(3000)
        .easing_function(duration::EasingFunction::Bounce)
        .easing_type(duration::EasingType::EaseIn)
        .build();

    let b = move_source::MoveSourceSettings::builder()
        .relative_transform(true)
        .scale(models::Coordinates::new(Some(x), Some(y)));

    let ms = b.build();

    let filter_name = filter_name.into().clone();
    let settings =
        move_source::MoveSource::new(source, filter_name.clone(), ms, duration);

    println!(
        "{}",
        serde_json::to_string_pretty(&settings).map_err(
            |e| anyhow::anyhow!("Failed to serialize settings: {}", e)
        )?
    );

    update_filter_and_enable(&scene.into(), &filter_name, settings, obs_client)
        .await
}

pub async fn rot_source(
    scene: impl Into<String>,
    source: impl Into<String>,
    filter_name: impl Into<String>,
    z: f32,
    obs_client: &OBSClient,
) -> Result<()> {
    let duration = duration::EasingDuration::builder()
        .duration(3000)
        .easing_function(duration::EasingFunction::Bounce)
        .easing_type(duration::EasingType::EaseIn)
        .build();

    let b = move_source::MoveSourceSettings::builder()
        .relative_transform(true)
        .rot(z);

    let ms = b.build();

    let filter_name = filter_name.into().clone();
    let settings =
        move_source::MoveSource::new(source, filter_name.clone(), ms, duration);

    println!(
        "{}",
        serde_json::to_string_pretty(&settings).map_err(
            |e| anyhow::anyhow!("Failed to serialize settings: {}", e)
        )?
    );

    update_filter_and_enable(&scene.into(), &filter_name, settings, obs_client)
        .await
}

pub async fn move_source(
    scene: impl Into<String>,
    source: impl Into<String>,
    filter_name: impl Into<String>,
    relative: bool,
    x: Option<f32>,
    y: Option<f32>,
    crop: Option<move_source::CropSettings>,
    scale: Option<Coordinates>,
    duration: Option<duration::EasingDuration>,
    obs_client: &OBSClient,
) -> Result<()> {
    let d = match duration {
        Some(d) => d,
        None => duration::EasingDuration::builder()
            .duration(3000)
            .easing_function(duration::EasingFunction::Bounce)
            .easing_type(duration::EasingType::EaseIn)
            .build(),
    };

    let b = move_source::MoveSourceSettings::builder()
        .relative_transform(relative)
        .position(Coordinates::new(x, y));

    let b = if let Some(c) = crop { b.crop(c) } else { b };
    let b = if let Some(s) = scale { b.scale(s) } else { b };

    let ms = b.build();

    let filter_name = filter_name.into().clone();
    let settings =
        move_source::MoveSource::new(source, filter_name.clone(), ms, d);

    println!("Move Source Settings ===");
    println!(
        "{}",
        serde_json::to_string_pretty(&settings).map_err(
            |e| anyhow::anyhow!("Failed to serialize settings: {}", e)
        )?
    );
    println!("=== Move Source Settings");

    update_filter_and_enable(&scene.into(), &filter_name, settings, obs_client)
        .await
}

pub async fn update_and_trigger_move_value_for_source(
    obs_client: &OBSClient,
    source: &str,
    filter_name: &str,
    filter_setting_name: &str,
    filter_setting_value: f32,
) -> Result<()> {
    // TODO: Figure out what is correct for this duration
    let settings = move_value::SingleSourceSetting::new(
        source,
        filter_setting_name.to_string(),
        filter_setting_value,
    );

    dbg!(&settings);
    update_filter_and_enable(source, filter_name, settings, obs_client).await
}

pub async fn update_and_trigger_filter<
    T: serde::Serialize + std::default::Default,
>(
    obs_client: &OBSClient,
    source: &str,
    filter_name: &str,
    settings: T,
    duration: duration::EasingDuration,
) -> Result<()> {
    let move_transition_filter_name = format!("Move_{}", filter_name);

    dbg!(&duration);
    let settings =
        move_value::Settings::new(filter_name.to_string(), settings, duration);
    update_filter_and_enable(
        source,
        &move_transition_filter_name,
        settings,
        obs_client,
    )
    .await
}

pub async fn move_source_in_scene_x_and_y(
    obs_client: &OBSClient,
    scene: &str,
    source: &str,
    x: f32,
    y: f32,
    duration: duration::EasingDuration,
) -> Result<()> {
    let filter_name = format!("Move_{}", source);

    let builder = move_source::MoveSourceSettings::builder()
        .position(models::Coordinates::new(Some(x), Some(y)))
        .x(x)
        .y(y);

    println!("Builder: {:?}", builder);
    let s = builder.build();

    println!("scene: {:?} | filter_name: {:?}", scene, filter_name);
    println!("{:?}", serde_json::to_string_pretty(&s));

    let settings = move_value::Settings::new(source.to_string(), s, duration);

    update_filter_and_enable(scene, &filter_name, settings, obs_client).await
}

async fn update_filter<T: serde::Serialize>(
    source: &str,
    filter_name: &str,
    new_settings: T,
    obs_client: &OBSClient,
) -> Result<()> {
    let settings = obws::requests::filters::SetSettings {
        source: SourceId::Name(source),
        filter: filter_name,
        settings: Some(new_settings),
        overlay: Some(false),
    };
    obs_client
        .filters()
        .set_settings(settings)
        .await
        .context(format!("Error updating filter: {}", filter_name))
}

pub async fn spin_source(
    obs_client: &OBSClient,
    source: &str,
    rotation_z: f32,
    duration: duration::EasingDuration,
) -> Result<()> {
    // let three_d_settings = ThreeDTransformPerspective::builder()
    //     .rotation_z(Some(rotation_z))
    //     .build();
    // let settings = move_value::Settings::new(
    //     filter_name.clone(),
    //     three_d_settings,
    //     duration,
    // );

    let filter_name = "3D-Transform-Perspective";

    let settings =
        move_value::Add::new(filter_name, "Rotation.Z", rotation_z, duration);
    println!(
        "{}",
        serde_json::to_string_pretty(&settings).map_err(
            |e| anyhow::anyhow!("Failed to serialize settings: {}", e)
        )?
    );

    let move_transition_filter_name = format!("Move_{}", filter_name);
    update_filter_and_enable(
        source,
        &move_transition_filter_name,
        settings,
        obs_client,
    )
    .await
}

pub async fn update_filter_and_enable<T: serde::Serialize>(
    source: &str,
    filter_name: &str,
    new_settings: T,
    obs_client: &obws::Client,
) -> Result<()> {
    update_filter(source, filter_name, new_settings, obs_client)
        .await
        .context(format!(
            "Failed to update Filter: {} on Source: {}",
            filter_name, source
        ))?;

    let filter_enabled = obws::requests::filters::SetEnabled {
        source: SourceId::Name(source),
        filter: filter_name,
        enabled: true,
    };
    Ok(obs_client.filters().set_enabled(filter_enabled).await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use obs_service::obs::create_obs_client;
    use obws::requests::sources::SourceId;

    #[tokio::test]
    #[ignore]
    async fn test_spin() {
        let obs_client = create_obs_client().await.unwrap();
        let source = "alex";
        let duration = duration::EasingDuration::new(300);
        let _ = spin_source(&obs_client, source, 1080.0, duration).await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_move_source() {
        let obs_client = create_obs_client().await.unwrap();

        let scene = "Memes";
        let source = "alex";
        let filter_name = "Move_alex";
        let res = obs_client
            .filters()
            .get(SourceId::Name(scene), filter_name)
            .await
            .unwrap();
        dbg!(&res);

        let _ = move_source(
            scene,
            source,
            filter_name,
            true,
            Some(-100.0),
            Some(-100.0),
            None,
            None,
            None,
            &obs_client,
        )
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_find_source() {
        let obs_client = create_obs_client().await.unwrap();
        let scene = "Memes";
        let source = "alex";
        let filter_name = "Move_alex";
        let _ = find_source(scene, source, filter_name, &obs_client).await;
    }
}
