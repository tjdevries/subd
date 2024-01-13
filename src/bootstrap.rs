use crate::move_transition;
use crate::move_transition_bootstrap;
use crate::obs;
use crate::sdf_effects;
use crate::stream_fx;
use anyhow::Result;
use obws;
use obws::Client as OBSClient;

// I don't know why it's 1 sometimes and it's 2 others
// RENAME THESE ONCE you figure what they are for
static MOVE_VALUE_TYPE_SINGLE: u32 = 1;
static MOVE_VALUE_TYPE_ZERO: u32 = 0;

static DEFAULT_DURATION: u32 = 7000;

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
        move_value_type: Some(MOVE_VALUE_TYPE_SINGLE),
        filter: String::from(obs::SDF_EFFECTS_FILTER_NAME),
        duration: Some(DEFAULT_DURATION),
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
        move_value_type: Some(MOVE_VALUE_TYPE_ZERO),
        filter: String::from(obs::BLUR_FILTER_NAME),
        duration: Some(DEFAULT_DURATION),
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
        move_value_type: Some(MOVE_VALUE_TYPE_ZERO),
        filter: String::from(&obs::SCROLL_FILTER_NAME.to_string()),
        duration: Some(DEFAULT_DURATION),
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
    // let camera_types = vec![ "Perspective", "CornerPin"];

    for (i, camera_type) in camera_types.iter().enumerate() {
        let filter_name = format!("3D-Transform-{}", camera_type);
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

        let stream_fx_filter_name = format!("Move_{}", filter_name.clone());

        let new_settings = move_transition::MoveSingleValueSetting {
            move_value_type: Some(MOVE_VALUE_TYPE_ZERO),
            filter: String::from(filter_name.clone()),
            duration: Some(DEFAULT_DURATION),
            ..Default::default()
        };
        let new_filter = obws::requests::filters::Create {
            source,
            filter: &stream_fx_filter_name,
            kind: obs::MOVE_VALUE_INTERNAL_FILTER_NAME,
            settings: Some(new_settings),
        };
        let _ = obs_client.filters().create(new_filter).await;

        // Create Default Move-Value for 3D Transform Filter
        let stream_fx_filter_name = format!("Default_{}", filter_name.clone());

        let filter_name = format!("3D_{}", camera_type);
        let new_settings = move_transition::MoveSingleValueSetting {
            move_value_type: Some(MOVE_VALUE_TYPE_ZERO),
            filter: String::from(filter_name.clone()),
            duration: Some(3000),
            ..Default::default()
        };
        let new_filter = obws::requests::filters::Create {
            source,
            filter: &stream_fx_filter_name,
            kind: obs::MOVE_VALUE_INTERNAL_FILTER_NAME,
            settings: Some(new_settings),
        };
        let _ = obs_client.filters().create(new_filter).await;
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
        move_value_type: Some(MOVE_VALUE_TYPE_ZERO),
        filter: String::from(obs::THE_3D_TRANSFORM_FILTER_NAME),
        duration: Some(DEFAULT_DURATION),
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

pub async fn remove_all_filters(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
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
    Ok(())
}
