use crate::move_transition;
use crate::move_transition_bootstrap;
use crate::obs;
use crate::sdf_effects;
use crate::stream_fx;
use anyhow::Result;
use obws;
use obws::Client as OBSClient;

pub async fn create_outline_filter(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let stream_fx_filter_name = obs::MOVE_OUTLINE_FILTER_NAME.to_string();

    // We look up Begin's Outline Settings
    let filter_details = match obs_client
        .filters()
        .get(obs::DEFAULT_SOURCE, obs::SDF_EFFECTS_FILTER_NAME)
        .await
    {
        Ok(val) => val,
        Err(_err) => {
            return Ok(());
        }
    };

    let new_settings =
        serde_json::from_value::<sdf_effects::SDFEffectsSettings>(
            filter_details.settings,
        )
        .unwrap();

    let new_filter = obws::requests::filters::Create {
        source,
        filter: obs::SDF_EFFECTS_FILTER_NAME,
        kind: &obs::SDF_EFFECTS_INTERNAL_FILTER_NAME.to_string(),
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    // I think this is fucking shit up
    // Create Move-Value for 3D Transform Filter
    let new_settings = move_transition::MoveSingleValueSetting {
        move_value_type: Some(1),
        filter: String::from(obs::SDF_EFFECTS_FILTER_NAME),
        duration: Some(7000),
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: &stream_fx_filter_name,
        kind: &obs::MOVE_VALUE_INTERNAL_FILTER_NAME.to_string(),
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    Ok(())
}

pub async fn create_blur_filters(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let stream_fx_filter_name = obs::MOVE_BLUR_FILTER_NAME.to_string();

    let stream_fx_settings = stream_fx::StreamFXSettings {
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: obs::BLUR_FILTER_NAME,
        kind: &obs::BLUR_INTERNAL_FILTER_NAME.to_string(),
        settings: Some(stream_fx_settings),
    };
    obs_client.filters().create(new_filter).await?;

    // Create Move-Value for 3D Transform Filter
    let new_settings = move_transition::MoveSingleValueSetting {
        move_value_type: Some(0),
        filter: String::from(obs::BLUR_FILTER_NAME),
        duration: Some(7000),
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: &stream_fx_filter_name,
        kind: &obs::MOVE_VALUE_INTERNAL_FILTER_NAME.to_string(),
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    Ok(())
}

pub async fn create_scroll_filters(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let stream_fx_filter_name = obs::MOVE_SCROLL_FILTER_NAME.to_string();

    let stream_fx_settings = stream_fx::StreamFXSettings {
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: &obs::SCROLL_FILTER_NAME.to_string(),
        kind: &obs::SCROLL_INTERNAL_FILTER_NAME.to_string(),
        settings: Some(stream_fx_settings),
    };
    obs_client.filters().create(new_filter).await?;

    // Create Move-Value for 3D Transform Filter
    let new_settings = move_transition::MoveSingleValueSetting {
        move_value_type: Some(0),
        filter: String::from(&obs::SCROLL_FILTER_NAME.to_string()),
        duration: Some(7000),
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: &stream_fx_filter_name,
        kind: &obs::MOVE_VALUE_INTERNAL_FILTER_NAME.to_string(),
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    Ok(())
}

pub async fn create_split_3d_transform_filters(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let camera_types = vec!["Orthographic", "Perspective", "CornerPin"];

    for (i, camera_type) in camera_types.iter().enumerate() {
        let filter_name = format!("3D_{}", camera_type);
        let stream_fx_settings = stream_fx::StreamFXSettings {
            camera_mode: Some(i as i32),
            ..Default::default()
        };
        let new_filter = obws::requests::filters::Create {
            source,
            filter: &filter_name,
            kind: obs::STREAM_FX_INTERNAL_FILTER_NAME,
            settings: Some(stream_fx_settings),
        };
        obs_client.filters().create(new_filter).await?;

        let stream_fx_filter_name = format!("Move_3D_{}", camera_type);

        let new_settings = move_transition::MoveSingleValueSetting {
            move_value_type: Some(0),
            filter: String::from(filter_name),
            duration: Some(7000),
            ..Default::default()
        };
        let new_filter = obws::requests::filters::Create {
            source,
            filter: &stream_fx_filter_name,
            kind: obs::MOVE_VALUE_INTERNAL_FILTER_NAME,
            settings: Some(new_settings),
        };
        obs_client.filters().create(new_filter).await?;

        // Create Default Move-Value for 3D Transform Filter
        let stream_fx_filter_name = format!("Move_3D_{}", camera_type);

        let filter_name = format!("3D_{}", camera_type);
        let new_settings = move_transition::MoveSingleValueSetting {
            move_value_type: Some(0),
            filter: String::from(filter_name),
            duration: Some(3000),
            ..Default::default()
        };
        let new_filter = obws::requests::filters::Create {
            source,
            filter: &stream_fx_filter_name,
            kind: obs::MOVE_VALUE_INTERNAL_FILTER_NAME,
            settings: Some(new_settings),
        };
        obs_client.filters().create(new_filter).await?;
    }

    Ok(())
}
pub async fn create_3d_transform_filters(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let stream_fx_filter_name = obs::MOVE_STREAM_FX_FILTER_NAME.to_string();

    let stream_fx_settings = stream_fx::StreamFXSettings {
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: obs::THE_3D_TRANSFORM_FILTER_NAME,
        kind: &obs::STREAM_FX_INTERNAL_FILTER_NAME.to_string(),
        settings: Some(stream_fx_settings),
    };
    obs_client.filters().create(new_filter).await?;

    // Create Move-Value for 3D Transform Filter
    let new_settings = move_transition::MoveSingleValueSetting {
        move_value_type: Some(0),
        filter: String::from(obs::THE_3D_TRANSFORM_FILTER_NAME),
        duration: Some(7000),
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: &stream_fx_filter_name,
        kind: &obs::MOVE_VALUE_INTERNAL_FILTER_NAME.to_string(),
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    Ok(())
}

pub async fn create_filters_for_source(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    println!("Creating Filters for Source: {}", source);

    let filters = match obs_client.filters().list(source).await {
        Ok(val) => val,
        Err(_) => return Ok(()),
    };

    if source == obs::DEFAULT_SOURCE {
        return Ok(());
    }

    for filter in filters {
        obs_client
            .filters()
            .remove(&source, &filter.name)
            .await
            .expect("Error Deleting Filter");
    }

    let filter_name = format!("Move_Source_Home_{}", source);
    move_transition_bootstrap::create_move_source_filters(
        obs::DEFAULT_SCENE,
        &source,
        &filter_name,
        &obs_client,
    )
    .await?;

    // We should seperate to it's own !chat command
    // create_split_3d_transform_filters(source, &obs_client).await?;
    create_3d_transform_filters(source, &obs_client).await?;
    create_scroll_filters(source, &obs_client).await?;
    create_blur_filters(source, &obs_client).await?;
    create_outline_filter(source, &obs_client).await?;

    let new_settings = move_transition::MoveSingleValueSetting {
        move_value_type: Some(1),
        filter: String::from(obs::THE_3D_TRANSFORM_FILTER_NAME),
        duration: Some(7000),
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: obs::DEFAULT_STREAM_FX_FILTER_NAME,
        kind: &obs::MOVE_VALUE_INTERNAL_FILTER_NAME.to_string(),
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    let new_settings = move_transition::MoveSingleValueSetting {
        move_value_type: Some(1),
        filter: String::from(&obs::SCROLL_FILTER_NAME.to_string()),
        duration: Some(7000),
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: obs::DEFAULT_SCROLL_FILTER_NAME,
        kind: &obs::MOVE_VALUE_INTERNAL_FILTER_NAME.to_string(),
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    // This is For Blur
    let new_settings = move_transition::MoveSingleValueSetting {
        move_value_type: Some(1),
        filter: String::from(obs::BLUR_FILTER_NAME),
        filter_blur_size: Some(1.0),
        setting_float: 0.0,
        duration: Some(7000),
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: obs::DEFAULT_BLUR_FILTER_NAME,
        kind: &obs::MOVE_VALUE_INTERNAL_FILTER_NAME.to_string(),
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    // This is for SDF Effects
    let new_settings = move_transition::MoveSingleValueSetting {
        move_value_type: Some(1),
        filter: String::from(obs::SDF_EFFECTS_FILTER_NAME),
        duration: Some(7000),
        glow_inner: Some(false),
        glow_outer: Some(false),
        shadow_outer: Some(false),
        shadow_inner: Some(false),
        outline: Some(false),
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: obs::DEFAULT_SDF_EFFECTS_FILTER_NAME,
        kind: &obs::MOVE_VALUE_INTERNAL_FILTER_NAME.to_string(),
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    let filter_name = format!("Move_Source_{}", source);

    move_transition_bootstrap::create_move_source_filters(
        obs::DEFAULT_SCENE,
        &source,
        &filter_name,
        &obs_client,
    )
    .await?;

    Ok(())
}
