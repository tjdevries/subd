use core::fmt;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::duration::EasingDuration;
use crate::models::Coordinates;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MoveSource {
    pub filter: String,

    pub source: String,

    #[serde(flatten)]
    pub settings: MoveSourceSettings,

    #[serde(flatten)]
    pub duration: EasingDuration,
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
        duration: EasingDuration,
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
    pub bounds: Coordinates,
    pub scale: Coordinates,
    pub crop: CropSettings,

    #[serde(rename = "pos")]
    pub position: Coordinates,

    pub rot: f32,
    #[serde(serialize_with = "crate::models::sign_serializer")]
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
    pub fn new(source: String) -> MoveSourceSettingsBuilder {
        MoveSourceSettingsBuilder {
            source,
            ..Default::default()
        }
    }

    pub fn build(self) -> MoveSourceSettings {
        let (pos_sign, scale_sign) =
            if self.relative_transform.unwrap_or_default() {
                (Sign::Positive, Sign::Multiply)
            } else {
                (Sign::Nothing, Sign::Nothing)
            };

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

    pub fn bounds(mut self, bounds: Coordinates) -> MoveSourceSettingsBuilder {
        self.bounds = Some(bounds);
        self
    }

    pub fn position(
        mut self,
        position: Coordinates,
    ) -> MoveSourceSettingsBuilder {
        self.position = Some(position);
        self
    }

    pub fn scale(mut self, scale: Coordinates) -> MoveSourceSettingsBuilder {
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
    pub source: String,
    pub bounds: Option<Coordinates>,
    pub scale: Option<Coordinates>,
    pub crop: Option<CropSettings>,
    pub position: Option<Coordinates>,
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

    pub fn builder() -> CropSettingsBuilder {
        CropSettingsBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct CropSettingsBuilder {
    pub left: Option<f32>,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,

    pub left_sign: Option<Sign>,
    pub top_sign: Option<Sign>,
    pub right_sign: Option<Sign>,
    pub bottom_sign: Option<Sign>,
}

impl CropSettingsBuilder {
    pub fn build(self) -> CropSettings {
        CropSettings {
            left: self.left,
            top: self.top,
            right: self.right,
            bottom: self.bottom,
            left_sign: self.left_sign.unwrap_or(Sign::Nothing),
            top_sign: self.top_sign.unwrap_or(Sign::Nothing),
            right_sign: self.right_sign.unwrap_or(Sign::Nothing),
            bottom_sign: self.bottom_sign.unwrap_or(Sign::Nothing),
        }
    }

    pub fn left(mut self, left: f32) -> CropSettingsBuilder {
        self.left = Some(left);
        self
    }

    pub fn left_sign(mut self, left_sign: Sign) -> CropSettingsBuilder {
        self.left_sign = Some(left_sign);
        self
    }

    pub fn right(mut self, right: f32) -> CropSettingsBuilder {
        self.right = Some(right);
        self
    }

    pub fn right_sign(mut self, right_sign: Sign) -> CropSettingsBuilder {
        self.right_sign = Some(right_sign);
        self
    }

    pub fn top(mut self, top: f32) -> CropSettingsBuilder {
        self.top = Some(top);
        self
    }

    pub fn top_sign(mut self, top_sign: Sign) -> CropSettingsBuilder {
        self.top_sign = Some(top_sign);
        self
    }

    pub fn bottom(mut self, bottom: f32) -> CropSettingsBuilder {
        self.bottom = Some(bottom);
        self
    }

    pub fn bottom_sign(mut self, bottom_sign: Sign) -> CropSettingsBuilder {
        self.bottom_sign = Some(bottom_sign);
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct CropSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    left: Option<f32>,

    #[serde(serialize_with = "crate::models::sign_serializer")]
    left_sign: Sign,

    #[serde(skip_serializing_if = "Option::is_none")]
    top: Option<f32>,
    #[serde(serialize_with = "crate::models::sign_serializer")]
    top_sign: Sign,

    #[serde(skip_serializing_if = "Option::is_none")]
    right: Option<f32>,

    #[serde(serialize_with = "crate::models::sign_serializer")]
    right_sign: Sign,

    #[serde(skip_serializing_if = "Option::is_none")]
    bottom: Option<f32>,

    #[serde(serialize_with = "crate::models::sign_serializer")]
    bottom_sign: Sign,
}
