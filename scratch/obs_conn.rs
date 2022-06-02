// let scene_item_properties = obs_client
//     .scene_items()
//     .get_scene_item_properties(Some("PC"), Either::Left("PC - Elgato"))
//     .await?;
// dbg!(&scene_item_properties);

// let mut pc_settings = obs_client
//     .sources()
//     .get_source_settings::<serde_json::Value>("PC - Elgato", None)
//     .await?;
//
// println!("Settings: {:?}", pc_settings.source_settings);
// if pc_settings.source_name == "PC - Elgato" {
//     // return Ok(());
//     // *pc_settings
//     //     .source_settings
//     //     .get_mut(")
//     //     .unwrap()
//     //     .get_mut("active")
//     //     .unwrap() = Value::Bool(true);
//     pc_settings
//         .source_settings
//         .as_object_mut()
//         .unwrap()
//         .insert("active".to_string(), Value::Bool(true));
// }

// println!("Settings: {:?}", pc_settings.source_settings);
// obs_client
//     .sources()
//     .set_source_settings::<serde_json::Value, serde_json::Value>(
//         obws::requests::SourceSettings {
//             source_name: &pc_settings.source_name,
//             source_type: Some(&pc_settings.source_type),
//             source_settings: &pc_settings.source_settings,
//         },
//     )
//     .await?;

// if pc_settings.source_name == "PC - Elgato" {
//     return Ok(());
// }
// pc_settings.source_settings.

// Turn on/off a filter for a scene
// obs_client
//     .sources()
//     .set_source_filter_visibility(SourceFilterVisibility {
//         source_name: "PC - Elgato",
//         filter_name: "ScrollTest",
//         filter_enabled: true,
//     })
//     .await?;
