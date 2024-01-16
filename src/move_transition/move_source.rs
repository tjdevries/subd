use crate::move_transition::duration;
use crate::move_transition::models;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MoveSource {
    pub filter: String,
    pub source: String,

    #[serde(flatten)]
    pub settings: MoveSourceSettings,

    #[serde(flatten)]
    pub duration: duration::EasingDuration,
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

#[derive(Deserialize, Debug, Default)]
pub struct MoveSourceSettings {
    pub bounds: Option<models::Coordinates>,
    pub scale: Option<models::Coordinates>,
    pub crop: Option<CropSettings>,
    pub position: Option<models::Coordinates>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub rot: Option<f32>,

    #[serde(skip)]
    relative_transform: bool,
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
        MoveSourceSettings {
            bounds: self.bounds,
            scale: self.scale,
            crop: self.crop,
            position: self.position,
            x: self.x,
            y: self.y,
            rot: self.rot,
            relative_transform: self.relative_transform.unwrap_or_default(),
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

#[derive(Default)]
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

impl Serialize for MoveSourceSettings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MoveSource", 8)?;
        state.serialize_field("bounds", &self.bounds)?;
        state.serialize_field("scale", &self.scale)?;
        state.serialize_field("crop", &self.crop)?;
        state.serialize_field("position", &self.position)?;
        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;
        state.serialize_field("rot", &self.rot)?;

        let transform_text = build_transform_text(self);
        state.serialize_field("transform_text", &transform_text)?;

        state.end()
    }
}

fn build_transform_text(s: &MoveSourceSettings) -> String {
    let default_coor = models::Coordinates::default();
    let default_crop = CropSettings::default();

    let x = s.position.unwrap_or(default_coor).x.unwrap_or(0.0);
    let y = s.position.unwrap_or(default_coor).y.unwrap_or(0.0);
    let rot = s.rot.unwrap_or(0.0);
    let scale_x = s.scale.unwrap_or(default_coor).x.unwrap_or(1.0);
    let scale_y = s.scale.unwrap_or(default_coor).y.unwrap_or(1.0);
    let bounds_x = s.bounds.unwrap_or(default_coor).x.unwrap_or(0.0);
    let bounds_y = s.bounds.unwrap_or(default_coor).y.unwrap_or(0.0);
    let crop_left = s.crop.unwrap_or(default_crop).left.unwrap_or(0.0);
    let crop_right = s.crop.unwrap_or(default_crop).right.unwrap_or(0.0);
    let crop_top = s.crop.unwrap_or(default_crop).top.unwrap_or(0.0);
    let crop_bottom = s.crop.unwrap_or(default_crop).bottom.unwrap_or(0.0);

    if s.relative_transform {
        format!(
            "pos: x {} y {} rot: {} bounds: x {} y {} scale: x {} y {} crop: l {} t {} r {} b {}",
            x, y, rot, bounds_x, bounds_y, scale_x, scale_y, crop_left, crop_top, crop_right, crop_bottom
        )
    } else {
        format!(
            "pos: x+{} y+{} rot:+{} scale: x*{} y*{} crop: l {} t {} r {} b {}",
            x,
            y,
            rot,
            scale_x,
            scale_y,
            crop_left,
            crop_top,
            crop_right,
            crop_bottom
        )
    }
}

impl CropSettings {
    pub fn new() -> Self {
        Self {
            left: None,
            top: None,
            right: None,
            bottom: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct CropSettings {
    left: Option<f32>,
    top: Option<f32>,
    right: Option<f32>,
    bottom: Option<f32>,
}
