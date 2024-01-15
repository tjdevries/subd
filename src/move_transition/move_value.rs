use crate::move_transition::duration::EasingDuration;
use crate::move_transition::move_transition;
use serde::{Deserialize, Serialize, Serializer};
use serde_repr::*;

// How do I make sure each of these have a default?
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SingleSetting {
    pub filter: String,

    #[serde(serialize_with = "single_setting")]
    move_value_type: (),

    // I think single setting is 2
    // #[serde(serialize_with = "value_type")]
    // pub value_type: (),
    #[serde(flatten)]
    pub duration: EasingDuration,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Settings<T> {
    pub filter: String,

    // This could always be the value
    #[serde(serialize_with = "settings")]
    move_value_type: (),

    // We need to handle these
    // // These should be Enums
    // #[serde(serialize_with = "value_type")]
    // pub value_type: (),
    #[serde(flatten)]
    pub settings: T,

    #[serde(flatten)]
    pub duration: EasingDuration,
}

// This is like a single settings
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Add {
    pub filter: String,

    // This could always be the value
    #[serde(serialize_with = "add")]
    move_value_type: (),

    // #[serde(serialize_with = "value_type")]
    // pub value_type: (),

    // This is like the single Settings
    // #[serde(flatten)]
    // pub settings: T,
    #[serde(flatten)]
    pub duration: EasingDuration,
}

// This is like a single settings
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Random {
    pub filter: String,

    // This could always be the value
    #[serde(serialize_with = "random")]
    move_value_type: (),

    // We need to handle these
    // // These should be Enums
    // #[serde(serialize_with = "value_type")]
    // pub value_type: (),

    // Takes a min and max value
    #[serde(flatten)]
    pub duration: EasingDuration,
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

// Single Settings
// {
//   "filterEnabled": false,
//   "filterIndex": 2,
//   "filterKind": "move_value_filter",
//   "filterName": "",
//   "filterSettings": {
//     "filter": "color",
//     "move_value_type": 0,
//     "setting_float": 20.0,
//     "setting_float_max": 0.04,
//     "setting_name": "contrast",
//     "value_type": 2
//   }
// }
//
// Move Value
// Settings
// {
//   "filterEnabled": false,
//   "filterIndex": 2,
//   "filterKind": "move_value_filter",
//   "filterName": "",
//   "filterSettings": {
//     "filter": "color",
//     "move_value_type": 1,
//     "setting_float": 20.0,
//     "setting_float_max": 0.04,
//     "setting_name": "contrast",
//     "value_type": 2
//   }
// }

// Random
// "filterSettings": {
//     "filter": "color",
//     "move_value_type": 2,
//     "setting_float": 0.0,
//     "setting_float_max": 0.04,
//     "setting_name": "contrast",
//     "value_type": 2
//   }
//

// Add
// "filterSettings": {
//     "filter": "color",
//     "move_value_type": 2,
//     "setting_float": 0.0,
//     "setting_float_max": 0.04,
//     "setting_name": "contrast",
//     "value_type": 2
//   }

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
enum MoveValueType {
    SingleSetting = 0,
    Settings = 1,
    Random = 2,
    Add = 3,
    Typing = 4,
}

fn single_setting<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_i32(MoveValueType::SingleSetting as i32)
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

    // Move brightness
    // {
    //   "filterEnabled": false,
    //   "filterIndex": 2,
    //   "filterKind": "move_value_filter",
    //   "filterName": "",
    //   "filterSettings": {
    //     "filter": "color",
    //     "move_value_type": 0,
    //     "setting_color": 16777215,
    //     "setting_float": 1.0,
    //     "setting_float_max": 0.04,
    //     "setting_name": "brightness",
    //     "value_type": 2
    //   }
    // }

    #[tokio::test]
    async fn test_fun() {
        let source = "test-source";
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
        let _ = move_transition::update_and_trigger_3d_filter(
            &obs_client,
            source,
            filter_name,
            settings,
            duration_settings,
        );

        // We won't need anything except Defaults

        // pub async fn update_and_trigger_3d_filter<
        //     T: serde::Serialize + std::default::Default,
        // >(
        //     obs_client: &OBSClient,
        //     source: &str,
        //     filter_name: &str,
        //     settings: T,
        //     duration_settings: models::DurationSettings,
        // ) -> Result<()> {

        // How can we modify?

        // Move Value
        // {
        //   "filterEnabled": false,
        //   "filterIndex": 2,
        //   "filterKind": "move_value_filter",
        //   "filterName": "",
        //   "filterSettings": {
        //     "filter": "color",
        //     "move_value_type": 0,
        //     "setting_color": 16777215,
        //     "setting_float": 1.0,
        //     "setting_float_max": 0.04,
        //     "setting_name": "brightness",
        //     "value_type": 2
        //   }
        // }

        // let obj = SingleSetting {
        //     filter: "test".to_string(),
        //     duration: EasingDuration::new(3000),
        //     ..Default::default()
        // };
        // let res = ::serde_json::to_string_pretty(&obj).unwrap();
        // println!("\nMove Single Value\n{}", res);

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
}
