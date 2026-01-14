use std::collections::HashSet;
use std::time::Instant;

use rayon::prelude::*;

use ash::vk;
use winit::{event_loop::EventLoop, window::Window};

use crate::vulkan::vk_meshlets::{Aabb, ColliderType, MeshletModelDesc};

pub const WIDTH: u32 = 1280;
pub const HEIGHT: u32 = 720;
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub mod environment;
pub mod gui;
pub mod config;
pub mod animation;
pub mod renderer;
pub mod ai;
pub mod physics;
pub mod player;
pub mod state;
pub mod sdf;
pub mod components;
pub mod asset_loader;
mod inventory;
mod overlay;
mod input;
pub mod firearm;
use components::*;
use firearm::FirearmSystem;

use environment::{
    sun_sphere::SunSphereRenderer,
    weather::{self, Weather},
    clouds::CloudRenderer,
    fire::FireRenderer,
    rain::RainRenderer,
    stars::StarsRenderer,
};
use sdf::SdfRenderer;
use gui::{
    menu::MenuRenderer,
    text::TextRenderer,
    hud::{HudRenderer, INV_COLS, INV_ROWS, INV_SLOT_GAP, INV_SLOT_SIZE},
    crosshair::CrosshairStyle,
};
use renderer::{FrameInput, Renderer, SkinnedActorDesc};
use renderer::lighting::{GpuLight, LightingSystem};
use config::{LightingConfig, PhysicsConfig};
use ai::{AIState, WalkTask};
use player::{Player, PlayerState};
use state::config::{GameStateConfig, GuiElementConfig, WindConfig};
use state::{GameState, Trigger};

#[derive(Default, Debug, Clone)]
pub struct InputState {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub sprint: bool,
    pub interact: bool,
    pub drop: bool,
    pub cursor_pos: [f32; 2],
    pub mouse_down: bool, 
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DragSource {
    Inventory(usize),
    Hotbar(usize),
}

#[derive(Clone, Copy, Debug)]
struct LightAttachment {
    model_idx: usize,
    relative_offset: glam::Vec3,
}



fn parse_crosshair_style(style: &str) -> CrosshairStyle {
    match style.to_ascii_lowercase().as_str() {
        "bars" | "cross" | "csgo" => CrosshairStyle::Bars,
        _ => CrosshairStyle::Dot,
    }
}

fn align_local_axes_to_view(
    world_forward: glam::Vec3,
    world_up: glam::Vec3,
    local_forward: glam::Vec3,
    local_up: glam::Vec3,
) -> glam::Quat {
    let world_forward = world_forward.normalize_or_zero();
    if world_forward.length_squared() < 1e-6 {
        return glam::Quat::IDENTITY;
    }

    let mut world_up = world_up.normalize_or_zero();
    if world_up.length_squared() < 1e-6 {
        world_up = glam::Vec3::Y;
    }
    let mut world_right = world_forward.cross(world_up);
    if world_right.length_squared() < 1e-6 {
        world_up = glam::Vec3::Z;
        world_right = world_forward.cross(world_up);
    }
    world_right = world_right.normalize_or_zero();
    world_up = world_right.cross(world_forward).normalize_or_zero();

    let local_forward = local_forward.normalize_or_zero();
    let mut local_up = local_up.normalize_or_zero();
    let mut local_right = local_forward.cross(local_up);
    if local_right.length_squared() < 1e-6 {
        return glam::Quat::IDENTITY;
    }
    local_right = local_right.normalize_or_zero();
    local_up = local_right.cross(local_forward).normalize_or_zero();

    let local_basis = glam::Mat3::from_cols(local_right, local_up, local_forward);
    let world_basis = glam::Mat3::from_cols(world_right, world_up, world_forward);
    glam::Quat::from_mat3(&(world_basis * local_basis.transpose()))
}

pub struct Engine {
    renderer: Renderer,
    framebuffer_resized: bool,
    last_frame_time: Instant,
    last_fps_time: Instant,
    frame_count: u32,
    mouse_sensitivity: f32,
    weather: Weather,
    base_sky_color: [f32; 4],
    sun_sphere_renderer: SunSphereRenderer,
    cloud_renderer: CloudRenderer,
    rain_renderer: RainRenderer,
    stars_renderer: StarsRenderer,
    fire_renderer: FireRenderer,
    sdf_renderer: SdfRenderer,
    last_view: glam::Mat4,
    last_proj: glam::Mat4,
    pub player: Player,
    sun_enabled: bool,
    bloom_enabled: bool,
    game_state: GameState,
    triggers: Vec<Trigger>,
    world: hecs::World,
    physics: PhysicsConfig,
    lighting: LightingConfig,
    text_renderer: TextRenderer,
    menu_renderer: MenuRenderer,
    hud_renderer: HudRenderer,
    player_state: PlayerState,
    inventory_open: bool,
    menu_open: bool,
    game_state_config: GameStateConfig,
    current_gui: Vec<GuiElementConfig>,
    current_scene: String,
    game_time: f32,
    time_speed: f32,
    debug_mode: bool,
    debug_text_idx: Option<usize>,
    wind_time: f32,
    effect_time: f32,
    rain_time: f32,
    wind: WindConfig,
     
    world_colliders: Vec<Aabb>,
    scratch_colliders: Vec<Aabb>,
     
    interactables: std::collections::HashMap<usize, String>,
     
    model_paths: std::collections::HashMap<usize, String>,
     
    collider_to_model: std::collections::HashMap<usize, usize>,
    firearm_system: FirearmSystem,
    interaction_text_idx: Option<usize>,
    inventory_title_idx: Option<usize>,
    inventory_icon_indices: Vec<usize>,
    inventory_visible_icons: usize,
    inventory_ui_dirty: bool,
     
    hotbar_icon_indices: [Option<usize>; 5],
    hotbar_icon_paths: [Option<String>; 5],
    equipped_text_idx: Option<usize>,
    last_held_model_idx: Option<usize>,
     
    drag_source: Option<DragSource>,
    dragged_item: Option<player::InventoryItem>,
    drag_icon_idx: Option<usize>,
     
