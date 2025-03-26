pub mod corner_pin;
pub mod orthographic;
pub mod perspective;

use serde::{Deserialize, Serialize};
use serde_repr::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum SpinFilters {
    Orthographic(orthographic::ThreeDTransformOrthographic),
    Perspective(perspective::ThreeDTransformPerspective),
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Default)]
#[repr(u8)]
pub enum CameraMode {
    #[default]
    Perspective = 0,
    Orthographic = 1,
    CornerPin = 2,
}

// This still feels wrong
// I think there is a more idiomatic Rust to do this
pub trait FilterName {
    fn filter_name(&self) -> String;
}
