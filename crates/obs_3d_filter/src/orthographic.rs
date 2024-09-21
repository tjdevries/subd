use crate::CameraMode;
use crate::FilterName;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ThreeDTransformOrthographic {
    #[serde(rename = "Position.X", skip_serializing_if = "Option::is_none")]
    pub position_x: Option<f32>,

    #[serde(rename = "Position.Y", skip_serializing_if = "Option::is_none")]
    pub position_y: Option<f32>,

    #[serde(rename = "Position.Z", skip_serializing_if = "Option::is_none")]
    pub position_z: Option<f32>,

    #[serde(rename = "Rotation.X", skip_serializing_if = "Option::is_none")]
    pub rotation_x: Option<f32>,

    #[serde(rename = "Rotation.Y", skip_serializing_if = "Option::is_none")]
    pub rotation_y: Option<f32>,

    #[serde(rename = "Rotation.Z", skip_serializing_if = "Option::is_none")]
    pub rotation_z: Option<f32>,

    #[serde(rename = "Scale.X", skip_serializing_if = "Option::is_none")]
    pub scale_x: Option<f32>,

    #[serde(rename = "Scale.Y", skip_serializing_if = "Option::is_none")]
    pub scale_y: Option<f32>,

    #[serde(rename = "Shear.X", skip_serializing_if = "Option::is_none")]
    pub shear_x: Option<f32>,

    #[serde(rename = "Shear.Y", skip_serializing_if = "Option::is_none")]
    pub shear_y: Option<f32>,

    #[serde(rename = "Camera.Mode")]
    pub camera_mode: CameraMode,
}

impl FilterName for ThreeDTransformOrthographic {
    // This should come from some constant
    fn filter_name(&self) -> String {
        "3D-Transform-Orthographic".to_string()
    }
}

impl ThreeDTransformOrthographic {
    pub fn builder() -> ThreeDTransformOrthographicBuilder {
        ThreeDTransformOrthographicBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct ThreeDTransformOrthographicBuilder {
    pub position_x: Option<f32>,
    pub position_y: Option<f32>,
    pub position_z: Option<f32>,
    pub rotation_x: Option<f32>,
    pub rotation_y: Option<f32>,
    pub rotation_z: Option<f32>,
    pub scale_x: Option<f32>,
    pub scale_y: Option<f32>,
    pub shear_x: Option<f32>,
    pub shear_y: Option<f32>,
}

impl ThreeDTransformOrthographicBuilder {
    pub fn new() -> ThreeDTransformOrthographicBuilder {
        ThreeDTransformOrthographicBuilder::default()
    }

    pub fn build(self) -> ThreeDTransformOrthographic {
        ThreeDTransformOrthographic {
            position_x: self.position_x,
            position_y: self.position_y,
            position_z: self.position_z,
            rotation_x: self.rotation_x,
            rotation_y: self.rotation_y,
            rotation_z: self.rotation_z,
            scale_x: self.scale_x,
            scale_y: self.scale_y,
            shear_x: self.shear_x,
            shear_y: self.shear_y,
            camera_mode: CameraMode::Orthographic,
            ..Default::default()
        }
    }

    pub fn position_x(
        mut self,
        position_x: Option<f32>,
    ) -> ThreeDTransformOrthographicBuilder {
        self.position_x = position_x;
        self
    }

    pub fn position_y(
        mut self,
        position_y: Option<f32>,
    ) -> ThreeDTransformOrthographicBuilder {
        self.position_y = position_y;
        self
    }

    pub fn position_z(
        mut self,
        position_z: Option<f32>,
    ) -> ThreeDTransformOrthographicBuilder {
        self.position_z = position_z;
        self
    }

    pub fn rotation_x(
        mut self,
        rotation_x: Option<f32>,
    ) -> ThreeDTransformOrthographicBuilder {
        self.rotation_x = rotation_x;
        self
    }

    pub fn rotation_y(
        mut self,
        rotation_y: Option<f32>,
    ) -> ThreeDTransformOrthographicBuilder {
        self.rotation_y = rotation_y;
        self
    }

    pub fn rotation_z(
        mut self,
        rotation_z: Option<f32>,
    ) -> ThreeDTransformOrthographicBuilder {
        self.rotation_z = rotation_z;
        self
    }

    pub fn scale_x(
        mut self,
        scale_x: Option<f32>,
    ) -> ThreeDTransformOrthographicBuilder {
        self.scale_x = scale_x;
        self
    }

    pub fn scale_y(
        mut self,
        scale_y: Option<f32>,
    ) -> ThreeDTransformOrthographicBuilder {
        self.scale_y = scale_y;
        self
    }

    pub fn shear_x(
        mut self,
        shear_x: Option<f32>,
    ) -> ThreeDTransformOrthographicBuilder {
        self.shear_x = shear_x;
        self
    }

    pub fn shear_y(
        mut self,
        shear_y: Option<f32>,
    ) -> ThreeDTransformOrthographicBuilder {
        self.shear_y = shear_y;
        self
    }
}
