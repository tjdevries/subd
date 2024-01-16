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

pub trait FilterName {
    fn filter_name(&self) -> String;
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Default)]
#[repr(u8)]
pub enum CameraMode {
    #[default]
    Perspective = 0,
    Orthographic = 1,
    CornerPin = 2,
}
