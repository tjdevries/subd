use crate::move_transition::duration::EasingDuration;
use serde::{Deserialize, Serialize, Serializer};
use serde_repr::*;

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

// How do I make sure each of these have a default?
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SingleSetting {
    pub filter: String,

    // This could always be the value
    #[serde(serialize_with = "single_setting")]
    move_value_type: (),

    // I don't want it to be public
    // pub move_value_type: MoveValueType::SingleSetting,

    // We need to handle these
    // // These should be Enums
    // #[serde(serialize_with = "value_type")]
    // pub value_type: (),
    // #[serde(serialize_with = "move_value_type")]
    #[serde(flatten)]
    pub duration: EasingDuration,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Settings<T> {
    pub filter: String,

    // We need to handle these
    // // These should be Enums
    // #[serde(serialize_with = "value_type")]
    // pub value_type: (),
    // #[serde(serialize_with = "move_value_type")]
    // pub move_value_type: (),
    #[serde(flatten)]
    pub settings: T,

    #[serde(flatten)]
    pub duration: EasingDuration,
}

// This is like a single settings
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Add {
    pub filter: String,

    // We need to handle these
    // // These should be Enums
    // #[serde(serialize_with = "value_type")]
    // pub value_type: (),
    // #[serde(serialize_with = "move_value_type")]
    // pub move_value_type: (),

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

    // We need to handle these
    // // These should be Enums
    // #[serde(serialize_with = "value_type")]
    // pub value_type: (),
    // #[serde(serialize_with = "move_value_type")]
    // pub move_value_type: (),

    // This is like the single Settings
    // #[serde(flatten)]
    // pub settings: T,
    //
    // Takes a min and max value
    #[serde(flatten)]
    pub duration: EasingDuration,
}

// Has to be on a typing source
// This is like a single settings
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Typing {
    pub filter: String,

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fun() {
        let obs_client = crate::obs::obs::create_obs_client().await.unwrap();
        let filter_details = obs_client
            .filters()
            .get("test-source", "move-value-single")
            .await
            .unwrap();
        let res = ::serde_json::to_string_pretty(&filter_details).unwrap();
        println!("\nMove Value\n{}", res);

        let obj = SingleSetting {
            filter: "test".to_string(),
            duration: EasingDuration::new(3000),
            ..Default::default()
        };
        let res = ::serde_json::to_string_pretty(&obj).unwrap();
        println!("\nMove Value\n{}", res);

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