    test_lights: Vec<GpuLight>,
    light_attachments: Vec<Option<LightAttachment>>,
    model_base_positions: Vec<glam::Vec3>,
    last_mouse_delta: glam::Vec2,
}

impl Engine {
    fn build_scene_lights(
        scene: &state::config::Scene,
        model_base_positions: &[glam::Vec3],
    ) -> (Vec<GpuLight>, Vec<Option<LightAttachment>>) {
        let mut lights = Vec::new();
        let mut attachments = Vec::new();
        let attach_eps = 0.01;

        for l in &scene.lights {
            let pos = glam::vec3(l.position[0], l.position[1], l.position[2]);
            let mut attachment = None;

            if let Some(needle) = &l.follow_model {
                if let Some((idx, base)) = scene
                    .models
                    .iter()
                    .enumerate()
                    .find_map(|(idx, m)| {
                        if m.path.contains(needle) {
                            Some((idx, glam::Vec3::from(m.offset)))
                        } else {
                            None
                        }
                    })
                {
                    attachment = Some(LightAttachment {
                        model_idx: idx,
                        relative_offset: pos - base,
                    });
                }
            } else if let Some((idx, base)) = model_base_positions
                .iter()
                .copied()
                .enumerate()
                .find(|(_, base)| (pos - *base).length_squared() <= attach_eps * attach_eps)
            {
                attachment = Some(LightAttachment {
                    model_idx: idx,
                    relative_offset: pos - base,
                });
            }

            lights.push(crate::engine::renderer::lighting::GpuLight::point(
                pos,
                l.radius,
                glam::vec3(l.color[0], l.color[1], l.color[2]),
                l.intensity,
            ));
            attachments.push(attachment);
        }

        (lights, attachments)
    }

