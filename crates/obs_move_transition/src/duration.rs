use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::str::FromStr;
use strum::EnumString;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct EasingDuration {
    pub duration: Option<i32>,

    #[serde(rename = "easing_function_match")]
    pub easing_function: EasingFunction,

    #[serde(rename = "easing_match")]
    pub easing_type: EasingType,
}

impl EasingDuration {
    pub fn builder() -> EasingDurationBuilder {
        EasingDurationBuilder {
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct EasingDurationBuilder {
    pub duration: Option<i32>,
    pub easing_function: Option<EasingFunction>,
    pub easing_type: Option<EasingType>,
}

impl EasingDurationBuilder {
    pub fn build(&self) -> EasingDuration {
        let ef = self.easing_function.unwrap_or(EasingFunction::Quadratic);
        let et = self.easing_type.unwrap_or(EasingType::NoEasing);
        EasingDuration {
            duration: self.duration,
            easing_function: ef,
            easing_type: et,
            ..Default::default()
        }
    }

    pub fn duration(mut self, duration: i32) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn easing_function(mut self, easing_function: EasingFunction) -> Self {
        self.easing_function = Some(easing_function);
        self
    }

    pub fn easing_type(mut self, easing_type: EasingType) -> Self {
        self.easing_type = Some(easing_type);
        self
    }
}

impl EasingDuration {
    pub fn new(duration: i32) -> EasingDuration {
        return EasingDuration {
            duration: Some(duration),
            ..Default::default()
        };
    }
}

#[derive(
    Serialize_repr,
    Deserialize_repr,
    PartialEq,
    Debug,
    Default,
    EnumString,
    Copy,
    Clone,
)]
#[repr(u8)]
pub enum EasingFunction {
    #[default]
    Quadratic = 1,
    Cubic = 2,
    Quartic = 3,
    Quintic = 4,
    Sine = 5,
    Circular = 6,
    Expotential = 7,
    Elastic = 8,
    Bounce = 9,
    Back = 10,
}

#[derive(
    Serialize_repr,
    Deserialize_repr,
    PartialEq,
    Debug,
    Default,
    EnumString,
    Copy,
    Clone,
)]
#[repr(u8)]
pub enum EasingType {
    NoEasing = 0,
    EaseIn = 1,
    EaseOut = 2,
    #[default]
    EaseInAndOut = 3,
}

pub fn find_easing_indicies(
    easing_function: impl Into<String>,
    easing_type: impl Into<String>,
) -> (i32, i32) {
    let ef =
        EasingFunction::from_str(&easing_function.into().to_case(Case::Pascal))
            .unwrap_or(EasingFunction::Quadratic) as i32;
    let et = EasingType::from_str(&easing_type.into().to_case(Case::Pascal))
        .unwrap_or(EasingType::NoEasing) as i32;
    (ef, et)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fun2() {
        let res = find_easing_indicies("cubic", "ease-in");
        assert_eq!(res, (2, 1));
    }
}
