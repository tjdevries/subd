use crate::move_transition;
use crate::move_transition_bootstrap;
use crate::obs;
use crate::obs_filters;
use crate::stream_fx;
use anyhow::Result;
use move_transition::MoveMultipleValuesSetting;
use obws::responses::filters::SourceFilter;
use obws::Client as OBSClient;
use sqlx::types::BigDecimal;

pub async fn generic_update_trigger_move_values<
    T: serde::Serialize
        + std::default::Default
        + obs_filters::three_d_transform::FilterName,
>(
    obs_client: &OBSClient,
    source: &str,

    // These are an struct
    duration: u64,
    easing_function_index: Option<i32>,
    easing_type_index: Option<i32>,
    settings: T,
) -> Result<()> {
    let filter_name = settings.filter_name();
    let new_settings = obs_filters::three_d_transform::MovePluginSettings {
        filter: filter_name.clone(),
        duration: Some(duration as u32),
        easing_function: easing_function_index,
        easing_type: easing_type_index,
        settings,
        ..Default::default()
    };

    let move_transition_filter_name = format!("Move_{}", filter_name);
    let _ = update_and_trigger_move_values_filter(
        source,
        &move_transition_filter_name,
        new_settings,
        &obs_client,
    )
    .await?;
    Ok(())
}

pub async fn top_right(
    scene: &str,
    scene_item: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let base_settings =
        move_transition::fetch_source_settings(scene, &scene_item, &obs_client)
            .await?;

    let new_settings =
        move_transition::custom_filter_settings(base_settings, 1662.0, 13.0);
    let filter_name = format!("Move_{}", scene_item);

    move_transition::move_with_move_source(
        scene,
        &filter_name,
        new_settings,
        &obs_client,
    )
    .await
}

// This doesn't really "find"
pub async fn find_or_create_filter(
    scene: &str,
    source: &str,
    filter_name: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let filters = match obs_client.filters().list(scene).await {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Error listing filters: {}", e);
            return Ok(());
        }
    };

    let mut filter_exists = false;
    for filter in filters {
        if filter.name == filter_name {
            filter_exists = true
        }
    }

    if !filter_exists {
        move_transition_bootstrap::create_move_source_filters(
            &scene,
            &source,
            &filter_name,
            &obs_client,
        )
        .await?;
    }

    Ok(())
}

pub async fn new_spin(
    source: &str,
    filter_name: &str,
    mut new_settings: move_transition::MoveMultipleValuesSetting,
    obs_client: &OBSClient,
) -> Result<()> {
    new_settings.move_value_type = 1;
    new_settings.value_type = 1;
    dbg!(&new_settings);
    let settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: filter_name,
        settings: new_settings,
        overlay: Some(true),
    };
    let _ = obs_client.filters().set_settings(settings).await;

    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: filter_name,
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;
    Ok(())
}

pub async fn update_and_trigger_move_values_filter<T: serde::Serialize>(
    source: &str,
    filter_name: &str,
    new_settings: obs_filters::three_d_transform::MovePluginSettings<T>,
    obs_client: &OBSClient,
) -> Result<()> {
    let settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: filter_name,
        settings: new_settings,
        overlay: Some(true),
    };
    let _ = obs_client.filters().set_settings(settings).await;

    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: filter_name,
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;
    Ok(())
}

pub async fn spin_source2(
    obs_client: &OBSClient,
    source: &str,
    rotation_z: f32,
    duration: u64,
    easing_function_index: Option<i32>,
    easing_type_index: Option<i32>,
) -> Result<()> {
    let filter_name = "3D-Transform-Perspective".to_string();

    let settings = obs_filters::three_d_transform::ThreeDTransformPerspective {
        rotation_z: Some(rotation_z),
        camera_mode: (),
        ..Default::default()
    };
    let new_settings = obs_filters::three_d_transform::MovePluginSettings {
        filter: filter_name.clone(),
        duration: Some(duration as u32),
        easing_function: easing_function_index,
        easing_type: easing_type_index,
        settings,
        ..Default::default()
    };

    dbg!(&new_settings);

    let move_transition_filter_name = format!("Move_{}", filter_name);

    let _ = update_and_trigger_move_values_filter(
        source,
        &move_transition_filter_name,
        new_settings,
        &obs_client,
    )
    .await?;
    Ok(())
}

pub async fn spin_source(
    source: &str,
    rotation_z: f32,
    duration: u64,
    easing_function_index: i32,
    easing_type_index: i32,
    obs_client: &OBSClient,
) -> Result<()> {
    let filter_name = "3D-Transform-Perspective";

    let new_settings = move_transition::MoveMultipleValuesSetting {
        rotation_z: Some(rotation_z),
        easing_function: Some(easing_function_index),
        easing_type: Some(easing_type_index),
        duration: Some(duration as u32),
        ..Default::default()
    };

    dbg!(&new_settings);

    let three_d_transform_filter_name = filter_name;
    let move_transition_filter_name =
        format!("Move_{}", three_d_transform_filter_name);

    let _ = move_transition::update_and_trigger_move_values_filter(
        source,
        &move_transition_filter_name,
        new_settings,
        &obs_client,
    )
    .await?;
    Ok(())
}

