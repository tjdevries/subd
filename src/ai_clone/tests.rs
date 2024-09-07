mod tests {
    use crate::ai_clone::bogan;
    use crate::ai_clone::bogan::create_and_show_bogan;
    use crate::constants;
    use crate::obs::obs::create_obs_client;
    use crate::obs::obs_source;

    #[tokio::test]
    async fn test_remove_old_bogans() {
        // Find all sources in a Scene
        // Iterate through and delete
        // Find all Filters in a scene (except the Chroma key)
        // iterate through and delete
        let obs_client = create_obs_client().await.unwrap();
        let scene = "BoganArmy";

        let items = obs_client.scene_items().list(scene).await.unwrap();
        dbg!(&items);
        // source_name: "bogan_122",

        let filters = obs_client.filters().list(scene).await.unwrap();

        for filter in filters {
            if filter.name != "Chroma Key" {
                let _ = obs_client.filters().remove(scene, &filter.name).await;
            }
        }

        for item in items {
            println!("{}", item.source_name);
            obs_client.scene_items().remove(scene, item.id).await;
        }
    }

    // #[tokio::test]
    // async fn test_bogan_army() {
    //     let obs_client = create_obs_client().await.unwrap();
    //     let filter = "Move_bogan_51";
    //     let scene = "BoganArmy";
    //     let filter_details =
    //         obs_client.filters().get(scene, filter).await.unwrap();
    //     let res = ::serde_json::to_string_pretty(&filter_details).unwrap();
    //     println!("res: {}", &res);
    // }

    #[tokio::test]
    async fn test_chat_bogan() {
        let splitmsg: Vec<String> = vec![
            "!bogan".to_string(),
            "disney".to_string(),
            "cat".to_string(),
        ];
        let obs_client = create_obs_client().await.unwrap();
        if let Err(e) = create_and_show_bogan(&obs_client, splitmsg).await {
            eprintln!("Error creating bogan: {}", e);
        }
        // Just enable a filter
    }

    // #[tokio::test]
    // async fn test_create_chroma_key() {
    //     let obs_client = create_obs_client().await.unwrap();
    //     let filter = "Chroma Key";
    //     let source = "bogan_12";
    //     let filter_details =
    //         obs_client.filters().get("bogan_12", filter).await.unwrap();
    //     let res = ::serde_json::to_string_pretty(&filter_details).unwrap();
    //     println!("res: {:?}", &res);
    //
    //     let chroma_key = bogan::ChromaKey { similarity: 420 };
    //     let new_filter = obws::requests::filters::Create {
    //         source,
    //         filter: "chroma_key_filter_v2",
    //         kind: constants::STREAM_FX_INTERNAL_FILTER_NAME,
    //         settings: Some(chroma_key),
    //     };
    //     obs_client.filters().create(new_filter).await.unwrap();
    //
    //     // let scene_item = "bogan_11";
    //     // let res = parse_scene_item_index(scene_item);
    //     // assert_eq!(11, res);
    // }
    // use crate::move_transition::duration;
    // use crate::move_transition::models::Coordinates;

    #[tokio::test]
    async fn test_old() {
        let splitmsg: Vec<String> = vec![
            "!bogan".to_string(),
            "disney".to_string(),
            "cat".to_string(),
        ];
        let obs_client = create_obs_client().await.unwrap();
        if let Err(e) = create_and_show_bogan(&obs_client, splitmsg).await {
            eprintln!("Error creating bogan: {}", e);
        }
        // Just enable a filter
    }

    #[tokio::test]
    async fn test_army_of_bogans() {
        let _obs_client = create_obs_client().await.unwrap();
        let scene = "BoganArmy";
        let source = "Bogan1";

        let _b = obws::requests::scene_items::CreateSceneItem {
            scene,
            source,
            enabled: Some(true),
        };
    }
}
