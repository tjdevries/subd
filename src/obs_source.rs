use crate::move_transition;
use crate::obs;
use anyhow::Result;
use obws::requests::scene_items::{
    Position, Scale, SceneItemTransform, SetTransform,
};
use obws::Client as OBSClient;

pub async fn find_id(
    scene: &str,
    source: &str,
    obs_client: &OBSClient,
) -> Result<i64, obws::Error> {
    let id_search = obws::requests::scene_items::Id {
        scene,
        source,
        ..Default::default()
    };

    obs_client.scene_items().id(id_search).await
}

// =============== //
// Scaling Sources //
// =============== //

pub async fn scale_source(
    scene: &str,
    source: &str,
    x: f32,
    y: f32,
    obs_client: &OBSClient,
) -> Result<()> {
    println!("Looking for ID: {} {}", scene, source);

    let id = match find_id(scene, source, &obs_client).await {
        Ok(val) => val,
        Err(err) => {
            println!("Error find_id: {:?}", err);
            return Ok(());
        }
    };
    println!("ID: {}", id);

    let new_scale = Scale {
        x: Some(x),
        y: Some(y),
    };
    let scene_transform = SceneItemTransform {
        scale: Some(new_scale),
        ..Default::default()
    };

    let set_transform = SetTransform {
        scene,
        item_id: id,
        transform: scene_transform,
    };
    match obs_client.scene_items().set_transform(set_transform).await {
        Ok(_) => {}
        Err(err) => {
            println!("Error Set Transform: {:?}", err);
        }
    }

    Ok(())
}

pub async fn scale(
    id: i64,
    new_scale: Scale,
    obs_client: &OBSClient,
) -> Result<()> {
    let scene_transform = SceneItemTransform {
        scale: Some(new_scale),
        ..Default::default()
    };

    let set_transform = SetTransform {
        scene: obs::DEFAULT_SCENE,
        item_id: id,
        transform: scene_transform,
    };
    obs_client
        .scene_items()
        .set_transform(set_transform)
        .await?;
    Ok(())
}

// ================ //
// Just Move Things //
// ================ //

// Move a Source using x, y and NO Filters
pub async fn move_source(
    scene: &str,
    source: &str,
    x: f32,
    y: f32,
    obs_client: &OBSClient,
) -> Result<()> {
    let id = match find_id(scene, source, &obs_client).await {
        Ok(val) => val,
        Err(e) => {
            println!("Error Finding ID {:?} {:?}", source, e);
            return Ok(());
        }
    };

    let new_position = Position {
        x: Some(x),
        y: Some(y),
    };
    let scene_transform = SceneItemTransform {
        position: Some(new_position),
        ..Default::default()
    };

    // TODO: figure out looking up Scene, based on Source
    let set_transform = SetTransform {
        scene,
        item_id: id,
        transform: scene_transform,
    };
    match obs_client.scene_items().set_transform(set_transform).await {
        Ok(_) => {}
        Err(e) => {
            println!("Error Transforming Scene: {:?}", e)
        }
    }

    Ok(())
}

pub async fn top_right(scene_item: &str, obs_client: &OBSClient) -> Result<()> {
    let base_settings = move_transition::fetch_source_settings(
        obs::DEFAULT_SCENE,
        &scene_item,
        &obs_client,
    )
    .await?;

    let new_settings =
        move_transition::custom_filter_settings(base_settings, 1662.0, 13.0);
    let filter_name = format!("Move_Source_{}", scene_item);
    move_transition::move_with_move_source(
        &filter_name,
        new_settings,
        &obs_client,
    )
    .await
}

pub async fn bottom_right(
    scene_item: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let settings = move_transition::fetch_source_settings(
        obs::DEFAULT_SCENE,
        &scene_item,
        &obs_client,
    )
    .await?;

    let new_settings =
        move_transition::custom_filter_settings(settings, 12.0, 878.0);
    let filter_name = format!("Move_Source_{}", scene_item);
    move_transition::move_with_move_source(
        &filter_name,
        new_settings,
        &obs_client,
    )
    .await
}

// ================= //
// Hide/Show Actions //
// ================= //

pub async fn show_source(
    scene: &str,
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    set_enabled(scene, source, true, obs_client).await
}

pub async fn set_enabled(
    scene: &str,
    source: &str,
    enabled: bool,
    obs_client: &OBSClient,
) -> Result<()> {
    match find_id(scene, source, &obs_client).await {
        Err(e) => {
            println!("Error finding ID for source {:?} {:?}", source, e)
        }
        Ok(id) => {
            let set_enabled: obws::requests::scene_items::SetEnabled =
                obws::requests::scene_items::SetEnabled {
                    enabled,
                    item_id: id,
                    scene,
                };

            match obs_client.scene_items().set_enabled(set_enabled).await {
                Err(e) => {
                    println!("Error Enabling Source: {:?} {:?}", source, e);
                }
                _ => (),
            }
        }
    };
    Ok(())
}

async fn set_enabled_on_all_sources(
    scene: &str,
    enabled: bool,
    obs_client: &OBSClient,
) -> Result<()> {
    match obs_client.scene_items().list(scene).await {
        Ok(items) => {
            for item in items {
                match set_enabled(
                    scene,
                    &item.source_name,
                    enabled,
                    &obs_client,
                )
                .await
                {
                    Ok(_) => (),
                    Err(e) => {
                        println!(
                            "Error SetEnabled State of source {:?} {:?}",
                            item.source_name, e
                        );
                    }
                }
            }
            return Ok(());
        }
        Err(e) => {
            println!("Error listing Scene Items for {:?} {:?}", scene, e);
            return Ok(());
        }
    }
}

pub async fn hide_sources(scene: &str, obs_client: &OBSClient) -> Result<()> {
    set_enabled_on_all_sources(scene, false, &obs_client).await
}

// ===========================================
//
// SCALING AGAIN
pub async fn trigger_grow(
    scene: &str,
    source: &str,
    base_scale: &Scale,
    x: f32,
    y: f32,
    obs_client: &OBSClient,
) -> Result<()> {
    println!(
        "\n\t~~~ Attempting to Scale: {} | X: {:?} Y: {:?}",
        source, base_scale.x, base_scale.y
    );

    if source == "all" {
        let sources = obs_client.scene_items().list(obs::DEFAULT_SCENE).await?;
        for source in sources {
            let new_scale = Scale {
                x: base_scale.x,
                y: base_scale.y,
            };
            let id = match find_id(
                obs::MEME_SCENE,
                &source.source_name,
                &obs_client,
            )
            .await
            {
                Ok(val) => val,
                Err(_) => return Ok(()),
            };

            if let Err(err) = scale(id, new_scale, &obs_client).await {
                println!("Error Finding ID: {}", err)
            };
        }
    } else {
        if let Err(err) = scale_source(&scene, &source, x, y, &obs_client).await
        {
            println!("Error Scaling Source: {}", err)
        };
    }
    Ok(())
}

pub async fn print_source_info(
    source: &str,
    scene: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let id = match find_id(obs::MEME_SCENE, source, &obs_client).await {
        Ok(val) => val,
        Err(_) => return Ok(()),
    };

    let settings = match obs_client.scene_items().transform(scene, id).await {
        Ok(val) => val,
        Err(err) => {
            println!("Error Fetching Transform Settings: {:?}", err);
            let blank_transform =
                obws::responses::scene_items::SceneItemTransform {
                    ..Default::default()
                };
            blank_transform
        }
    };

    println!("Source: {:?}", settings);
    Ok(())
}
