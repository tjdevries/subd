use crate::move_transition::duration;
use crate::move_transition::models;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MoveSource {
    pub filter: String,

    pub source: String,
    // #[serde(serialize_with = "move_source_value_type")]
    // pub move_value_type: (),
    //     kind: "move_source_filter",
    #[serde(flatten)]
    pub settings: MoveSourceSettings,

    #[serde(flatten)]
    pub duration: duration::EasingDuration,
}

impl Serialize for MoveSourceSettings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 7 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("MoveSource", 5)?;
        state.serialize_field("bounds", &self.bounds)?;
        state.serialize_field("scale", &self.scale)?;
        state.serialize_field("crop", &self.crop)?;
        state.serialize_field("position", &self.position)?;
        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;
        state.serialize_field("rot", &self.rot)?;

        let default_coordinates = models::Coordinates::default();
        let x = self
            .position
            .unwrap_or(default_coordinates)
            .x
            .unwrap_or(0.0);
        let y = self
            .position
            .unwrap_or(models::Coordinates::default())
            .y
            .unwrap_or(0.0);
        let rot = self.rot.unwrap_or(0.0);
        let scale_x = self
            .scale
            .unwrap_or(models::Coordinates::default())
            .x
            .unwrap_or(0.0);
        let scale_y = self
            .scale
            .unwrap_or(models::Coordinates::default())
            .y
            .unwrap_or(0.0);
        let bounds_x = self
            .bounds
            .unwrap_or(models::Coordinates::default())
            .x
            .unwrap_or(0.0);
        let bounds_y = self
            .bounds
            .unwrap_or(models::Coordinates::default())
            .y
            .unwrap_or(0.0);
        let crop_left = self
            .crop
            .unwrap_or(CropSettings::default())
            .left
            .unwrap_or(0.0);
        let crop_right = self
            .crop
            .unwrap_or(CropSettings::default())
            .right
            .unwrap_or(0.0);
        let crop_top = self
            .crop
            .unwrap_or(CropSettings::default())
            .top
            .unwrap_or(0.0);
        let crop_bottom = self
            .crop
            .unwrap_or(CropSettings::default())
            .bottom
            .unwrap_or(0.0);

        let transform_text = format!(
            "pos: x {} y {} rot: {} bounds: x {} y {} scale: x {} y {} crop: l {} t {} r {} b {}",
            x,
            y,
            rot,
            bounds_x,
            bounds_y,
            scale_x,
            scale_y,
            crop_left,
            crop_top,
            crop_right,
            crop_bottom
        );
        state.serialize_field("transform_text", &transform_text)?;
        state.end()
    }
}

impl MoveSource {
    pub fn new(
        source: impl Into<String>,
        filter: impl Into<String>,
        settings: MoveSourceSettings,
        duration: duration::EasingDuration,
    ) -> Self {
        Self {
            source: source.into(),
            filter: filter.into(),
            settings,
            duration,
            ..Default::default()
        }
    }
}

//     enabled: false,
//     index: 0,
//     kind: "move_source_filter",
//     name: "",
//     "source": String("test-source"),
#[derive(Deserialize, Debug, Default)]
pub struct MoveSourceSettings {
    bounds: Option<models::Coordinates>,
    scale: Option<models::Coordinates>,
    crop: Option<CropSettings>,
    position: Option<models::Coordinates>,
    x: Option<f32>,
    y: Option<f32>,
    rot: Option<f32>,
}

impl MoveSourceSettings {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl CropSettings {
    pub fn new() -> Self {
        Self {
            left: None,
            top: None,
            right: None,
            bottom: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct CropSettings {
    left: Option<f32>,
    top: Option<f32>,
    right: Option<f32>,
    bottom: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::move_transition::move_transition;

    #[tokio::test]
    async fn test_fun() {
        let obs_client = crate::obs::obs::create_obs_client().await.unwrap();
        let duration = crate::move_transition::duration::EasingDuration {
            duration: Some(3000),
            ..Default::default()
        };
        let ms = MoveSourceSettings {
            x: Some(500.0),
            y: Some(100.0),
            position: Some(models::Coordinates {
                x: Some(500.0),
                y: Some(100.0),
            }),

            scale: Some(models::Coordinates {
                x: Some(2.0),
                y: Some(2.0),
            }),
            rot: Some(1090.0),
            ..Default::default()
        };

        let source = "alex";
        let filter_name = "Move_alex";
        let scene = "Memes";
        let settings = MoveSource::new(source, filter_name, ms, duration);

        println!(
            "Settings:\n\n {}",
            serde_json::to_string_pretty(&settings).unwrap()
        );

        let res = move_transition::update_and_trigger_single_value_filter(
            &obs_client,
            scene,
            filter_name,
            settings,
        )
        .await;
        if let Err(err) = res {
            println!("Error: {:?}", err);
        }
    }

    // move source
    // SourceFilter {
    //     settings: Object {
    //         "rot": Number(0.0),
    //         "scale": Object {
    //         "x": Number(1.0),
    //         "y": Number(1.0),
    //     },
    //     "transform_text": String("pos: x 0.0 y 0.0 rot: 0.0 scale: x 1.000 y 1.000 crop: l 0 t 0 r 0 b 0")
    //     }
    // }
}
