use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GlowStrokeShadowSettings {
    fill: bool,
    stroke_fill_color: i32,
    stroke_fill_source: String,
    stroke_fill_type: FillTypes,
    stroke_offset: f32,
    stroke_size: f32,
    stroke_source: Option<String>,
}

impl GlowStrokeShadowSettings {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn builder() -> GlowStrokeShadowSettingsBuilder {
        GlowStrokeShadowSettingsBuilder::new()
    }
}

pub struct GlowStrokeShadowSettingsBuilder {
    _fill: Option<bool>,
    _stroke_fill_color: Option<i32>,
    _stroke_fill_source: Option<String>,
    _stroke_fill_type: Option<FillTypes>,
    stroke_offset: Option<f32>,
    stroke_size: Option<f32>,
    _stroke_source: Option<String>,
}

impl Default for GlowStrokeShadowSettingsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GlowStrokeShadowSettingsBuilder {
    pub fn new() -> Self {
        Self {
            _fill: None,
            _stroke_fill_color: None,
            _stroke_fill_source: None,
            _stroke_fill_type: None,
            stroke_offset: None,
            stroke_size: None,
            _stroke_source: None,
        }
    }

    pub fn build(self) -> GlowStrokeShadowSettings {
        GlowStrokeShadowSettings {
            // fill: None,
            // stroke_fill_color: None,
            // stroke_fill_source: None,
            // stroke_fill_type: None,
            stroke_size: self.stroke_size.unwrap_or(1.0),
            stroke_offset: self.stroke_offset.unwrap_or(1.0),
            // stroke_source: None,
            ..Default::default()
        }
    }

    pub fn stroke_size(mut self, stroke_size: f32) -> Self {
        self.stroke_size = Some(stroke_size);
        self
    }

    pub fn stroke_offset(mut self, stroke_offset: f32) -> Self {
        self.stroke_offset = Some(stroke_offset);
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
enum FillTypes {
    #[default]
    Color,
    // stroke_fill_type: Number(2),
}

#[cfg(test)]
mod tests {
    // use super::*;

    //use crate::move_transition::move_transition::update_and_trigger_move_value_for_source;
    //use crate::obs::obs;
    //use crate::obs::obs_source;
    //use serde_json::Value;
    //
    //#[tokio::test]
    //async fn test_glow_stroke_shadow() {
    //    assert_eq!(2 + 2, 4);
    //    let obs_client = obs::create_obs_client().await.unwrap();
    //    let filter = "Move_Outline";
    //    let scene = "OutlineEffects";
    //    let source = "BeginOutline1";
    //
    //    let filter_details =
    //        obs_client.filters().get(source, &filter).await.unwrap();
    //    println!("------------------------");
    //    println!("\n\tFilter Settings: {:?}", filter_details);
    //    println!("------------------------");
    //
    //    // assert_eq!(11, 0);
    //
    //    let item_id = obs_source::find_id(scene, source, &obs_client)
    //        .await
    //        .unwrap();
    //
    //    println!("Item ID: {:?}", item_id);
    //
    //    // I want to see what this so I can write a struct to Deserialize
    //    // But I don't know waht the values to be ignored
    //    // serde::Value
    //    let settings =
    //        obs_client.inputs().settings::<Value>(source).await.unwrap();
    //    println!("------------------------");
    //    println!("\n\tSource: {:?}", settings);
    //    println!("------------------------");
    //
    //    // offset
    //    // stroke_size
    //    // let b = GlowStrokeShadowSettings::builder()
    //    //     .stroke_size(3.0)
    //    //     .stroke_offset(10.0)
    //    //     .build();
    //
    //    let _d = crate::move_transition::duration::EasingDuration::new(300);
    //    // let settings =
    //    //     crate::move_transition::move_value::SingleSourceSetting {
    //    //         source: source.to_string(),
    //    //         setting_float: 10.0,
    //    //         setting_name: "Stroke Size".to_string(),
    //    //         duration: d,
    //    //         ..Default::default()
    //    //     };
    //
    //    let _stroke_size_min = 0.0;
    //    let _stroke_size_max = 100.0;
    //
    //    let _stroke_offset_min = 0.0;
    //    let _stroke_offset_max = 50.0;
    //
    //    if let Err(e) = update_and_trigger_move_value_for_source(
    //        &obs_client,
    //        source.into(),
    //        filter.into(),
    //        // "stroke_size",
    //        "stroke_offset",
    //        1.0,
    //    )
    //    .await
    //    {
    //        println!("Error: {:?}", e);
    //    }
    //    // I need to use this generic value to update the move_transition
    //
    //    // Pass this to the move_transistion
    //
    //    // We are going to want to call move_transition
    //}
}