    fn update_attached_lights(&mut self) {
        for (light_idx, attachment) in self.light_attachments.iter().enumerate() {
            let Some(attachment) = attachment else {
                continue;
            };

            let mut world_pos = self
                .model_base_positions
                .get(attachment.model_idx)
                .copied()
                .unwrap_or(glam::Vec3::ZERO);

            for (_id, (pos, mesh)) in self.world.query::<(&Position, &StaticMesh)>().iter() {
                if mesh.model_id as usize == attachment.model_idx {
                    world_pos = pos.0;
                    break;
                }
            }

            let pos = world_pos + attachment.relative_offset;
            if let Some(light) = self.test_lights.get_mut(light_idx) {
                light.position[0] = pos.x;
                light.position[1] = pos.y;
                light.position[2] = pos.z;
            }
        }
    }
    pub fn new(
        event_loop: &EventLoop<()>,
        fullscreen: bool,
        crosshair_enabled_default: bool,
        msaa_samples: vk::SampleCountFlags,
    ) -> Self {
        let state_config = state::config::GameStateConfig::load("assets/config/game.ron");
        let initial_time = state_config.initial_time;
        let lighting = state_config.lighting.clone();  
         
        let time_speed = 24.0 / state_config.day_cycle_duration;
        let current_scene = state_config.default_scene.clone();
        let scene_path = format!("{}/{}.ron", state_config.scene_dir, current_scene);
        let scene = state::config::Scene::load(&scene_path);
        let wind = scene
            .wind
            .clone()
            .unwrap_or_else(|| state_config.wind.clone());

        let mut world = hecs::World::new();
        let mut models = Vec::new();
        let mut model_base_positions = Vec::new();
        
        for (idx, m) in scene.models.iter().enumerate() {
            if m.rigid_body {
                 let pos = glam::Vec3::from(m.offset);
                 let pushable = m.pushable;
                 world.spawn((
                     Position(pos),
                     PhysicsBody {
                         radius: 1.0 * m.scale,  
                         active: true,
                         base_position: pos,
                         on_ground: false,
                         pushable,
                     },
                     Velocity(glam::Vec3::ZERO),
                     StaticMesh {
                         model_id: idx as u32,
                         offset: glam::Vec3::ZERO,
                     }
                 ));
            }
            models.push(MeshletModelDesc {
                path: m.path.clone(),
                offset: m.offset,
                scale: m.scale,
                material_id: m.material_id,
                collision_enabled: m.collision,
                collider_type: match m.collider_type.as_str() {
                    "stairs" => ColliderType::Stairs,
                    "ladder" => ColliderType::Ladder,
                    "ramp" => ColliderType::Ramp,
                    _ => ColliderType::Solid,
                },
                opacity: m.opacity,
                color: m.material.as_ref().map(|mat| mat.color).unwrap_or([1.0; 3]),
                metalness: m.material.as_ref().map(|mat| mat.metalness).unwrap_or(0.0),
                roughness: m.material.as_ref().map(|mat| mat.roughness).unwrap_or(0.5),
            });
            model_base_positions.push(glam::Vec3::from(m.offset));
        }

        let crosshair_scale = state_config.crosshair.scale;
        let crosshair_style = parse_crosshair_style(&state_config.crosshair.style);
        let crosshair_enabled = state_config.crosshair.enabled && crosshair_enabled_default;

         
        let skinned_asset = scene
            .ai
            .iter()
            .find_map(|ai| ai.asset.clone());

        let window = Renderer::init_window(event_loop, fullscreen);
        let mut renderer = unsafe {
            Renderer::new(window, fullscreen, msaa_samples)
        };
        
        let colliders = unsafe {
             renderer.update_models(&models)
        };
        let colliders_len = colliders.len();

        let mut interactables = std::collections::HashMap::new();
        let mut model_paths = std::collections::HashMap::new();
        let mut collider_to_model = std::collections::HashMap::new();
        let mut firearm_system = FirearmSystem::new();
        let mut collider_idx_counter = 0;
        for (idx, m) in scene.models.iter().enumerate() {
            model_paths.insert(idx, m.path.clone());
            if let Some(interaction) = &m.interactable {
                interactables.insert(idx, interaction.clone());
            }
            firearm_system.register_model(idx, m.firearm, m.bullet, &m.path);
            if m.collision {
                collider_to_model.insert(collider_idx_counter, idx);
                collider_idx_counter += 1;
            }
        }
        firearm_system.init_bullets();
        
        renderer.crosshair_style = crosshair_style;
        renderer.crosshair_scale = crosshair_scale;
        renderer.crosshair_enabled = crosshair_enabled;
        
        unsafe {
            renderer.crosshair_renderer.update_scale(
                &renderer.instance,
                &renderer.device,
                renderer.physical_device,
                renderer.command_pool,
                renderer.graphics_queue,
                crosshair_scale,
            );
        }
        
        let weather = weather::load_weather_scene();
        if current_scene == "menu" || current_scene == "scene3" {
            renderer.sky_color = [0.0, 0.0, 0.0, 1.0];
        } else {
            renderer.sky_color = weather.sky_color;
        }
        
         
        let physics = state_config.player_config.physics.clone();

         
        let lighting = state_config.lighting.clone();
        let sun_sphere_renderer = SunSphereRenderer::new(&renderer.device, renderer.render_pass, msaa_samples);
        let cloud_renderer = CloudRenderer::new(&renderer.device, renderer.render_pass, msaa_samples);
        let rain_renderer = RainRenderer::new(&renderer.device, renderer.render_pass, msaa_samples);
        let stars_renderer = StarsRenderer::new(&renderer.device, renderer.render_pass, msaa_samples);
        let fire_renderer = FireRenderer::new(
            &renderer.instance,
            &renderer.device,
            renderer.physical_device,
            renderer.render_pass,
            msaa_samples,
        );
        let sdf_renderer = SdfRenderer::new(
            &renderer.instance,
            &renderer.device,
            renderer.physical_device,
            renderer.render_pass,
            renderer.graphics_queue,
            renderer.command_pool,
            msaa_samples,
        );
        
         
        
        let (test_lights, light_attachments) =
            Self::build_scene_lights(&scene, &model_base_positions);
        let mut text_renderer = TextRenderer::new(
            &renderer.device,
            renderer.render_pass,
            "assets/GothicA1-Regular.ttf",
            msaa_samples,
        );
        
        let pool = renderer.ui_descriptor_pool;
        for elem in &scene.gui {
            unsafe {
                text_renderer.add_text(
                    &renderer.instance,
                    &renderer.device,
                    renderer.physical_device,
                    renderer.command_pool,
                    renderer.graphics_queue,
                    pool,
                    renderer.swapchain_extent,
                    &elem.text,
                    elem.position,
                    elem.scale,
                    elem.action.clone(),
                );
            }
        }

        let menu_renderer = MenuRenderer::new(
            &renderer.instance,
            &renderer.device,
            renderer.physical_device,
            renderer.command_pool,
            renderer.graphics_queue,
            renderer.render_pass,
            renderer.swapchain_extent,
            msaa_samples,
        );
        
        let hud_renderer = HudRenderer::new(
            &renderer.instance,
            &renderer.device,
            renderer.physical_device,
            renderer.command_pool,
            renderer.graphics_queue,
            renderer.render_pass,
            msaa_samples,
        );
        
         
        let mut triggers = Vec::new();
        for t in &scene.triggers {
            triggers.push(Trigger {
                id: t.id.clone(),
                region: t.region.clone(),
                fire: t.fire.clone(),
                actions: t.actions.clone(),
            });
        }

        let mut initial_skinned_descs = Vec::new();
        let bunny_model_idx = scene.models.iter().position(|m| m.path.contains("bunny")).unwrap_or(0);

        for ai in &scene.ai {
            let pos = glam::Vec3::from_array(ai.position);
            let forward = glam::Vec3::Z;  
            
            if let Some(asset_path) = &ai.asset {
                 
                initial_skinned_descs.push(SkinnedActorDesc {
                    id: ai.id.clone(),
                    path: asset_path.clone(),
                    position: pos,
                    scale: None,
                    rotation_y_deg: None,
                });
                
                world.spawn((
                    Position(pos),
                     AiAgent {
                        state: AIState::new(pos, forward),
                        current_task: None,
                        wander_radius: if ai.behavior == "idle" { 0.0 } else { 8.0 },
                        speed: 2.5,
                    },
                    SkinnedMesh {
                        resource_id: ai.id.clone(),
                    }
                ));
            } else if ai.ai_type == "bunny" {
                 
                world.spawn((
                    Position(pos),
                    AiAgent {
                        state: AIState::new(pos, forward),
                        current_task: None,
                        wander_radius: 6.0,
                        speed: 1.5,
                    },
                    StaticMesh {
                         model_id: bunny_model_idx as u32,
                         offset: glam::Vec3::ZERO,
                    }
                ));
            }
        }
        
        unsafe {
            renderer.set_skinned_actors(&initial_skinned_descs);
        }

        let initial_gui = scene.gui.clone();
        let menu_open = state_config.default_scene != "scene2";

        Self {
            renderer,
            framebuffer_resized: false,
            last_frame_time: Instant::now(),
            last_fps_time: Instant::now(),
            frame_count: 0,
            mouse_sensitivity: 0.0025,
            weather,
            base_sky_color: [0.4, 0.6, 1.0, 1.0],
            sun_sphere_renderer,
            cloud_renderer,
            rain_renderer,
            stars_renderer,
            fire_renderer,
            sdf_renderer,
            last_view: glam::Mat4::IDENTITY,
            last_proj: glam::Mat4::IDENTITY,
            player: Player::new(glam::Vec3::new(0.0, 0.5, 8.0)),
            sun_enabled: true,
            bloom_enabled: true,
            game_state: GameState::default(),
            triggers,
            world,
            physics,
            lighting,
            text_renderer,
            menu_renderer,
            hud_renderer,
            player_state: PlayerState::new(),
            inventory_open: false,
            menu_open,
            game_state_config: state_config,
            current_gui: initial_gui,
            current_scene,
            game_time: initial_time,
            time_speed,
            debug_mode: false,
            debug_text_idx: None,
            wind_time: 0.0,
            effect_time: 0.0,
            rain_time: 0.0,
            wind,
            world_colliders: colliders,
            scratch_colliders: Vec::with_capacity(colliders_len + 1),
            interactables,
            model_paths,
            collider_to_model,
            firearm_system,
            interaction_text_idx: None,
            inventory_title_idx: None,
            inventory_icon_indices: Vec::new(),
            inventory_visible_icons: 0,
            inventory_ui_dirty: true,
            hotbar_icon_indices: [None; 5],
            hotbar_icon_paths: [const { None }; 5],
            equipped_text_idx: None,
            last_held_model_idx: None,
            drag_source: None,
            dragged_item: None,
            drag_icon_idx: None,
            test_lights,
            light_attachments,
            model_base_positions,
            last_mouse_delta: glam::Vec2::ZERO,
        }
    }

    pub fn window(&self) -> &Window {
        &self.renderer.window
    }

    pub fn handle_resize(&mut self) {
        self.framebuffer_resized = true;
    }

    pub fn handle_mouse_delta(&mut self, dx: f64, dy: f64) {
        if self.menu_open || self.inventory_open {
            return;
        }
        self.last_mouse_delta = glam::Vec2::new(dx as f32, dy as f32);
        self.player
            .add_look(dx as f32, dy as f32, self.mouse_sensitivity);
    }

