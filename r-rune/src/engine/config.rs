use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct PhysicsConfig {
    pub gravity: f32,
    pub base_speed: f32,
    pub sprint_multiplier: f32,
    pub air_control: f32,
    pub ground_accel: f32,
    pub friction: f32,
    pub jump_velocity: f32,
    #[serde(default = "default_capsule_radius")]
    pub capsule_radius: f32,
    #[serde(default = "default_capsule_height")]
    pub capsule_height: f32,
}

fn default_capsule_radius() -> f32 { 0.3 }
fn default_capsule_height() -> f32 { 1.7 }

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: -9.81,
            base_speed: 3.8,
            sprint_multiplier: 1.7,
            air_control: 4.0,
            ground_accel: 18.0,
            friction: 8.0,
            jump_velocity: 6.5,
            capsule_radius: default_capsule_radius(),
            capsule_height: default_capsule_height(),
        }
    }
}

impl PhysicsConfig {
     
}

#[derive(Clone, Debug, Deserialize)]
pub struct DirectionalLight {
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct ShadowSettings {
    pub enabled: bool,
    pub strength: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LightingConfig {
    #[serde(default)]
    pub directional: Vec<DirectionalLight>,
    #[serde(default)]
    pub shadows: ShadowSettings,
}

impl Default for LightingConfig {
    fn default() -> Self {
        Self {
            directional: vec![DirectionalLight {
                direction: [-0.545, 0.818, 0.182],
                color: [1.0, 0.97, 0.9],
                intensity: 40.0,
            }],
            shadows: ShadowSettings {
                enabled: false,
                strength: 0.6,
            },
        }
    }
}

impl LightingConfig {
     
}
