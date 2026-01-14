use std::collections::HashSet;

pub mod config;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub enum Region {
    Sphere { center: [f32; 3], radius: f32 },
    Aabb { min: [f32; 3], max: [f32; 3] },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FirePolicy {
    Once,
    Repeat,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Trigger {
    pub id: String,
    pub region: Region,
    #[serde(default = "default_fire")]
    pub fire: FirePolicy,
    #[serde(default)]
    pub actions: Vec<String>,
}

fn default_fire() -> FirePolicy {
    FirePolicy::Once
}

#[derive(Default)]
pub struct GameState {
    pub flags: HashSet<String>,
    pub fired: HashSet<String>,
}

impl Region {
    pub fn contains(&self, point: glam::Vec3) -> bool {
        match self {
            Region::Sphere { center, radius } => {
                let c = glam::Vec3::from(*center);
                (point - c).length_squared() <= radius * radius
            }
            Region::Aabb { min, max } => {
                point.x >= min[0]
                    && point.x <= max[0]
                    && point.y >= min[1]
                    && point.y <= max[1]
                    && point.z >= min[2]
                    && point.z <= max[2]
            }
        }
    }
}

pub fn load_triggers(path: &str) -> Vec<Trigger> {
    let data = std::fs::read_to_string(path).expect("failed to read triggers file");
    ron::from_str(&data).expect("failed to parse triggers ron")
}
