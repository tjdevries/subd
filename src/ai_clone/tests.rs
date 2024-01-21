mod tests {
    use crate::ai_clone::bogan;
    use crate::ai_clone::bogan::create_and_show_bogan;
    use crate::constants;
    use crate::obs::obs::create_obs_client;
    use crate::obs::obs_source;

    #[tokio::test]
    async fn test_bogan_army() {
        let obs_client = create_obs_client().await.unwrap();
    }

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

    #[tokio::test]
    async fn test_create_chroma_key() {
        let obs_client = create_obs_client().await.unwrap();
        let filter = "Chroma Key";
        let source = "bogan_12";
        let filter_details =
            obs_client.filters().get("bogan_12", filter).await.unwrap();
        let res = ::serde_json::to_string_pretty(&filter_details).unwrap();
        println!("res: {:?}", &res);

        let chroma_key = bogan::ChromaKey { similarity: 420 };
        let new_filter = obws::requests::filters::Create {
            source,
            filter: "chroma_key_filter_v2",
            kind: constants::STREAM_FX_INTERNAL_FILTER_NAME,
            settings: Some(chroma_key),
        };
        obs_client.filters().create(new_filter).await.unwrap();

        // let scene_item = "bogan_11";
        // let res = parse_scene_item_index(scene_item);
        // assert_eq!(11, res);
    }
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