    pub fn toggle_menu(&mut self) {
        self.menu_open = !self.menu_open;
    }

    pub fn set_debug(&mut self, enabled: bool) {
        self.debug_mode = enabled;
        if !enabled {
            if let Some(idx) = self.debug_text_idx.take() {
                unsafe {
                    self.text_renderer
                        .remove_text(idx, &self.renderer.device, self.renderer.ui_descriptor_pool);
                }
                self.handle_text_removed(idx);
            } else {
                self.debug_text_idx = None;
            }
        }
    }

    pub fn set_crosshair_enabled(&mut self, enabled: bool) {
        self.renderer.crosshair_enabled = enabled;
    }

    pub fn set_bloom_enabled(&mut self, enabled: bool) {
        self.bloom_enabled = enabled;
        self.renderer.set_bloom_enabled(enabled);
    }

    pub fn set_crosshair_style(&mut self, style: CrosshairStyle, scale: f32) {
        self.renderer.crosshair_style = style;
        self.renderer.crosshair_scale = scale;
        unsafe {
            self.renderer.crosshair_renderer.update_scale(
                    &self.renderer.instance,
                    &self.renderer.device,
                    self.renderer.physical_device,
                    self.renderer.command_pool,
                    self.renderer.graphics_queue,
                    scale,
                );
        }
    }



     
     
     
    pub fn load_scene(&mut self, scene_name: &str) {
        println!("Loading scene: {}", scene_name);
        let scene_path = format!("{}/{}.ron", self.game_state_config.scene_dir, scene_name);
        let scene = state::config::Scene::load(&scene_path);

         
         
        let mut models = Vec::new();
        let mut model_base_positions = Vec::new();
        
         
        self.world.clear();
        self.last_held_model_idx = None;
        
        for (idx, m) in scene.models.iter().enumerate() {
            if m.rigid_body {
                 let pos = glam::Vec3::from(m.offset);
                 let pushable = m.pushable;
                 self.world.spawn((
                     Position(pos),
                     PhysicsBody {
                         radius: 1.0 * m.scale,
                         active: true,
                         base_position: pos,
                         on_ground: false,
                         pushable,
                     },
                     Velocity(glam::Vec3::ZERO),
                     StaticMesh {
                         model_id: idx as u32,
                         offset: glam::Vec3::ZERO,
                     }
                 ));
            }
            models.push(MeshletModelDesc {
                path: m.path.clone(),
                offset: m.offset,
                scale: m.scale,
                material_id: m.material_id,
                collision_enabled: m.collision,
                collider_type: match m.collider_type.as_str() {
                    "stairs" => ColliderType::Stairs,
                    "ladder" => ColliderType::Ladder,
                    "ramp" => ColliderType::Ramp,
                    _ => ColliderType::Solid,
                },
                opacity: m.opacity,
                color: m.material.as_ref().map(|mat| mat.color).unwrap_or([1.0; 3]),
                metalness: m.material.as_ref().map(|mat| mat.metalness).unwrap_or(0.0),
                roughness: m.material.as_ref().map(|mat| mat.roughness).unwrap_or(0.5),
            });
            model_base_positions.push(glam::Vec3::from(m.offset));
        }
        
         
        unsafe {
            self.renderer.device.device_wait_idle().unwrap();
            self.world_colliders = self.renderer.update_models(&models);
        }
        self.scratch_colliders.clear();

         
        self.interactables.clear();
        self.model_paths.clear();
        self.collider_to_model.clear();
        self.interactables.clear();
        self.model_paths.clear();
        self.collider_to_model.clear();
        self.firearm_system.clear();
        let mut collider_idx_counter = 0;
        for (idx, m) in scene.models.iter().enumerate() {
             
            self.model_paths.insert(idx, m.path.clone());
            if let Some(interaction) = &m.interactable {
                self.interactables.insert(idx, interaction.clone());
            }
            self.firearm_system.register_model(idx, m.firearm, m.bullet, &m.path);
            if m.collision {
                self.collider_to_model.insert(collider_idx_counter, idx);
                collider_idx_counter += 1;
            }
        }
        self.firearm_system.init_bullets();


         
        self.triggers.clear();
        for t in &scene.triggers {
            self.triggers.push(Trigger {
                id: t.id.clone(),
                region: t.region.clone(),
                fire: t.fire.clone(),
                actions: t.actions.clone(),
            });
        }
        
         
        self.current_gui = scene.gui.clone();
        
        unsafe {
            let pool = self.renderer.ui_descriptor_pool;  
            self.text_renderer.clear_instances(&self.renderer.device, pool);
            for elem in &self.current_gui {
                self.text_renderer.add_text(
                    &self.renderer.instance,
                    &self.renderer.device,
                    self.renderer.physical_device,
                    self.renderer.command_pool,
                    self.renderer.graphics_queue,
                    pool,
                    self.renderer.swapchain_extent,
                    &elem.text,
                    elem.position,
                    elem.scale,
                    elem.action.clone(),
                );
            }
        }
        self.debug_text_idx = None;
        self.interaction_text_idx = None;
        self.inventory_title_idx = None;
        self.inventory_icon_indices.clear();
        self.hotbar_icon_indices = [None; 5];
        self.hotbar_icon_paths = [const { None }; 5];
        self.inventory_visible_icons = 0;
        self.inventory_ui_dirty = true;

        let (test_lights, light_attachments) =
            Self::build_scene_lights(&scene, &model_base_positions);
        self.test_lights = test_lights;
        self.light_attachments = light_attachments;
        self.model_base_positions = model_base_positions;

         
        let mut skinned_descs = Vec::new();
        let bunny_model_idx = scene.models.iter().position(|m| m.path.contains("bunny")).unwrap_or(0);

        for ai in &scene.ai {
            let pos = glam::Vec3::from_array(ai.position);
            let forward = glam::Vec3::Z;

            if let Some(asset_path) = &ai.asset {
                 skinned_descs.push(SkinnedActorDesc {
                    id: ai.id.clone(),
                    path: asset_path.clone(),
                    position: pos,
                    scale: None,
                    rotation_y_deg: None,
                 });
                  
                 if scene_name == "scene2" {
                     self.world.spawn((
                        Position(pos),
                        AiAgent {
                            state: AIState::new(pos, forward),
                            current_task: None,
                            wander_radius: if ai.behavior == "idle" { 0.0 } else { 8.0 },
                            speed: 2.5,
                        },
                        SkinnedMesh {
                            resource_id: ai.id.clone(),
                        }
                     ));
                 }
            } else if ai.ai_type == "bunny" && scene_name == "scene2" {
                 self.world.spawn((
                    Position(pos),
                    AiAgent {
                        state: AIState::new(pos, forward),
                        current_task: None,
                        wander_radius: if ai.behavior == "idle" { 0.0 } else { 6.0 },
                        speed: 1.5,
                    },
                    StaticMesh {
                         model_id: bunny_model_idx as u32,
                         offset: glam::Vec3::ZERO,
                    }
                 ));
            }
        }
        
        unsafe {
            self.renderer.set_skinned_actors(&skinned_descs);
        }
        
         
        if scene_name == "scene2" {
             self.player = Player::new(glam::Vec3::new(0.0, 0.5, 8.0));
             self.menu_open = false;
        } else {
             self.menu_open = true;  
        }

        self.current_scene = scene_name.to_string();

        self.wind = scene
            .wind
            .clone()
            .unwrap_or_else(|| self.game_state_config.wind.clone());

         
        if scene_name == "menu" || scene_name == "scene3" {
             
            self.renderer.sky_color = [0.0, 0.0, 0.0, 1.0];
        } else {
            let weather = weather::load_weather_scene();
            self.weather = weather;
            self.renderer.sky_color = self.weather.sky_color;
        }
    }

