use serde::{Deserialize, Serialize, Serializer};
use std::fmt;

// MoveSource
// MoveValue
//      - Single
//      - Multiple
//      - Add
//      - Random
//      - Typing
// MoveAction

// This should have a default?
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MovePluginSettings<T> {
    pub filter: String,

    // These should be Enums
    #[serde(serialize_with = "value_type")]
    pub value_type: (),

    #[serde(serialize_with = "move_value_type")]
    pub move_value_type: (),

    #[serde(flatten)]
    pub settings: T,

    #[serde(flatten)]
    pub duration: DurationSettings,
}

// This is wrong now
fn value_type<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_i32(2)
}

fn move_value_type<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_i32(1)
}

// TODO: We need to add defaults
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct DurationSettings {
    pub duration: Option<i32>,
    pub easing_function_index: Option<i32>,
    pub easing_type_index: Option<i32>,

    // We don't need these Strings
    pub easing_type: Option<String>,
    pub easing_function: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Coordinates {
    pub x: Option<f32>,
    pub y: Option<f32>,
}

enum FilterKind {
    MoveAction,
    MoveValue,
    MoveSource,
}

impl fmt::Display for FilterKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FilterKind::MoveAction => write!(f, "move_action_filter"),
            FilterKind::MoveValue => write!(f, "move_value_filter"),
            FilterKind::MoveSource => write!(f, "move_source_filter"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_move_transition_filters() {
        let obs_client = crate::obs::obs::create_obs_client().await.unwrap();
        let filter_details = obs_client
            .filters()
            .get("test-source", "move-value")
            .await
            .unwrap();
        let res = ::serde_json::to_string_pretty(&filter_details).unwrap();
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
