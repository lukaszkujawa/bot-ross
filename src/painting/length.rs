use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Length {
    Range([f32; 2]),
    Full(Full),
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Full {
    Full,
}

impl Length {
    pub fn range(self) -> Option<[f32; 2]> {
        match self {
            Self::Range(range) => Some(range),
            Self::Full(_) => None,
        }
    }

    pub fn longest(self, [width, height]: [f32; 2]) -> f32 {
        self.range().map_or(width.hypot(height), |[_, longest]| longest)
    }
}
