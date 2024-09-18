#[cfg(test)]
mod tests {
    use crate::move_transition::duration::EasingDuration;
    use crate::move_transition::models::Coordinates;
    use crate::move_transition::move_source::{MoveSource, MoveSourceSettings};
    use crate::move_transition::move_transition;
    use crate::move_transition::move_value::*;
    use obs_service;

    #[tokio::test]
    async fn test_duration() {
        let obj = EasingDuration::new(3000);
        let res = serde_json::to_string(&obj);
        println!("{:?}", res);
        // Get a new object serial/deserial
        // let obs_client = crate::obs::obs::create_obs_client().await.unwrap();
        // let filter_details =
        //     obs_client.filters().get("test-source", "move-value").await.unwrap();
        // let res = ::serde_json::to_string_pretty(&filter_details).unwrap();
        // println!("\nMove Value\n{}", res);

        // let filter_details =
        //     obs_client.filters().get("test-source", "move-action").await.unwrap();
        // let res = ::serde_json::to_string_pretty(&filter_details).unwrap();
        // println!("\nMove Action\n{}", res);
        //
        // let filter_details =
        //     obs_client.filters().get("test-scene", "move-source").await.unwrap();
        // let res = ::serde_json::to_string_pretty(&filter_details).unwrap();
        // println!("\nMove Source\n{}", res);
        // let x = MoveValueType::Settings as i32;
        // println!("X: {}", x);
    }

    #[tokio::test]
    async fn test_settings() {
        let source = "alex";
        let filter_name = "move-value-single";

        let obs_client = obs_service::obs::create_obs_client().await.unwrap();
        let filter_details =
            obs_client.filters().get(source, filter_name).await.unwrap();
        let res = ::serde_json::to_string_pretty(&filter_details).unwrap();
        let duration_settings =
            crate::move_transition::duration::EasingDuration {
                duration: Some(3000),
                ..Default::default()
            };
        println!("\nMove Value\n{}", res);

        let threed =
            crate::three_d_filter::perspective::ThreeDTransformPerspective::builder()
                .field_of_view(Some(90.0))
                .build();
        let req = Settings::new(
            "3D-Transform-Perspective",
            threed,
            duration_settings,
        );
        let _ = move_transition::update_filter_and_enable(
            source,
            filter_name,
            req,
            &obs_client,
        )
        .await;
    }

    #[tokio::test]
    async fn test_random() {
        let source = "alex";
        let filter_name = "move-value-single";

        let obs_client = obs_service::obs::create_obs_client().await.unwrap();
        let filter_details =
            obs_client.filters().get(source, filter_name).await.unwrap();
        let res = ::serde_json::to_string_pretty(&filter_details).unwrap();
        let duration_settings =
            crate::move_transition::duration::EasingDuration {
                duration: Some(3000),
                ..Default::default()
            };
        println!("\nMove Value\n{}", res);

        let _saturation_rng = [-1, 5];
        let req =
            Random::new("Scroll", "speed_x", 0.0, 100.0, duration_settings);
        let _ = move_transition::update_filter_and_enable(
            source,
            filter_name,
            req,
            &obs_client,
        )
        .await;
    }

    #[tokio::test]
    async fn test_add() {
        let source = "alex";
        let filter_name = "move-value-single";

        let obs_client = obs_service::obs::create_obs_client().await.unwrap();
        let filter_details =
            obs_client.filters().get(source, filter_name).await.unwrap();
        let res = ::serde_json::to_string_pretty(&filter_details).unwrap();
        let duration_settings =
            crate::move_transition::duration::EasingDuration {
                duration: Some(3000),
                ..Default::default()
            };
        println!("\nMove Value\n{}", res);

        let _saturation_rng = [-1, 5];
        // let req = Add::new(
        //     "color",
        //     "saturation",
        //     1.0,
        //     duration_settings
        // );

        let req = Add::new("Scroll", "speed_x", 100.0, duration_settings);
        // let req = Add::new(
        //     "Blur",
        //     "Filter.Blur.Size",
        //     10.0,
        //     duration_settings
        // );
        let _ = move_transition::update_filter_and_enable(
            source,
            filter_name,
            req,
            &obs_client,
        )
        .await;
    }

    #[tokio::test]
    async fn test_single_setting() {
        let source = "alex";
        let filter_name = "move-value-single";

        let obs_client = obs_service::obs::create_obs_client().await.unwrap();
        let filter_details =
            obs_client.filters().get(source, filter_name).await.unwrap();
        let res = ::serde_json::to_string_pretty(&filter_details).unwrap();
        let duration_settings =
            crate::move_transition::duration::EasingDuration {
                duration: Some(3000),
                ..Default::default()
            };
        println!("\nMove Value\n{}", res);
        let req =
            SingleSetting::new("color", "opacity", -0.99, duration_settings);

        let _ = move_transition::update_filter_and_enable(
            source,
            filter_name,
            req,
            &obs_client,
        )
        .await;
    }

    // use crate::move_transition::move_source::Sign;

    // BoganArmy Scene
    // List of Bogan Image File Names
    // Method for moving Bogans away from each other + Cropping Logic
    #[tokio::test]
    async fn test_fun() {
        let obs_client = obs_service::obs::create_obs_client().await.unwrap();
        let duration =
            crate::move_transition::duration::EasingDuration::new(300);
        let ms = MoveSourceSettings::builder()
            .position(Coordinates::new(Some(100.0), Some(100.0)))
            .rot(360.0)
            .scale(Coordinates::new(Some(1.2), Some(1.2)))
            .relative_transform(true)
            .build();
        let res = serde_json::to_string_pretty(&ms).unwrap();
        println!("{:?}", res);

        // let source = "technofroggo";
        let source = "alex";
        let filter_name = "Move_alex";
        let settings = MoveSource::new(source, filter_name, ms, duration);
        let scene = "Memes";
        let res = move_transition::update_filter_and_enable(
            scene,
            filter_name,
            settings,
            &obs_client,
        )
        .await;
        if let Err(err) = res {
            println!("Error: {:?}", err);
        }
    }

    #[tokio::test]
    async fn test_scale() {
        let obs_client = obs_service::obs::create_obs_client().await.unwrap();
        let duration =
            crate::move_transition::duration::EasingDuration::new(300);
        let ms = MoveSourceSettings::builder()
            .x(100.0)
            .y(100.0)
            .bounds(Coordinates::new(Some(500.0), Some(100.0)))
            .scale(Coordinates::new(Some(1.2), Some(1.2)))
            .rot(1080.0)
            .build();

        let source = "alex";
        let filter_name = "Move_alex";
        let scene = "Memes";
        let settings = MoveSource::new(source, filter_name, ms, duration);

        println!(
            "Settings:\n\n {}",
            serde_json::to_string_pretty(&settings).unwrap()
        );

        let res = move_transition::update_filter_and_enable(
            scene,
            filter_name,
            settings,
            &obs_client,
        )
        .await;
        if let Err(err) = res {
            println!("Error: {:?}", err);
        }
    }
    // Min Blur
    //
    // Max Blur
    //
    // More Blur
    //
    // Less Blur
    //
    // Random Blur
}
