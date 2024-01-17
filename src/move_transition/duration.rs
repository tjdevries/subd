use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::str::FromStr;
use strum::EnumString;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct EasingDuration {
    pub duration: Option<i32>,
    pub easing_function: EasingFunction,
    pub easing_type: EasingType,
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
    Serialize_repr, Deserialize_repr, PartialEq, Debug, Default, EnumString,
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
    Serialize_repr, Deserialize_repr, PartialEq, Debug, Default, EnumString,
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
