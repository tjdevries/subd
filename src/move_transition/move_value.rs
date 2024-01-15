use crate::move_transition::duration::EasingDuration;
use serde::{Deserialize, Serialize, Serializer};
use serde_repr::*;

// How do I make sure each of these have a default?
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SingleSetting {
    pub filter: String,

    #[serde(serialize_with = "single_setting")]
    move_value_type: (),

    #[serde(flatten)]
    pub duration: EasingDuration,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Settings<T> {
    pub filter: String,

    #[serde(serialize_with = "settings")]
    pub move_value_type: (),

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

    #[serde(flatten)]
    pub duration: EasingDuration,
}

// This is like a single settings
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Random {
    pub filter: String,

    #[serde(serialize_with = "random")]
    move_value_type: (),

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
    }
}
