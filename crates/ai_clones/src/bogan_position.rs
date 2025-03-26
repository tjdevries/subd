use anyhow::Result;
use obs_move_transition;
use obws::Client as OBSClient;

#[warn(dead_code)]
struct BoganPosition {
    _rot: f32,
    x: f32,
    y: f32,
}

pub async fn rotate_bogan_order(
    scene: &str,
    index: i32,
    obs_client: &OBSClient,
) -> Result<()> {
    let position_1 = BoganPosition {
        _rot: 0.0,
        x: 0.0,
        y: 800.0,
    };
    let position_2 = BoganPosition {
        _rot: 0.0,
        x: 260.0,
        y: 800.0,
    };
    let position_3 = BoganPosition {
        _rot: 0.0,
        x: 500.0,
        y: 800.0,
    };
    let position_4 = BoganPosition {
        _rot: 0.0,
        x: 750.0,
        y: 800.0,
    };

    let hidden = BoganPosition {
        _rot: 0.0,
        x: 0.0,
        y: 1800.0,
    };

    let new_bogan = format!("bogan_{}", index);
    let second_bogan = format!("bogan_{}", index - 1);
    let third_bogan = format!("bogan_{}", index - 2);
    let fourth_bogan = format!("bogan_{}", index - 3);
    let fifth_bogan = format!("bogan_{}", index - 4);
    let _ =
        move_bogan(scene, new_bogan.as_str(), &position_1, obs_client).await;
    let _ =
        move_bogan(scene, second_bogan.as_str(), &position_2, obs_client).await;
    let _ =
        move_bogan(scene, third_bogan.as_str(), &position_3, obs_client).await;
    let _ =
        move_bogan(scene, fourth_bogan.as_str(), &position_4, obs_client).await;
    let _ = move_bogan(scene, fifth_bogan.as_str(), &hidden, obs_client).await;

    Ok(())
}

async fn move_bogan(
    scene: &str,
    source: &str,
    bogan_position: &BoganPosition,
    obs_client: &OBSClient,
) -> Result<()> {
    // This is where we are trying to scale and crop our source
    let scale =
        obs_move_transition::models::Coordinates::new(Some(0.4), Some(0.4));
    let c = obs_move_transition::move_source::CropSettings::builder().build();
    // let c = CropSettings::builder().left(580.0).build();
    let filter_name = format!("Move_{}", source);

    let d = obs_move_transition::duration::EasingDuration::builder()
        .duration(1500)
        .easing_function(obs_move_transition::duration::EasingFunction::Sine)
        .easing_type(obs_move_transition::duration::EasingType::EaseIn)
        .build();

    // This doesn't take rot!
    obs_move_transition::move_source(
        scene,
        source,
        filter_name.clone(),
        false,
        Some(bogan_position.x),
        Some(bogan_position.y),
        Some(c),
        Some(scale),
        Some(d),
        obs_client,
    )
    .await
}
