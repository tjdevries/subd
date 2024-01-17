use crate::constants;
use crate::move_transition::duration;
use crate::move_transition::duration::EasingDuration;
use crate::move_transition::models;
use crate::move_transition::models::Coordinates;
use crate::move_transition::move_source;
use crate::move_transition::move_source::CropSettings;
use crate::move_transition::move_source::MoveSource;
use crate::move_transition::move_source::MoveSourceSettings;
use crate::move_transition::move_transition;
use crate::move_transition::move_value;
use crate::three_d_filter::perspective::ThreeDTransformPerspective;
use anyhow::{Context, Result};
use obws::Client as OBSClient;

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
    let ms = MoveSourceSettings::builder()
        .relative_transform(false)
        .position(Coordinates::new(Some(100.0), Some(100.0)))
        .scale(Coordinates::new(Some(1.0), Some(1.0)))
        .rot(0.0)
        .build();
    let filter_name = filter_name.into();
    let settings =
        MoveSource::new(source, &filter_name, ms, EasingDuration::new(300));
    move_transition::update_filter_and_enable(
        &scene.into(),
        &filter_name,
        settings,
        &obs_client,
    )
    .await
}

pub async fn move_source(
    scene: impl Into<String>,
    source: impl Into<String>,
    filter_name: impl Into<String>,
    x: Option<f32>,
    y: Option<f32>,
    obs_client: &OBSClient,
) -> Result<()> {
    let duration = EasingDuration::builder()
        .duration(3000)
        .easing_function(duration::EasingFunction::Bounce)
        .easing_type(duration::EasingType::EaseIn)
        .build();

    dbg!(&duration);

    let c = CropSettings::builder().left(580.0).build();
    let ms = MoveSourceSettings::builder()
        .relative_transform(true)
        .position(Coordinates::new(x, y))
        .crop(c)
        .build();
    let filter_name = filter_name.into().clone();
    let settings = MoveSource::new(source, filter_name.clone(), ms, duration);

    println!("{}", serde_json::to_string_pretty(&settings).unwrap());

    move_transition::update_filter_and_enable(
        &scene.into(),
        &filter_name,
        settings,
        &obs_client,
    )
    .await
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

pub async fn spin_source(
    obs_client: &OBSClient,
    source: &str,
    rotation_z: f32,
    duration: duration::EasingDuration,
) -> Result<()> {
    let filter_name =
        constants::THREE_D_TRANSITION_PERSPECTIVE_FILTER_NAME.to_string();
    let three_d_settings = ThreeDTransformPerspective::builder()
        .rotation_z(Some(rotation_z))
        .build();
    let move_transition_filter_name = format!("Move_{}", source);
    let settings = move_value::Settings::new(
        filter_name.clone(),
        three_d_settings,
        duration,
    );
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

pub async fn update_filter_and_enable<T: serde::Serialize>(
    source: &str,
    filter_name: &str,
    new_settings: T,
    obs_client: &obws::Client,
) -> Result<()> {
    update_filter(source, filter_name, new_settings, &obs_client)
        .await
        .context(format!(
            "Failed to update Filter: {} on Source: {}",
            filter_name, source
        ))?;

    let filter_enabled = obws::requests::filters::SetEnabled {
        source,
        filter: &filter_name,
        enabled: true,
    };
    Ok(obs_client.filters().set_enabled(filter_enabled).await?)
}

async fn update_filter<T: serde::Serialize>(
    source: &str,
    filter_name: &str,
    new_settings: T,
    obs_client: &OBSClient,
) -> Result<()> {
    let settings = obws::requests::filters::SetSettings {
        source,
        filter: filter_name,
        settings: Some(new_settings),
        overlay: Some(false),
    };
    Ok(obs_client
        .filters()
        .set_settings(settings)
        .await
        .context(format!("Error updating filter: {}", filter_name))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obs::obs::create_obs_client;

    #[tokio::test]
    async fn test_move_source() {
        let obs_client = create_obs_client().await.unwrap();

        let scene = "Memes";
        let source = "alex";
        let filter_name = "Move_alex";
        let res = obs_client.filters().get(scene, filter_name).await.unwrap();
        dbg!(&res);

        let _ = move_transition::move_source(
            scene,
            source,
            filter_name,
            Some(-100.0),
            Some(-100.0),
            &obs_client,
        )
        .await;
    }

    #[tokio::test]
    async fn test_find_source() {
        let obs_client = create_obs_client().await.unwrap();
        let scene = "Memes";
        let source = "alex";
        let filter_name = "Move_alex";
        let _ = move_transition::find_source(
            scene,
            source,
            filter_name,
            &obs_client,
        )
        .await;
    }
}