// Scale Source taking in BigD
pub async fn scale_source2(
    scene: &str,
    source: &str,
    x: f32,
    y: f32,
    obs_client: &OBSClient,
) -> Result<()> {
    let filter_name = format!("Move_{}", source);
    let new_settings = move_transition::MoveMultipleValuesSetting {
        scale_x: Some(x),
        scale_y: Some(y),
        duration: Some(300),
        ..Default::default()
    };

    // Let's save the obs_source position
    // which requires update_or_create
    // let mut settings =
    //     move_transition::fetch_source_settings(scene, &source, &obs_client)
    //         .await?;

    // settings.x = x;
    // settings.y = y;

    move_transition::move_with_move_source2(
        scene,
        &filter_name,
        new_settings,
        &obs_client,
    )
    .await
}

pub async fn scale_source(
    scene: &str,
    source: &str,
    x: f32,
    y: f32,
    duration: u64,
    easing_function_index: i32,
    easing_type_index: i32,
    obs_client: &OBSClient,
) -> Result<()> {
    let filter_name = format!("Move_{}", source);
    let mut settings =
        move_transition::fetch_source_settings(scene, &source, &obs_client)
            .await?;

    settings.scale = Some(move_transition::Coordinates {
        x: Some(x),
        y: Some(y),
    });
    settings.bounds = Some(move_transition::Coordinates {
        x: Some(x),
        y: Some(y),
    });

    settings.duration = Some(duration);
    settings.easing_type = Some(easing_type_index);
    settings.easing_function = Some(easing_function_index);

    move_transition::move_with_move_source(
        scene,
        &filter_name,
        settings,
        &obs_client,
    )
    .await
}

pub async fn move_source_with_custom_settings(
    scene: &str,
    source: &str,
    custom_settings: move_transition::MoveSourceFilterSettings,
    obs_client: &OBSClient,
) -> Result<()> {
    let filter_name = format!("Move_{}", source);

    let _ =
        find_or_create_filter(&scene, &source, &filter_name, obs_client).await;

    move_transition::move_with_move_source(
        scene,
        &filter_name,
        custom_settings,
        &obs_client,
    )
    .await
}

pub async fn customize_wide(
    scene: &str,
    source: &str,
    _x: f32,
    duration: u64,
    easing_function_index: i32,
    easing_type_index: i32,
    obs_client: &OBSClient,
) -> Result<()> {
    let filter_name = format!("Move_{}", source);

    let _ =
        find_or_create_filter(&scene, &source, &filter_name, obs_client).await;

    let mut new_settings =
        move_transition::fetch_source_settings(scene, &source, &obs_client)
            .await?;
    // let mut new_settings =
    //     move_transition::custom_filter_settings(settings, x, y);

    new_settings.duration = Some(duration);
    new_settings.easing_type = Some(easing_type_index);
    new_settings.easing_function = Some(easing_function_index);

    move_transition::move_with_move_source(
        scene,
        &filter_name,
        new_settings,
        &obs_client,
    )
    .await

    // let _ = move_transition_effects::trigger_move_value_3d_transform(
    //     source,
    //     filter_name,
    //     filter_setting_name,
    //     filter_value,
    //     duration,
    //     obs_client,
    // )
    // .await;
}

pub async fn move_source_in_scene_x_and_y(
    scene: &str,
    source: &str,
    x: f32,
    y: f32,
    duration: u64,
    easing_function_index: i32,
    easing_type_index: i32,
    obs_client: &OBSClient,
) -> Result<()> {
    let filter_name = format!("Move_{}", source);

    let _ = find_or_create_filter(&scene, &source, &filter_name, obs_client)
        .await?;

    let settings =
        move_transition::fetch_source_settings(scene, &source, &obs_client)
            .await?;
    let mut new_settings =
        move_transition::custom_filter_settings(settings, x, y);

    new_settings.duration = Some(duration);
    new_settings.easing_type = Some(easing_type_index);
    new_settings.easing_function = Some(easing_function_index);

    move_transition::move_with_move_source(
        scene,
        &filter_name,
        new_settings,
        &obs_client,
    )
    .await
}

pub async fn bottom_right(
    scene: &str,
    scene_item: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let filter_name = format!("Move_{}", scene_item);

    let _ =
        find_or_create_filter(&scene, &scene_item, &filter_name, obs_client)
            .await;

    let settings =
        move_transition::fetch_source_settings(scene, &scene_item, &obs_client)
            .await?;
    let new_settings =
        move_transition::custom_filter_settings(settings, 12.0, 878.0);

    move_transition::move_with_move_source(
        scene,
        &filter_name,
        new_settings,
        &obs_client,
    )
    .await
}

