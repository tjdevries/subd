use serde::{Deserialize, Serialize, Serializer};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Coordinates {
    pub x: Option<f32>,
    pub y: Option<f32>,
}

// I think the easing will want to be an Enum
// TODO: Finish this
// Then we need to add defaults
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct DurationSettings {
    pub duration: Option<i32>,
    pub easing_function_index: Option<i32>,
    pub easing_type_index: Option<i32>,
    pub easing_type: Option<String>,
    pub easing_function: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MovePluginSettings<T> {
    pub filter: String,

    #[serde(serialize_with = "value_type")]
    pub value_type: (),

    #[serde(serialize_with = "move_value_type")]
    pub move_value_type: (),

    // #[serde(
    //     rename = "easing_function_match",
    //     skip_serializing_if = "Option::is_none"
    // )]
    // pub easing_function: Option<i32>,
    // #[serde(rename = "easing_match", skip_serializing_if = "Option::is_none")]
    // pub easing_type: Option<i32>,
    // #[serde(rename = "duration", skip_serializing_if = "Option::is_none")]
    // pub duration: Option<i32>,
    #[serde(flatten)]
    pub settings: T,

    #[serde(flatten)]
    pub duration: DurationSettings,
}

fn value_type<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_i32(2)
}

fn move_value_type<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_i32(1)
}
