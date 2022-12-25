use crate::move_transition;
use crate::obs;
use crate::obs_hotkeys;
use anyhow::Result;
use obws;
use obws::requests::scene_items::{Scale, SceneItemTransform, SetTransform};
use obws::Client as OBSClient;

pub async fn trigger_character_filters(
    base_source: &str,
    obs_client: &OBSClient,
    enabled: bool,
) -> Result<()> {
    let scene = "Characters";

    let mut filter_name_modifier = "Hide";
    if enabled {
        filter_name_modifier = "Show"
    };

    // println!(
    //     "We are going to try and {} {} sources",
    //     filter_name_modifier, base_source
    // );

    // So this just fails
    let filter_name = format!("{}{}", filter_name_modifier, base_source);
    let filter_enabled = obws::requests::filters::SetEnabled {
        source: scene,
        filter: &filter_name,
        enabled: true,
    };
    // println!("Attempting to Trigger: {}", filter_name);
    obs_client.filters().set_enabled(filter_enabled).await?;

    let filter_name = format!("{}{}-text", filter_name_modifier, base_source);
    let filter_enabled = obws::requests::filters::SetEnabled {
        source: scene,
        filter: &filter_name,
        enabled: true,
    };
    // println!("Attempting to Trigger: {}", filter_name);
    obs_client.filters().set_enabled(filter_enabled).await?;

    let filter_name =
        format!("{}{}-speech_bubble", filter_name_modifier, base_source);
    let filter_enabled = obws::requests::filters::SetEnabled {
        source: scene,
        filter: &filter_name,
        enabled: true,
    };
    // println!("Attempting to Trigger: {}", filter_name);
    obs_client.filters().set_enabled(filter_enabled).await?;

    Ok(())
}

pub async fn norm(source: &str, obs_client: &OBSClient) -> Result<()> {
    println!("Attempting to Make: {source} normal!");

    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: &obs::DEFAULT_STREAM_FX_FILTER_NAME,
        enabled: true,
    };
    match obs_client.filters().set_enabled(filter_enabled).await {
        Ok(_) => {}
        Err(_) => return Ok(()),
    }
    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: &obs::DEFAULT_SCROLL_FILTER_NAME,
        enabled: true,
    };
    // This is not the way
    match obs_client.filters().set_enabled(filter_enabled).await {
        Ok(_) => {}
        Err(_) => return Ok(()),
    }
    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: &obs::DEFAULT_BLUR_FILTER_NAME,
        enabled: true,
    };
    match obs_client.filters().set_enabled(filter_enabled).await {
        Ok(_) => {}
        Err(_) => return Ok(()),
    }

    let id = match find_id(obs::MEME_SCENE, source, &obs_client).await {
        Ok(val) => val,
        Err(_) => return Ok(()),
    };

    let new_scale = Scale {
        x: Some(1.0),
        y: Some(1.0),
    };
    match scale(id, new_scale, &obs_client).await {
        Ok(_) => {}
        Err(e) => println!("{:?}", e),
    }

    Ok(())
}

// ============================================================================
// ============================================================================
// ============================================================================
// ============================================================================

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

async fn find_id(
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

// =======================================================================================

pub async fn staff(source: &str, obs_client: &OBSClient) -> Result<()> {
    _ = move_transition::update_and_trigger_move_value_filter(
        source,
        "Move_Blur",
        "Filter.Blur.Size",
        100.0,
        5000,
        2,
        &obs_client,
    )
    .await;

    // What are these doing here like this?
    let filter_name = "Move_Source";
    let filter_setting_name = "speed_x";
    let filter_value = -115200.0;
    let duration = 5000;
    move_transition::update_and_trigger_move_value_filter(
        source,
        filter_name,
        filter_setting_name,
        filter_value,
        duration,
        2,
        &obs_client,
    )
    .await?;

    // TODO: This should triggered from a fucntion
    obs_client
        .hotkeys()
        .trigger_by_sequence("OBS_KEY_U", obs_hotkeys::SUPER_KEY)
        .await?;
    Ok(())
}

// TODO: This needs some heavy refactoring
pub async fn follow(
    scene: &str,
    source: &str,
    leader: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let id = match find_id(obs::MEME_SCENE, source, &obs_client).await {
        Ok(val) => val,
        Err(_) => return Ok(()),
    };

    let untouchable_sources: Vec<String> = vec![
        "Screen".to_string(),
        "SBF_Condo".to_string(),
        "Top Screen".to_string(),
        "twitchchat".to_string(),
        "Emotes".to_string(),
        "TwitchAlerts".to_string(),
    ];

    let leader_settings =
        match obs_client.scene_items().transform(scene, id).await {
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

    let sources = obs_client.scene_items().list(obs::DEFAULT_SCENE).await?;
    for s in sources {
        for bad_source in &untouchable_sources {
            if bad_source == &s.source_name {
                // Will this continue out of the whole function???
                continue;
            }
        }
        let base_settings = match move_transition::fetch_source_settings(
            obs::DEFAULT_SCENE,
            &s.source_name,
            &obs_client,
        )
        .await
        {
            Ok(val) => val,
            Err(err) => {
                println!("Error fetching Source Settings: {:?}", err);
                continue;
            }
        };
        if s.source_name != leader {
            let new_settings = move_transition::custom_filter_settings(
                base_settings,
                leader_settings.position_x,
                leader_settings.position_y,
            );
            let filter_name = format!("Move_Source_{}", s.source_name);
            _ = move_transition::move_with_move_source(
                &filter_name,
                new_settings,
                &obs_client,
            )
            .await;
        }
    }

    Ok(())
}
