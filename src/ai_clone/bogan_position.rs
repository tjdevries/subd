use crate::move_transition::duration;
use crate::move_transition::duration::EasingDuration;
use crate::move_transition::models::Coordinates;
use crate::move_transition::move_source::CropSettings;
use crate::move_transition::move_transition;
use anyhow::Result;
use obws::Client as OBSClient;

struct BoganPosition {
    rot: f32,
    x: f32,
    y: f32,
}

pub async fn rotate_bogan_order(
    scene: &str,
    index: i32,
    obs_client: &OBSClient,
) -> Result<()> {
    let position_1 = BoganPosition {
        rot: 0.0,
        x: 0.0,
        y: 800.0,
    };
    let position_2 = BoganPosition {
        rot: 0.0,
        x: 260.0,
        y: 800.0,
    };
    let position_3 = BoganPosition {
        rot: 0.0,
        x: 500.0,
        y: 800.0,
    };
    let position_4 = BoganPosition {
        rot: 0.0,
        x: 750.0,
        y: 800.0,
    };

    let hidden = BoganPosition {
        rot: 0.0,
        x: 0.0,
        y: 1800.0,
    };

    let new_bogan = format!("bogan_{}", index);
    let second_bogan = format!("bogan_{}", index - 1);
    let third_bogan = format!("bogan_{}", index - 2);
    let fourth_bogan = format!("bogan_{}", index - 3);
    let fifth_bogan = format!("bogan_{}", index - 4);
    let _ =
        move_bogan(scene, new_bogan.as_str(), &position_1, &obs_client).await;
    let _ = move_bogan(scene, second_bogan.as_str(), &position_2, &obs_client)
        .await;
    let _ =
        move_bogan(scene, third_bogan.as_str(), &position_3, &obs_client).await;
    let _ = move_bogan(scene, fourth_bogan.as_str(), &position_4, &obs_client)
        .await;
    let _ = move_bogan(scene, fifth_bogan.as_str(), &hidden, &obs_client).await;

    Ok(())
}

async fn move_bogan(
    scene: &str,
    source: &str,
    bogan_position: &BoganPosition,
    obs_client: &OBSClient,
) -> Result<()> {
    // This is where we are trying to scale and crop our source
    let scale = Coordinates::new(Some(0.4), Some(0.4));
    let c = CropSettings::builder().build();
    // let c = CropSettings::builder().left(580.0).build();
    let filter_name = format!("Move_{}", source);

    let d = EasingDuration::builder()
        .duration(1500)
        .easing_function(duration::EasingFunction::Sine)
        .easing_type(duration::EasingType::EaseIn)
        .build();

    // This doesn't take rot!
    move_transition::move_source(
        scene,
        source.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obs::obs;

    #[tokio::test]
    async fn test_bogan_position() {
        let obs_client = obs::create_obs_client().await.unwrap();

        let scene = "BoganArmy";
        let index = crate::ai_clone::bogan::find_current_bogan_index(
            scene,
            &obs_client,
        )
        .await
        .unwrap();

        let bogan_1 = "bogan_13";
        let bogan_2 = "bogan_14";
        let bogan_3 = "bogan_15";
        let bogan_4 = "bogan_16";
    }
}