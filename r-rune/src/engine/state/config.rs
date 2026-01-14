use serde::Deserialize;
use super::{FirePolicy, Region};

use crate::engine::config::PhysicsConfig;

#[derive(Clone, Debug, Deserialize)]
pub struct Keybinds {
    pub forward: String,
    pub backward: String,
    pub left: String,
    pub right: String,
    pub jump: String,
    pub sprint: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PlayerConfig {
    pub physics: PhysicsConfig,
    pub keybinds: Keybinds,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModelConfig {
    pub path: String,
    pub offset: [f32; 3],
    pub scale: f32,
    pub material_id: f32,
    #[serde(default = "default_true")]
    pub collision: bool,
    #[serde(default = "default_collider_type")]
    pub collider_type: String,
    #[serde(default)]
    pub rigid_body: bool,
    #[serde(default = "default_true")]
    pub pushable: bool,
    #[serde(default)]
    pub interactable: Option<String>,  
    #[serde(default)]
    pub firearm: bool,
    #[serde(default)]
    pub bullet: bool,
    #[serde(default = "default_one")]
    pub opacity: f32,
    #[serde(default)]
    pub material: Option<MaterialProps>,
    #[serde(default)]
    pub texture: Option<String>,  // Path to texture file
}

#[derive(Clone, Debug, Deserialize)]
pub struct MaterialProps {
    pub color: [f32; 3],
    pub metalness: f32,
    pub roughness: f32,
}


#[derive(Clone, Debug, Deserialize)]
pub struct AIConfig {
    pub id: String,
    #[serde(rename = "type")]
    pub ai_type: String,
    pub position: [f32; 3],
    pub behavior: String,
    #[serde(default)]
    pub asset: Option<String>,
    #[serde(default)]
    pub collision: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TriggerConfig {
    pub id: String,
    pub region: Region,
    pub fire: FirePolicy,
    pub actions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuiElementConfig {
    pub id: String,
    pub text: String,
    pub position: [f32; 2],
    pub scale: f32,
    pub action: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CrosshairConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_crosshair_style")]
    pub style: String,
    #[serde(default = "default_one")]
    pub scale: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LightConfig {
    pub position: [f32; 3],
    #[serde(default = "default_light_radius")]
    pub radius: f32,
    pub color: [f32; 3],
    #[serde(default = "default_one")]
    pub intensity: f32,
    #[serde(default)]
    pub follow_model: Option<String>,
}

fn default_light_radius() -> f32 {
    10.0
}

#[derive(Clone, Debug, Deserialize)]
pub struct Scene {
    pub models: Vec<ModelConfig>,
    pub ai: Vec<AIConfig>,
    pub triggers: Vec<TriggerConfig>,
    #[serde(default)]
    pub gui: Vec<GuiElementConfig>,
    #[serde(default)]
    pub wind: Option<WindConfig>,
    #[serde(default)]
    pub lights: Vec<LightConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GameStateConfig {
    #[serde(default = "default_time")]
    pub initial_time: f32,
    #[serde(default = "default_day_cycle")]
    pub day_cycle_duration: f32,
    #[serde(default)]
    pub crosshair: CrosshairConfig,
    pub player_config: PlayerConfig,
    pub default_scene: String,
    #[serde(default)]
    pub clouds: CloudConfig,
    #[serde(default)]
    pub wind: WindConfig,
    #[serde(default)]
    pub rain: RainConfig,
    #[serde(default)]
    pub snow: SnowConfig,
    #[serde(default)]
    pub enable_rain: bool,
    #[serde(default)]
    pub enable_snow: bool,
    #[serde(default)]
    pub lighting: crate::engine::config::LightingConfig,
    #[serde(default = "default_scene_dir")]
    pub scene_dir: String,
    #[serde(default)]
    pub weapon_sway: WeaponSwayConfig,
}

impl GameStateConfig {
    pub fn load(path: &str) -> Self {
        let data = std::fs::read_to_string(path).expect("Failed to read config file");
        ron::from_str(&data).expect("Failed to parse config file")
    }
}

impl Scene {
    pub fn load(path: &str) -> Self {
        let data = std::fs::read_to_string(path).expect("Failed to read scene file");
        ron::from_str(&data).expect("Failed to parse scene file")
    }
}

fn default_scene_dir() -> String {
    "assets/scenes".to_string()
}

fn default_time() -> f32 {
    12.0
}

fn default_day_cycle() -> f32 {
    120.0
}

fn default_true() -> bool {
    true
}

fn default_collider_type() -> String {
    "solid".to_string()
}

fn default_one() -> f32 {
    1.0
}

fn default_crosshair_style() -> String {
    "dot".to_string()
}

impl Default for CrosshairConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            style: default_crosshair_style(),
            scale: default_one(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct CloudConfig {
    #[serde(default = "default_cloud_density")]
    pub density: f32,
    #[serde(default = "default_cloud_scale")]
    pub scale: f32,
    #[serde(default = "default_cloud_min_height")]
    pub min_height: f32,
    #[serde(default = "default_cloud_max_height")]
    pub max_height: f32,
    #[serde(default = "default_cloud_absorption")]
    pub absorption: f32,
    #[serde(default = "default_cloud_wind_scale")]
    pub wind_scale: f32,
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            density: default_cloud_density(),
            scale: default_cloud_scale(),
            min_height: default_cloud_min_height(),
            max_height: default_cloud_max_height(),
            absorption: default_cloud_absorption(),
            wind_scale: default_cloud_wind_scale(),
        }
    }
}

fn default_cloud_density() -> f32 { 0.5 }
fn default_cloud_scale() -> f32 { 0.01 }
fn default_cloud_min_height() -> f32 { 150.0 }
fn default_cloud_max_height() -> f32 { 450.0 }
fn default_cloud_absorption() -> f32 { 0.5 }
fn default_cloud_wind_scale() -> f32 { 3.0 }

#[derive(Clone, Debug, Deserialize)]
pub struct RainConfig {
    #[serde(default = "default_rain_intensity")]
    pub intensity: f32,
    #[serde(default = "default_rain_drop_count")]
    pub drop_count: u32,
    #[serde(default = "default_rain_spawn_radius")]
    pub spawn_radius: f32,
    #[serde(default = "default_rain_spawn_height")]
    pub spawn_height: f32,
    #[serde(default = "default_rain_ground_height")]
    pub ground_height: f32,
    #[serde(default = "default_rain_fall_speed")]
    pub fall_speed: f32,
    #[serde(default = "default_rain_drop_length")]
    pub drop_length: f32,
    #[serde(default = "default_rain_drop_width")]
    pub drop_width: f32,
    #[serde(default = "default_rain_wind_scale")]
    pub wind_scale: f32,
}

impl Default for RainConfig {
    fn default() -> Self {
        Self {
            intensity: default_rain_intensity(),
            drop_count: default_rain_drop_count(),
            spawn_radius: default_rain_spawn_radius(),
            spawn_height: default_rain_spawn_height(),
            ground_height: default_rain_ground_height(),
            fall_speed: default_rain_fall_speed(),
            drop_length: default_rain_drop_length(),
            drop_width: default_rain_drop_width(),
            wind_scale: default_rain_wind_scale(),
        }
    }
}

fn default_rain_intensity() -> f32 { 0.8 }
fn default_rain_drop_count() -> u32 { 20000 }
fn default_rain_spawn_radius() -> f32 { 35.0 }
fn default_rain_spawn_height() -> f32 { 25.0 }
fn default_rain_ground_height() -> f32 { 0.0 }
fn default_rain_fall_speed() -> f32 { 30.0 }
fn default_rain_drop_length() -> f32 { 0.9 }
fn default_rain_drop_width() -> f32 { 0.03 }
fn default_rain_wind_scale() -> f32 { 1.0 }


#[derive(Clone, Debug, Deserialize)]
pub struct SnowConfig {
    #[serde(default = "default_snow_intensity")]
    pub intensity: f32,
    #[serde(default = "default_snow_flake_count")]
    pub flake_count: u32,
    #[serde(default = "default_snow_spawn_radius")]
    pub spawn_radius: f32,
    #[serde(default = "default_snow_spawn_height")]
    pub spawn_height: f32,
    #[serde(default = "default_snow_ground_height")]
    pub ground_height: f32,
    #[serde(default = "default_snow_fall_speed")]
    pub fall_speed: f32,
    #[serde(default = "default_snow_flake_size")]
    pub flake_size: f32,
    #[serde(default = "default_snow_wind_scale")]
    pub wind_scale: f32,
    #[serde(default = "default_snow_turbulence")]
    pub turbulence: f32,
}

impl Default for SnowConfig {
    fn default() -> Self {
        Self {
            intensity: default_snow_intensity(),
            flake_count: default_snow_flake_count(),
            spawn_radius: default_snow_spawn_radius(),
            spawn_height: default_snow_spawn_height(),
            ground_height: default_snow_ground_height(),
            fall_speed: default_snow_fall_speed(),
            flake_size: default_snow_flake_size(),
            wind_scale: default_snow_wind_scale(),
            turbulence: default_snow_turbulence(),
        }
    }
}

fn default_snow_intensity() -> f32 { 0.8 }
fn default_snow_flake_count() -> u32 { 50000 }
fn default_snow_spawn_radius() -> f32 { 35.0 }
fn default_snow_spawn_height() -> f32 { 25.0 }
fn default_snow_ground_height() -> f32 { 0.0 }
fn default_snow_fall_speed() -> f32 { 1.5 }  
fn default_snow_flake_size() -> f32 { 0.05 }
fn default_snow_wind_scale() -> f32 { 0.5 }
fn default_snow_turbulence() -> f32 { 1.0 }

#[derive(Clone, Debug, Deserialize)]
pub struct WindConfig {
    #[serde(default = "default_wind_dir")]
    pub direction: [f32; 2],
    #[serde(default = "default_wind_speed")]
    pub speed: f32,
    #[serde(default = "default_wind_turbulence")]
    pub turbulence: f32,
}

impl Default for WindConfig {
    fn default() -> Self {
        Self {
            direction: default_wind_dir(),
            speed: default_wind_speed(),
            turbulence: default_wind_turbulence(),
        }
    }
}

fn default_wind_dir() -> [f32; 2] { [0.6, 0.4] }
fn default_wind_speed() -> f32 { 4.0 }
fn default_wind_turbulence() -> f32 { 0.35 }

#[derive(Clone, Debug, Deserialize)]
pub struct WeaponSwayConfig {
    #[serde(default = "default_camera_sway_scale")]
    pub camera_sway_scale: f32,
    #[serde(default = "default_movement_sway_scale")]
    pub movement_sway_scale: f32,
    #[serde(default = "default_sway_speed")]
    pub sway_speed: f32,
    #[serde(default = "default_sway_limit")]
    pub sway_limit: f32,
}

impl Default for WeaponSwayConfig {
    fn default() -> Self {
        Self {
            camera_sway_scale: default_camera_sway_scale(),
            movement_sway_scale: default_movement_sway_scale(),
            sway_speed: default_sway_speed(),
            sway_limit: default_sway_limit(),
        }
    }
}

fn default_camera_sway_scale() -> f32 { 0.002 }
fn default_movement_sway_scale() -> f32 { 0.005 }
fn default_sway_speed() -> f32 { 10.0 }
fn default_sway_limit() -> f32 { 0.1 }