// ===========================================================

// SPIN
pub async fn spin(
    source: &str,
    filter_setting_name: &str,
    filter_value: f32,
    duration: u32,
    obs_client: &OBSClient,
) -> Result<()> {
    // This feels like it belongs somewhere higher-up in the code
    let setting_name = match filter_setting_name {
        "spin" | "z" => "Rotation.Z",
        "spinx" | "x" => "Rotation.X",
        "spiny" | "y" => "Rotation.Y",
        _ => "Rotation.Z",
    };

    // let move_transition_filter_name = format!("Move_{}", three_d_transform_filter_name);
    // println!("MOVE {}", move_transition_filter_name);
    // _ = move_transition::update_and_trigger_move_value_filter(
    //     source,
    //     &move_transition_filter_name,
    //     filter_setting_name,
    //     filter_value,
    //     &three_d_transform_filter_name,
    //     duration,
    //     obs::SINGLE_SETTING_VALUE_TYPE,
    //     &obs_client,
    // )
    // .await;
    // TODO: fix

    let move_filtername = format!("Move_{}", obs::THE_3D_TRANSFORM_FILTER_NAME);

    match move_transition::update_and_trigger_move_value_filter(
        source,
        &move_filtername,
        setting_name,
        filter_value,
        obs::THE_3D_TRANSFORM_FILTER_NAME,
        duration,
        2, // not sure if this is the right value | THIS NEEDS TO BE ABSTRACTED
        &obs_client,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            println!("Error Updating and Triggering Value in !spin {:?}", e);
        }
    }

    Ok(())
}

// =============================================================================

// TODO: This needs some heavy refactoring
// This only affects 3D transforms
pub async fn trigger_3d(
    source: &str,
    filter_setting_name: &str,
    filter_value: f32,
    duration: u32,
    obs_client: &OBSClient,
) -> Result<()> {
    let camera_types_per_filter = stream_fx::camera_type_config();

    let camera_number = camera_types_per_filter[&filter_setting_name];

    let filter_details = obs_client
        .filters()
        .get(&source, obs::THE_3D_TRANSFORM_FILTER_NAME)
        .await;

    let filt: SourceFilter = match filter_details {
        Ok(val) => val,
        Err(_) => return Ok(()),
    };

    let mut new_settings = match serde_json::from_value::<
        stream_fx::StreamFXSettings,
    >(filt.settings)
    {
        Ok(val) => val,
        Err(e) => {
            println!("Error With New Settings: {:?}", e);
            stream_fx::StreamFXSettings {
                ..Default::default()
            }
        }
    };

    // Resetting this Camera Mode
    new_settings.camera_mode = Some(camera_number);

    let new_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: obs::THE_3D_TRANSFORM_FILTER_NAME,
        settings: new_settings,
        overlay: None,
    };
    obs_client.filters().set_settings(new_settings).await?;

    // TODO: Fix
    move_transition::update_and_trigger_move_value_filter(
        source,
        "Move_Stream_FX", // TODO Abstract this
        filter_setting_name,
        filter_value,
        "kjA,,jkjkk",
        duration,
        obs::SINGLE_SETTING_VALUE_TYPE,
        &obs_client,
    )
    .await
}

// Filter name
// OG filter
// Move Filter
//
// Example: OG filter: 3D-Transform
//          Move Filter Move_3D-Transform
//
pub async fn trigger_move_value_3d_transform(
    source: &str,
    filter_name: &str,
    filter_setting_name: &str,
    filter_value: f32,
    duration: u32,
    obs_client: &OBSClient,
) -> Result<()> {
    let three_d_transform_filter_name = filter_name;
    let filter_settings = obs_client
        .filters()
        .get(&source, &three_d_transform_filter_name)
        .await;

    let filt: SourceFilter = match filter_settings {
        Ok(val) => val,
        Err(_) => return Ok(()),
    };
    println!("\nOG 3D Transform Filter Settings: {:?}", filt);

    let new_settings = match serde_json::from_value::<stream_fx::StreamFXSettings>(
        filt.settings,
    ) {
        Ok(val) => val,
        Err(e) => {
            println!("Error With New Settings: {:?}", e);
            stream_fx::StreamFXSettings {
                ..Default::default()
            }
        }
    };
    println!("\nNew 3D Transform Filter Settings: {:?}", new_settings);

    let new_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: filter_name,
        settings: new_settings,
        overlay: None,
    };
    obs_client.filters().set_settings(new_settings).await?;

    let move_transition_filter_name =
        format!("Move_{}", three_d_transform_filter_name);
    println!("MOVE {}", move_transition_filter_name);
    _ = move_transition::update_and_trigger_move_value_filter(
        source,
        &move_transition_filter_name,
        filter_setting_name,
        filter_value,
        &three_d_transform_filter_name,
        duration,
        obs::SINGLE_SETTING_VALUE_TYPE,
        &obs_client,
    )
    .await;
    Ok(())
}
