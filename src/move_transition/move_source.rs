use crate::move_transition::duration;
use crate::move_transition::models;
use core::fmt;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::Display;

use super::models::Coordinates;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MoveSource {
    pub filter: String,

    pub source: String,

    #[serde(flatten)]
    pub settings: MoveSourceSettings,

    #[serde(flatten)]
    pub duration: duration::EasingDuration,
}

#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
#[serde(untagged)]
pub enum Sign {
    #[default]
    Nothing,
    Positive,
    Negative,
    Multiply,
    Divide,
}

impl Display for Sign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Sign::Nothing => " ",
            Sign::Positive => "+",
            Sign::Negative => "-",
            Sign::Multiply => "*",
            Sign::Divide => "/",
        };
        write!(f, "{}", s)
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

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MoveSourceSettings {
    pub bounds: models::Coordinates,
    pub scale: models::Coordinates,
    pub crop: CropSettings,

    #[serde(rename = "pos")]
    pub position: models::Coordinates,

    pub rot: f32,
    #[serde(
        serialize_with = "crate::move_transition::models::sign_serializer"
    )]
    pub rot_sign: Sign,
}

impl MoveSourceSettings {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn builder() -> MoveSourceSettingsBuilder {
        MoveSourceSettingsBuilder::default()
    }
}

impl MoveSourceSettingsBuilder {
    pub fn new() -> MoveSourceSettingsBuilder {
        MoveSourceSettingsBuilder::default()
    }

    pub fn build(self) -> MoveSourceSettings {
        let (pos_sign, scale_sign) =
            if self.relative_transform.unwrap_or_default() {
                (Sign::Positive, Sign::Multiply)
            } else {
                (Sign::Nothing, Sign::Nothing)
            };

        let bounds = self.bounds.unwrap_or_default().with_signs(pos_sign);
        MoveSourceSettings {
            bounds: self.bounds.unwrap_or_default().with_signs(pos_sign),
            scale: self
                .scale
                .unwrap_or(Coordinates::new(Some(1.0), Some(1.0)))
                .with_signs(scale_sign),
            position: self.position.unwrap_or_default().with_signs(pos_sign),
            crop: self.crop.unwrap_or_default(),
            rot_sign: pos_sign,
            rot: self.rot.unwrap_or_default(),
        }
    }

    pub fn bounds(
        mut self,
        bounds: models::Coordinates,
    ) -> MoveSourceSettingsBuilder {
        self.bounds = Some(bounds);
        self
    }

    pub fn position(
        mut self,
        position: models::Coordinates,
    ) -> MoveSourceSettingsBuilder {
        self.position = Some(position);
        self
    }

    pub fn scale(
        mut self,
        scale: models::Coordinates,
    ) -> MoveSourceSettingsBuilder {
        self.scale = Some(scale);
        self
    }

    pub fn crop(mut self, crop: CropSettings) -> MoveSourceSettingsBuilder {
        self.crop = Some(crop);
        self
    }

    pub fn x(mut self, x: f32) -> MoveSourceSettingsBuilder {
        self.x = Some(x);
        self
    }

    pub fn y(mut self, y: f32) -> MoveSourceSettingsBuilder {
        self.y = Some(y);
        self
    }

    pub fn rot(mut self, rot: f32) -> MoveSourceSettingsBuilder {
        self.rot = Some(rot);
        self
    }

    pub fn relative_transform(
        mut self,
        relative_transform: bool,
    ) -> MoveSourceSettingsBuilder {
        self.relative_transform = Some(relative_transform);
        self
    }
}

#[derive(Default, Debug)]
pub struct MoveSourceSettingsBuilder {
    pub bounds: Option<models::Coordinates>,
    pub scale: Option<models::Coordinates>,
    pub crop: Option<CropSettings>,
    pub position: Option<models::Coordinates>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub rot: Option<f32>,
    pub relative_transform: Option<bool>,
}

impl CropSettings {
    pub fn new() -> Self {
        Self {
            left: None,
            top: None,
            right: None,
            bottom: None,
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct CropSettings {
    left: Option<f32>,
    #[serde(
        serialize_with = "crate::move_transition::models::sign_serializer"
    )]
    left_sign: Sign,

    top: Option<f32>,
    #[serde(
        serialize_with = "crate::move_transition::models::sign_serializer"
    )]
    top_sign: Sign,

    right: Option<f32>,
    #[serde(
        serialize_with = "crate::move_transition::models::sign_serializer"
    )]
    right_sign: Sign,

    bottom: Option<f32>,
    #[serde(
        serialize_with = "crate::move_transition::models::sign_serializer"
    )]
    bottom_sign: Sign,
}
