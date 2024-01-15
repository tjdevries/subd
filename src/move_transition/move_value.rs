use crate::move_transition::duration::EasingDuration;
use serde::{Deserialize, Serialize, Serializer};
use serde_repr::*;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SingleSetting {
    #[serde(rename = "filter")]
    pub target_filter: String,

    pub setting_float: f32,

    pub setting_name: String,

    #[serde(serialize_with = "single_setting")]
    pub move_value_type: (),

    #[serde(flatten)]
    pub duration: EasingDuration,
}

impl SingleSetting {
    pub fn new(
        target_filter: impl Into<String>,
        setting_name: impl Into<String>,
        setting_float: f32,
        duration: EasingDuration,
    ) -> Self {
        Self {
            target_filter: target_filter.into(),
            setting_name: setting_name.into(),
            setting_float,
            duration,
            ..Default::default()
        }
    }
}

fn single_setting<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_i32(MoveValueType::SingleSetting as i32)
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Settings<T> {
    #[serde(rename = "filter")]
    pub target_filter: String,

    #[serde(serialize_with = "settings")]
    pub move_value_type: (),

    #[serde(flatten)]
    pub settings: T,

    #[serde(flatten)]
    pub duration: EasingDuration,
}

pub struct SettingsBuilder<T> {
    pub target_filter: String,
    pub settings: Option<T>,
    pub duration: EasingDuration,
}

impl<T: serde::Serialize + std::default::Default> Settings<T> {
    pub fn new(
        target_filter: impl Into<String>,
        settings: T,
        duration: EasingDuration,
    ) -> Self {
        Self {
            target_filter: target_filter.into(),
            settings,
            duration,
            ..Default::default()
        }
    }
}

impl Add {
    pub fn new(
        target_filter: impl Into<String>,
        setting_name: impl Into<String>,
        setting_float: f32,
        duration: EasingDuration,
    ) -> Self {
        Self {
            target_filter: target_filter.into(),
            setting_name: setting_name.into(),
            setting_float,
            duration,
            ..Default::default()
        }
    }
}

// This is like a single settings
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Add {
    #[serde(rename = "filter")]
    pub target_filter: String,

    // This could always be the value
    #[serde(serialize_with = "add")]
    move_value_type: (),

    // Might need to be optional
    pub setting_float: f32,

    pub setting_name: String,

    #[serde(flatten)]
    pub duration: EasingDuration,
}

// This is like a single settings
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Random {
    #[serde(rename = "filter")]
    pub target_filter: String,

    #[serde(serialize_with = "random")]
    move_value_type: (),

    pub setting_name: String,
    pub setting_float_min: f32,
    pub setting_float_max: f32,

    // Takes a min and max value
    #[serde(flatten)]
    pub duration: EasingDuration,
}

impl Random {
    pub fn new(
        target_filter: impl Into<String>,
        setting_name: impl Into<String>,
        setting_float_min: f32,
        setting_float_max: f32,
        duration: EasingDuration,
    ) -> Self {
        Self {
            target_filter: target_filter.into(),
            setting_name: setting_name.into(),
            setting_float_min,
            setting_float_max,
            duration,
            ..Default::default()
        }
    }
}

// Has to be on a typing source
// This is like a single settings
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Typing {
    pub filter: String,

    // This could always be the value
    #[serde(serialize_with = "typing")]
    move_value_type: (),

    #[serde(flatten)]
    pub duration: EasingDuration,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
enum MoveValueType {
    SingleSetting = 0,
    Settings = 1,
    Random = 2,
    Add = 3,
    Typing = 4,
}

fn settings<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_i32(MoveValueType::Settings as i32)
}

fn random<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_i32(MoveValueType::Random as i32)
}

fn add<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_i32(MoveValueType::Add as i32)
}

fn typing<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_i32(MoveValueType::Typing as i32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::move_transition::move_transition;

    #[tokio::test]
    async fn test_settings() {
        let source = "alex";
        let filter_name = "move-value-single";

        let obs_client = crate::obs::obs::create_obs_client().await.unwrap();
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
            crate::obs_filters::three_d_transform::ThreeDTransformPerspective {
                field_of_view: Some(90.0),
                ..Default::default()
            };
        let req = Settings::new(
            "3D-Transform-Perspective",
            threed,
            duration_settings,
        );
        let _ = move_transition::update_and_trigger_single_value_filter(
            &obs_client,
            source,
            filter_name,
            req,
        )
        .await;
    }

    #[tokio::test]
    async fn test_random() {
        let source = "alex";
        let filter_name = "move-value-single";

        let obs_client = crate::obs::obs::create_obs_client().await.unwrap();
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
        let _ = move_transition::update_and_trigger_single_value_filter(
            &obs_client,
            source,
            filter_name,
            req,
        )
        .await;
    }

    #[tokio::test]
    async fn test_add() {
        let source = "alex";
        let filter_name = "move-value-single";

        let obs_client = crate::obs::obs::create_obs_client().await.unwrap();
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
        let _ = move_transition::update_and_trigger_single_value_filter(
            &obs_client,
            source,
            filter_name,
            req,
        )
        .await;
    }

    #[tokio::test]
    async fn test_single_setting() {
        let source = "alex";
        let filter_name = "move-value-single";

        let obs_client = crate::obs::obs::create_obs_client().await.unwrap();
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

        let _ = move_transition::update_and_trigger_single_value_filter(
            &obs_client,
            source,
            filter_name,
            req,
        )
        .await;
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
