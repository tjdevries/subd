use serde::{Deserialize, Serialize};
use serde_repr::*;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct EasingDuration {
    pub duration: Option<i32>,
    pub easing_function: EasingFunction,
    pub easing_type: EasingType,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Default)]
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

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Default)]
#[repr(u8)]
pub enum EasingType {
    #[default]
    NoEasing = 0,
    EasingIn = 1,
    EasingOut = 2,
    EasingInAndOut = 3,
}

impl EasingDuration {
    pub fn new(duration: i32) -> EasingDuration {
        return EasingDuration {
            duration: Some(duration),
            ..Default::default()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_duration() {
        let obj = EasingDuration::new(3000);
        let res = serde_json::to_string(&obj);
        println!("{:?}", res);
        // Get a new object serial/deserial
        // let obs_client = crate::obs::obs::create_obs_client().await.unwrap();
        // let filter_details =
        //     obs_client.filters().get("test-source", "move-value").await.unwrap();
        // let res = ::serde_json::to_string_pretty(&filter_details).unwrap();
        // println!("\nMove Value\n{}", res);

        // let filter_details =
        //     obs_client.filters().get("test-source", "move-action").await.unwrap();
        // let res = ::serde_json::to_string_pretty(&filter_details).unwrap();
        // println!("\nMove Action\n{}", res);
        //
        // let filter_details =
        //     obs_client.filters().get("test-scene", "move-source").await.unwrap();
        // let res = ::serde_json::to_string_pretty(&filter_details).unwrap();
        // println!("\nMove Source\n{}", res);
        // let x = MoveValueType::Settings as i32;
        // println!("X: {}", x);
    }
}
