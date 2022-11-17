use serde::{Deserialize, Serialize};

// TODO: consider serde defaults???
// move into it's own SDF_Effects file???
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SDFEffectsSettings {
    #[serde(rename = "Filter.SDFEffects.Shadow.Inner.Alpha")]
    pub shadow_inner_alpha: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Outer.Alpha")]
    pub shadow_outer_alpha: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Glow.Outer")]
    pub glow_outer: Option<bool>,

    #[serde(rename = "Filter.SDFEffects.Glow.Outer.Alpha")]
    pub glow_outer_alpha: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Glow.Outer.Color")]
    pub outer_color: Option<u64>,

    #[serde(rename = "Filter.SDFEffects.Glow.Outer.Sharpness")]
    pub glow_outer_sharpness: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Glow.Outer.Width")]
    pub glow_outer_width: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Outer")]
    pub shadow_outer: Option<bool>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Outer.Color")]
    pub shadow_outer_color: Option<u64>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Inner.Color")]
    pub shadow_inner_color: Option<u64>,

    #[serde(rename = "Filter.SDFEffects.SDF.Scale")]
    pub scale: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.SDF.Threshold")]
    pub threshold: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Inner")]
    pub shadow_inner: Option<bool>,

    #[serde(rename = "Filter.SDFEffects.Glow.Inner.Alpha")]
    pub glow_inner_alpha: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Glow.Inner")]
    pub glow_inner: Option<bool>,

    #[serde(rename = "Filter.SDFEffects.Glow.Inner.Color")]
    pub inner_color: Option<u64>,

    #[serde(rename = "Filter.SDFEffects.Glow.Inner.Sharpness")]
    pub glow_inner_sharpness: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Glow.Inner.Width")]
    pub glow_inner_width: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Outline")]
    pub outline: Option<bool>,

    #[serde(rename = "Filter.SDFEffects.Outline.Color")]
    pub outline_color: Option<u64>,

    #[serde(rename = "Filter.SDFEffects.Outline.Width")]
    pub outline_width: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Outer.Range.Maximum")]
    pub shadow_outer_range_max: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Inner.Range.Maximum")]
    pub shadow_inner_range_max: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Inner.Range.Minimum")]
    pub shadow_inner_range_min: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Outer.Range.Minimum")]
    pub shadow_outer_range_min: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Inner.Offset.Y")]
    pub shadow_inner_offset_y: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Inner.Offset.X")]
    pub shadow_inner_offset_x: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Outer.Offset.Y")]
    pub shadow_outer_offset_y: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.SDF.Scale")]
    pub sdf_scale: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.SDF.Threshold")]
    pub sdf_threshold: Option<f32>,

    #[serde(rename = "Commit")]
    pub commit: Option<String>,

    #[serde(rename = "Version")]
    pub version: Option<u64>,
}
