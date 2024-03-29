use crate::obs;
use anyhow::Result;
use obws::requests::scene_items::{
    Position, Scale, SceneItemTransform, SetTransform,
};
use obws::Client as OBSClient;

// ==================================================
// == Scaling Sources
// ==================================================

// This takes in x & y
// plus the scene and source to scale the source
//
// the function scale() needs the id and the Scale already
// calculated to scale
pub async fn scale_source(
    scene: &str,
    source: &str,
    x: f32,
    y: f32,
    obs_client: &OBSClient,
) -> Result<(), anyhow::Error> {
    let id = find_id(scene, source, &obs_client).await?;

    println!("scene: {} | ID: {}", scene, id);
    // What's the ID???
    let new_scale = Scale {
        x: Some(x),
        y: Some(y),
    };
    scale(scene, id, new_scale, obs_client).await
}

// Scale for the X & Y of the source in terms of relation to each other,
// and not the overall size in the scene
pub async fn scale(
    scene: &str,
    id: i64,
    new_scale: Scale,
    obs_client: &OBSClient,
) -> Result<()> {
    let scene_transform = SceneItemTransform {
        scale: Some(new_scale),
        ..Default::default()
    };

    let set_transform = SetTransform {
        scene,
        item_id: id,
        transform: scene_transform,
    };
    let result = obs_client.scene_items().set_transform(set_transform).await;
    match result {
        Ok(_) => {
            println!("Successful Scale of Scene Item: {:?}", id);
        }
        Err(err) => {
            println!("Error Scaling Source: {:?}", err);
        }
    }
    Ok(())
}

// TODO: This has problems
pub async fn trigger_grow(
    scene: &str,
    source: &str,
    base_scale: &Scale,
    x: f32,
    y: f32,
    obs_client: &OBSClient,
) -> Result<(), anyhow::Error> {
    // This has an "all" concept???
    // This also takes in a scene, BUT has DEFAULT_SCENE and MEME_SCENE
    // hardcoded into it
    if source == "all" {
        let sources = obs_client.scene_items().list(obs::DEFAULT_SCENE).await?;
        for source in sources {
            let new_scale = Scale {
                x: base_scale.x,
                y: base_scale.y,
            };

            // Do we need to do this not to crash all the time???
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

            scale(obs::MEME_SCENE, id, new_scale, &obs_client).await?;
        }
    } else {
        println!("ABOUT TO SCALE SOURCE: {} {}", scene, source);
        scale_source(&scene, &source, x, y, &obs_client).await?;
    }
    Ok(())
}

// ====================================================
// == Moving Things
// ====================================================

// Move a Source using x, y and NO Filters
pub async fn move_source(
    scene: &str,
    source: &str,
    x: f32,
    y: f32,
    obs_client: &OBSClient,
) -> Result<(), anyhow::Error> {
    let id = find_id(scene, source, &obs_client).await?;

    let new_position = Position {
        x: Some(x),
        y: Some(y),
    };
    let scene_transform = SceneItemTransform {
        position: Some(new_position),
        ..Default::default()
    };

    let set_transform = SetTransform {
        scene,
        item_id: id,
        transform: scene_transform,
    };
    let _ = obs_client.scene_items().set_transform(set_transform).await;
    Ok(())
}

// ====================================================
// == Hide/Show Actions
// ====================================================

pub async fn show_source(
    scene: &str,
    source: &str,
    obs_client: &OBSClient,
) -> Result<(), anyhow::Error> {
    set_enabled(scene, source, true, obs_client).await
}

pub async fn hide_sources(
    scene: &str,
    obs_client: &OBSClient,
) -> Result<(), anyhow::Error> {
    set_enabled_on_all_sources(scene, false, &obs_client).await
}

pub async fn set_enabled(
    scene: &str,
    source: &str,
    enabled: bool,
    obs_client: &OBSClient,
) -> Result<(), anyhow::Error> {
    let id = find_id(scene, source, &obs_client).await?;

    let set_enabled: obws::requests::scene_items::SetEnabled =
        obws::requests::scene_items::SetEnabled {
            enabled,
            item_id: id,
            scene,
        };

    let _ = obs_client.scene_items().set_enabled(set_enabled).await;
    Ok(())
}

async fn set_enabled_on_all_sources(
    scene: &str,
    enabled: bool,
    obs_client: &OBSClient,
) -> Result<(), anyhow::Error> {
    let items = obs_client.scene_items().list(scene).await?;
    for item in items {
        // If we can't set an item as enabled we just move on with our lives
        let _ =
            set_enabled(scene, &item.source_name, enabled, &obs_client).await;
    }
    Ok(())
}

// ====================================================
// == Debug Info
// ====================================================

pub async fn print_source_info(
    source: &str,
    scene: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let id = find_id(obs::MEME_SCENE, source, &obs_client).await?;
    let settings = obs_client.scene_items().transform(scene, id).await?;

    println!("Source Settings: {:?}", settings);
    Ok(())
}

// ====================================================
// == Utility
// ====================================================

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