    fn update_camera(&mut self, input: &InputState, dt: f32) -> FrameInput {
        self.scratch_colliders.clear();
        self.scratch_colliders.extend(self.world_colliders.iter().copied());
         
        for (_id, (pos, body)) in self.world.query::<(&Position, &PhysicsBody)>().iter() {
            if body.active {
                let extents = glam::Vec3::splat(body.radius);
                self.scratch_colliders.push(Aabb::new(pos.0 - extents, pos.0 + extents, ColliderType::Solid));
            }
        }

        physics::player_physics::step(
            &mut self.player,
            input,
            &self.physics,
            &self.scratch_colliders,
            dt,
        );
        let (forward, right, up) = self.player.view_axes();
        let mut push_dir = glam::Vec3::ZERO;
        if input.forward {
            push_dir += forward;
        }
        if input.backward {
            push_dir -= forward;
        }
        if input.left {
            push_dir -= right;
        }
        if input.right {
            push_dir += right;
        }
        push_dir.y = 0.0;
        if push_dir.length_squared() > 0.0 {
            push_dir = push_dir.normalize();
        }
        
         
        for (_id, (pos, body, vel)) in self.world.query_mut::<(&mut Position, &mut PhysicsBody, &mut Velocity)>() {
             body.step(pos, vel, &self.player, &self.world_colliders, self.physics.gravity, push_dir, dt);
        }
        let eye = self.player.eye_position();
        let equipped_model_idx = self
            .player_state
            .equipped_item
            .as_deref()
            .and_then(|item_id| item_id.strip_prefix("item_"))
            .and_then(|idx| idx.parse::<usize>().ok());

         
         
        
        let ray_dir = forward.normalize();
        let ray_origin = eye;
        let min_dist = 5.0;  
        let mut best_dist = min_dist;
        let mut hit_model: Option<usize> = None;

        // Parallel ray intersection against world colliders
        let best_hit = self.world_colliders
            .par_iter()
            .enumerate()
            .filter_map(|(i, aabb)| {
                aabb.ray_intersect(ray_origin, ray_dir)
                    .filter(|&dist| dist < min_dist)
                    .and_then(|dist| {
                        self.collider_to_model.get(&i)
                            .filter(|&&model_idx| Some(model_idx) != equipped_model_idx)
                            .map(|&model_idx| (dist, model_idx))
                    })
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        
        if let Some((dist, model_idx)) = best_hit {
            best_dist = dist;
            hit_model = Some(model_idx);
        }
        
         
        for (_id, (pos, body, mesh)) in self.world.query::<(&Position, &PhysicsBody, &StaticMesh)>().iter() {
             let margin = 0.05;
             let extents = glam::Vec3::splat(body.radius + margin);
             let aabb = Aabb::new(pos.0 - extents, pos.0 + extents, ColliderType::Solid);
             
             if let Some(dist) = aabb.ray_intersect(ray_origin, ray_dir) {
                 if dist < best_dist {
                     if Some(mesh.model_id as usize) == equipped_model_idx {
                         continue;
                     }
                     best_dist = dist;
                     hit_model = Some(mesh.model_id as usize);
                 }
             }
        }
        
        let mut hovering_interactable = false;
        if let Some(idx) = hit_model {
             if let Some(interaction) = self.interactables.get(&idx) {
                 hovering_interactable = true;
                 
                  
                 if self.interaction_text_idx.is_none() {
                     println!("Showing interaction text for {}", interaction); 
                     unsafe {
                        self.text_renderer.add_text(
                            &self.renderer.instance,
                            &self.renderer.device,
                            self.renderer.physical_device,
                            self.renderer.command_pool,
                            self.renderer.graphics_queue,
                            self.renderer.ui_descriptor_pool,
                            self.renderer.swapchain_extent,
                            &format!("Press E to {}", interaction),
                            [0.0, -0.1],  
                            0.5,
                            None
                        );
                        self.interaction_text_idx = Some(self.text_renderer.instances.len() - 1);
                     }
                 }

                 if input.interact {
                     println!("Interacting with model {}: {}", idx, interaction);
                     if interaction == "pickup" {
                          
                         let (item_name, item_icon) = if let Some(path) = self.model_paths.get(&idx) {
                             if path.contains("ak47") {
                                 ("AK-47".to_string(), Some("assets/models/ak47/WPNT_AKM_AlbedoTransparency.png".to_string()))
                             } else if path.contains("rock") {
                                 ("Rock".to_string(), Some("assets/rock.png".to_string()))
                             } else {
                                 ("Item".to_string(), None)
                             }
                         } else {
                             ("Item".to_string(), None)
                         };
                         
                          
                         use player::InventoryItem;
                         self.player_state.add_item(InventoryItem {
                             id: format!("item_{}", idx),
                             name: item_name,
                             count: 1,
                             icon: item_icon,
                             firearm: self.firearm_system.firearm_models.contains(&idx),
                         });
                         self.inventory_ui_dirty = true;
                         println!("Picked up item! Inventory: {:?}", self.player_state.inventory);
                         
                          
                         for (_id, (pos, body, vel, mesh)) in self.world.query_mut::<(&mut Position, &mut PhysicsBody, &mut Velocity, &StaticMesh)>() {
                             if mesh.model_id == idx as u32 {
                                 pos.0 = glam::Vec3::new(0.0, -1000.0, 0.0);
                                 body.active = false;
                                 vel.0 = glam::Vec3::ZERO;
                                 break;
                             }
                         }

                          
                         if let Some(text_idx) = self.interaction_text_idx.take() {
                             unsafe {
                                 self.text_renderer.remove_text(text_idx, &self.renderer.device, self.renderer.ui_descriptor_pool);
                             }
                             self.handle_text_removed(text_idx);
                         }
                     }
                 }
             }
        }

        if !hovering_interactable {
             
             if let Some(text_idx) = self.interaction_text_idx.take() {
                 unsafe {
                     self.text_renderer.remove_text(text_idx, &self.renderer.device, self.renderer.ui_descriptor_pool);
                 }
                 self.handle_text_removed(text_idx);
             }
        }

         
        let mut owned_model_indices = HashSet::new();
        
         
        for item in &self.player_state.inventory {
             if let Some(suffix) = item.id.strip_prefix("item_") {
                 if let Ok(idx) = suffix.parse::<u32>() {
                     owned_model_indices.insert(idx);
                 }
             }
        }
         
        for item_opt in &self.player_state.hotbar {
            if let Some(item) = item_opt {
                 if let Some(suffix) = item.id.strip_prefix("item_") {
                     if let Ok(idx) = suffix.parse::<u32>() {
                         owned_model_indices.insert(idx);
                     }
                 }
            }
        }
         
        if let Some(item) = &self.dragged_item {
             if let Some(suffix) = item.id.strip_prefix("item_") {
                 if let Ok(idx) = suffix.parse::<u32>() {
                     owned_model_indices.insert(idx);
                 }
             }
        }
        
         
        let equipped_idx = if let Some(id) = &self.player_state.equipped_item {
            if let Some(suffix) = id.strip_prefix("item_") {
                suffix.parse::<u32>().ok()
            } else { None }
        } else { None };
        
        if let Some(idx) = equipped_idx {
            owned_model_indices.insert(idx);
        }

         
        for (_id, (pos, body, vel, mesh)) in self.world.query_mut::<(&mut Position, &mut PhysicsBody, &mut Velocity, &StaticMesh)>() {
             if let Some(eq_idx) = equipped_idx {
                 if mesh.model_id == eq_idx {
                      
                     body.active = false;
                     
                     let sway_cfg = &self.game_state_config.weapon_sway;
                     let hand_offset = self.firearm_system.calculate_hand_offset(
                         forward,
                         right,
                         up,
                         self.player.velocity,
                         self.last_mouse_delta,
                         sway_cfg.camera_sway_scale,
                         sway_cfg.movement_sway_scale,
                         sway_cfg.sway_speed,
                         sway_cfg.sway_limit,
                         dt,
                     );
                     pos.0 = eye + hand_offset;
                     vel.0 = glam::Vec3::ZERO;
                     continue;
                 }
             }
             
             if owned_model_indices.contains(&mesh.model_id) {
                  
                  
                 if pos.0.y > -500.0 {
                     pos.0 = glam::Vec3::new(0.0, -1000.0, 0.0);
                     body.active = false;
                     vel.0 = glam::Vec3::ZERO;
                 }
             }
        }

        if input.drop {
             if let Some(item_id) = self.player_state.equipped_item.clone() {
                 if let Some(model_idx_str) = item_id.strip_prefix("item_") {
                     if let Ok(idx) = model_idx_str.parse::<u32>() {
                         for (_id, (pos, body, vel, mesh)) in self.world.query_mut::<(&mut Position, &mut PhysicsBody, &mut Velocity, &StaticMesh)>() {
                             if mesh.model_id == idx {
                                 println!("Dropping item {}", item_id);
                                 body.active = true;
                                 pos.0 = eye + forward * 1.5;
                                 vel.0 = forward * 5.0;  
                                 
                                  
                                 self.player_state.remove_item(&item_id, 1);
                                 
                                  
                                 for i in 0..5 {
                                     if let Some(hotbar_item) = &self.player_state.hotbar[i] {
                                         if hotbar_item.id == item_id {
                                             self.player_state.hotbar[i] = None;
                                             break;
                                         }
                                     }
                                 }
                                 
                                 self.player_state.equipped_item = None;
                                 self.inventory_ui_dirty = true;
                                 self.update_equipped_text();
                                 break;
                             }
                         }
                     }
                 }
             }
        }

        let view = glam::Mat4::look_at_rh(eye, eye + forward, up);
        let mut proj = glam::Mat4::perspective_rh(
            60.0f32.to_radians(),
            self.renderer.swapchain_extent.width as f32
                / self.renderer.swapchain_extent.height as f32,
            0.1,
            200000.0,
        );
         
        proj.y_axis.y *= -1.0;
        self.last_view = view;
        self.last_proj = proj;

        self.process_triggers(eye);


         
        let _dir_light = self
            .lighting
            .directional
            .get(0)
            .cloned()
            .unwrap_or_else(|| config::DirectionalLight {
                direction: self.weather.sun.direction.to_array(),
                color: self.weather.sun.color,
                intensity: self.weather.sun.intensity,
            });

        let sun_dir = if self.sun_enabled {
             
            self.weather.sun.direction
        } else {
            glam::Vec3::ZERO
        };
        let sun_pos = if self.sun_enabled {
            self.weather.sun.position
        } else {
            glam::Vec3::ZERO
        };
        let sun_intensity = if self.sun_enabled {
            self.weather.sun.intensity
        } else {
            0.0
        };

        let wind_dir = glam::Vec2::new(self.wind.direction[0], self.wind.direction[1]);

        FrameInput {
            view,
            proj,
            camera_pos: eye,
            sun_pos,
            sun_dir,
            sun_intensity,
            sun_color: if self.sun_enabled {
                glam::Vec3::from_array(self.weather.sun.color)
            } else {
                glam::Vec3::ZERO
            },
            sphere_offset: glam::Vec3::ZERO,  
            sphere_model_idx: None,  
            hovered_model_idx: if hovering_interactable { hit_model.map(|idx| idx as u32) } else { None },
            effect_time: self.effect_time,
            wind_time: self.wind_time,
            cloud_config: self.game_state_config.clouds.clone(),
            rain_config: self.game_state_config.rain.clone(),
            snow_config: self.game_state_config.snow.clone(),
            enable_snow: if self.current_scene == "menu" { false } else { self.game_state_config.enable_snow },
            enable_rain: if self.current_scene == "menu" { false } else { self.game_state_config.enable_rain },
            rain_time: self.rain_time,
            wind_dir,
            wind_speed: self.wind.speed,
            wind_turbulence: self.wind.turbulence,
            dt,
        }
    }

    pub unsafe fn render(&mut self, input: &InputState) {
        let dt = self.last_frame_time.elapsed().as_secs_f32();
        self.last_frame_time = Instant::now();

         
        self.game_time += dt * self.time_speed;
        self.effect_time += dt * 12.0;
        self.wind_time += dt;
        self.rain_time += dt;
        self.weather.update(self.game_time);
        
         
        if self.sun_enabled && self.current_scene != "menu" && self.current_scene != "scene3" {
            self.renderer.sky_color = self.weather.sky_color;
        }
        
         
        self.update_attached_lights();
        self.renderer
            .lighting_system
            .upload_lights(&self.renderer.device, &self.test_lights);
         
         
        let mut static_updates: Vec<(usize, glam::Vec3)> = Vec::new();
        
         
        for (_id, (pos, agent, skinned, static_mesh)) in self.world.query_mut::<(&mut Position, &mut AiAgent, Option<&SkinnedMesh>, Option<&StaticMesh>)>() {
            if agent.current_task.is_none() && agent.wander_radius > 0.001 {
                agent.current_task = Some(Self::random_task(agent.wander_radius, agent.speed, agent.state.position.y));
            }
            if let Some(task) = &mut agent.current_task {
                if task.step(&mut agent.state, dt) {
                    agent.current_task = None;
                }
            }
             
            pos.0 = agent.state.position; 

            if let Some(skinned_mesh) = skinned {
                unsafe {
                    self.renderer.update_skinned_actor_position(&skinned_mesh.resource_id, agent.state.position);
                }
            }
            if let Some(mesh) = static_mesh {
                 static_updates.push((mesh.model_id as usize, pos.0));
            }
        }
        
         
        for (_id, (pos, body, mesh)) in self.world.query_mut::<(&Position, &PhysicsBody, &StaticMesh)>() {
             
            let offset = pos.0 - body.base_position; 
            static_updates.push((mesh.model_id as usize, offset));
        }

        unsafe {
            self.renderer.update_model_offsets(&static_updates);
        }

        unsafe {
            self.renderer.update_skinned(dt);
        }

         
        unsafe {
            self.renderer.update_rt_tlas();
        }

        let resized = unsafe { self.renderer.resize_if_requested(self.framebuffer_resized) };
        if resized {
            unsafe {
                self.rebuild_overlays();
            }
            self.framebuffer_resized = false;
        }

        if self.renderer.swapchain_extent.width == 0 || self.renderer.swapchain_extent.height == 0 {
            return;
        }

        self.renderer
            .device
            .wait_for_fences(
                &[self.renderer.in_flight_fences[self.renderer.current_frame]],
                true,
                u64::MAX,
            )
            .unwrap();

        let (image_index, _) = match self.renderer.swapchain_loader.acquire_next_image(
            self.renderer.swapchain,
            u64::MAX,
            self.renderer.image_available_semaphores[self.renderer.current_frame],
            vk::Fence::null(),
        ) {
            Ok(result) => result,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.renderer.recreate_swapchain();
                return;
            }
            Err(e) => panic!("Acquire error: {}", e),
        };

        self.renderer
            .device
            .reset_fences(&[self.renderer.in_flight_fences[self.renderer.current_frame]])
            .unwrap();

        let frame_input = self.update_camera(input, dt);

        let equipped_model_idx = self
            .player_state
            .equipped_item
            .as_deref()
            .and_then(|item_id| item_id.strip_prefix("item_"))
            .and_then(|idx| idx.parse::<usize>().ok());
        if let Some(model_idx) = equipped_model_idx {
            let mut held_offset = None;
            for (_id, (pos, body, mesh)) in self
                .world
                .query::<(&Position, &PhysicsBody, &StaticMesh)>()
                .iter()
            {
                if mesh.model_id as usize == model_idx {
                    held_offset = Some(pos.0 - body.base_position);
                    break;
                }
            }
            if let Some(offset) = held_offset {
                unsafe {
                    self.renderer.update_model_offsets(&[(model_idx, offset)]);
                }
            }
        }

        let mut rotation_updates: Vec<(usize, glam::Quat)> = Vec::new();
        let mut rotated_idx = None;
        if let Some(model_idx) = equipped_model_idx {
            if self
                .model_paths
                .get(&model_idx)
                .map_or(false, |path| path.contains("ak47"))
            {
                let (forward, _right, up) = self.player.view_axes();
                let ak_local_forward = glam::Vec3::NEG_Y;  
                let ak_local_up = glam::Vec3::Z;
                let rotation = align_local_axes_to_view(forward, up, ak_local_forward, ak_local_up);
                rotation_updates.push((model_idx, rotation));
                rotated_idx = Some(model_idx);
            }
        }
        if self.last_held_model_idx != rotated_idx {
            if let Some(prev_idx) = self.last_held_model_idx {
                rotation_updates.push((prev_idx, glam::Quat::IDENTITY));
            }
            self.last_held_model_idx = rotated_idx;
        }
        if !rotation_updates.is_empty() {
            unsafe {
                self.renderer.update_model_rotations(&rotation_updates);
            }
        }

        self.firearm_system.update_bullets(
            dt,
            &mut self.renderer,
            &self.world_colliders,
            &mut self.world,
            &self.model_base_positions,
        );

        self.update_debug_overlay();
        self.sync_inventory_ui();
        
        // Reset mouse delta for next frame - sway returns to neutral when mouse stops
        self.last_mouse_delta = glam::Vec2::ZERO;

         
        if let Some(item) = &self.dragged_item {
            if let Some(path) = &item.icon {
                 let width = self.renderer.swapchain_extent.width as f32;
                 let height = self.renderer.swapchain_extent.height as f32;
                 let ndc_x = (input.cursor_pos[0] / width) * 2.0 - 1.0;
                 let ndc_y = (input.cursor_pos[1] / height) * 2.0 - 1.0;
                 
                 let icon_ndc = 0.1;
                 let size = [
                     icon_ndc * width * 0.5, 
                     icon_ndc * height * 0.5,
                 ];
                 let pos = [ndc_x, ndc_y];
                 
                 if let Some(idx) = self.drag_icon_idx {
                       
                       
                       
                       
                      unsafe {
                          self.text_renderer.update_position(
                              idx,
                              &self.renderer.instance,
                              &self.renderer.device,
                              self.renderer.physical_device,
                              self.renderer.command_pool,
                              self.renderer.graphics_queue,
                              self.renderer.swapchain_extent,
                              pos,
                          );
                      }
                 } else {
                      let idx = self.text_renderer.add_image(
                          &self.renderer.instance,
                          &self.renderer.device,
                          self.renderer.physical_device,
                          self.renderer.command_pool,
                          self.renderer.graphics_queue,
                          self.renderer.ui_descriptor_pool,
                          self.renderer.swapchain_extent,
                          path,
                          pos,
                          size
                      );
                      self.drag_icon_idx = Some(idx);
                 }
            }
        } else {
             if let Some(idx) = self.drag_icon_idx.take() {
                 unsafe {
                     self.text_renderer.remove_text(idx, &self.renderer.device, self.renderer.ui_descriptor_pool);
                 }
                 self.handle_text_removed(idx);
             }
        }

        let mut text_indices: Vec<usize> = Vec::new();
        if !self.menu_open {
            if self.debug_mode {
                if let Some(idx) = self.debug_text_idx {
                    text_indices.push(idx);
                }
            }
            if let Some(idx) = self.equipped_text_idx {
                text_indices.push(idx);
            }
            if let Some(idx) = self.interaction_text_idx {
               text_indices.push(idx);
            }
            if self.inventory_open {
                if let Some(idx) = self.inventory_title_idx {
                    text_indices.push(idx);
                }
                let visible = self
                    .inventory_visible_icons
                    .min(self.inventory_icon_indices.len());
                text_indices.extend(self.inventory_icon_indices.iter().take(visible).copied());
            }
             
            for i in 0..5 {
                if let Some(idx) = self.hotbar_icon_indices[i] {
                    text_indices.push(idx);
                }
            }
            if let Some(idx) = self.drag_icon_idx {
                text_indices.push(idx);
            }
        }

        let text_param = if !text_indices.is_empty() {
             Some((&self.text_renderer, Some(text_indices.as_slice())))
        } else if self.menu_open {
              
              
              
              
              
             Some((&self.text_renderer, None))
        } else {
             None
        };


        self.renderer.record_command_buffer(
            image_index as usize,
            self.renderer.current_frame,
            &frame_input,
            text_param,
            if self.menu_open || self.current_scene == "menu" {
                 
                 
                None
            } else {
                None
            },
            if self.sun_enabled { Some(&self.cloud_renderer) } else { None },  
            if self.current_scene != "menu" { Some(&self.rain_renderer) } else { None },
            if self.current_scene != "menu" { Some(&self.stars_renderer) } else { None },
            if self.current_scene != "menu" { Some(&self.sun_sphere_renderer) } else { None },
            Some(&mut self.fire_renderer),
            if self.current_scene == "scene2" { Some(&self.sdf_renderer) } else { None },
            if self.current_scene != "menu" { Some((&self.hud_renderer, self.player_state.health, self.inventory_open, self.player_state.active_hotbar_slot)) } else { None },
        );

        let wait_semaphores = [self.renderer.image_available_semaphores[self.renderer.current_frame]];
        let signal_semaphores =
            [self.renderer.render_finished_semaphores[self.renderer.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [self.renderer.command_buffers[self.renderer.current_frame]];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        self.renderer
            .device
            .queue_submit(
                self.renderer.graphics_queue,
                &[submit_info],
                self.renderer.in_flight_fences[self.renderer.current_frame],
            )
            .unwrap();

        let swapchains = [self.renderer.swapchain];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        if let Err(e) = self
            .renderer
            .swapchain_loader
            .queue_present(self.renderer.present_queue, &present_info)
        {
            match e {
                vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => {
                    self.framebuffer_resized = true;
                }
                _ => panic!("Present error: {}", e),
            }
        }

        self.renderer.current_frame = (self.renderer.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;

        self.frame_count += 1;
        if self.last_fps_time.elapsed().as_secs() >= 1 {
            let fps = self.frame_count;
            self.renderer
                .window
                .set_title(&format!("Vulkan Meshlet Engine - FPS: {}", fps));
            self.frame_count = 0;
            self.last_fps_time = Instant::now();
        }
    }


    pub fn set_sun_enabled(&mut self, enabled: bool) {
        self.sun_enabled = enabled;
        if self.current_scene == "menu" || self.current_scene == "scene3" {
            self.renderer.sky_color = [0.0, 0.0, 0.0, 1.0];
        } else {
            self.renderer.sky_color = if enabled {
                self.weather.sky_color
            } else {
                self.base_sky_color
            };
        }
    }

    fn process_triggers(&mut self, eye: glam::Vec3) {
        for trig in &self.triggers {
            let already = self.game_state.fired.contains(&trig.id);
            let can_fire = match trig.fire {
                state::FirePolicy::Once => !already,
                state::FirePolicy::Repeat => true,
            };
            if !can_fire {
                continue;
            }
            if !trig.region.contains(eye) {
                continue;
            }

            for action in &trig.actions {
                if let Some(rest) = action.strip_prefix("flag:") {
                    self.game_state.flags.insert(rest.to_string());
                } else if let Some(msg) = action.strip_prefix("log:") {
                    println!("[Trigger:{}] {}", trig.id, msg);
                } else {
                    println!("[Trigger:{}] Unknown action '{}'", trig.id, action);
                }
            }

            if matches!(trig.fire, state::FirePolicy::Once) {
                self.game_state.fired.insert(trig.id.clone());
            }
        }
    }

     
    
    fn random_task(radius: f32, speed: f32, base_y: f32) -> WalkTask {
    if radius < 0.001 {
        return WalkTask::new(glam::Vec3::ZERO, 0.0, 1.0, None, None);
    }
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let r = rng.gen_range((radius * 0.5)..radius);
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
         
         
         
         
         
         
         
        let target = glam::Vec3::new(r * angle.cos(), base_y, r * angle.sin());
        WalkTask::new(target, speed, 0.5, None, None)
    }
    
    pub fn toggle_noclip(&mut self) {
        self.player.noclip = !self.player.noclip;
        println!("Noclip: {}", self.player.noclip);
    }



    pub fn is_interface_open(&self) -> bool {
        self.inventory_open || self.menu_open
    }
}
