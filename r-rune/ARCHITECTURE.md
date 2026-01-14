# R-Rune Engine Architecture

This document provides a comprehensive breakdown of the R-Rune game engine, a custom Vulkan-based 3D engine written in Rust. It covers all major systems, the startup flow, and detailed function documentation for every file.

---

## Table of Contents

1. [Overview](#overview)
2. [Technology Stack](#technology-stack)
3. [Project Structure](#project-structure)
4. [Engine Startup Flow](#engine-startup-flow)
5. [Core Systems](#core-systems)
   - [Engine Core](#engine-core)
   - [Renderer](#renderer)
   - [Player System](#player-system)
   - [Physics System](#physics-system)
   - [ECS Components](#ecs-components)
   - [AI System](#ai-system)
6. [Environment Systems](#environment-systems)
   - [Weather System](#weather-system)
   - [Cloud Renderer](#cloud-renderer)
   - [Rain Renderer](#rain-renderer)
   - [Fire Renderer](#fire-renderer)
   - [Stars Renderer](#stars-renderer)
   - [Sun Sphere Renderer](#sun-sphere-renderer)
7. [GUI Systems](#gui-systems)
   - [Text Renderer](#text-renderer)
   - [HUD Renderer](#hud-renderer)
   - [Menu Renderer](#menu-renderer)
   - [Crosshair System](#crosshair-system)
8. [State & Configuration](#state--configuration)
9. [SDF Rendering](#sdf-rendering)
10. [Vulkan Abstraction Layer](#vulkan-abstraction-layer)
11. [Shader Reference](#shader-reference)
12. [File-by-File Function Documentation](#file-by-file-function-documentation)

---

## Overview

R-Rune is a modern 3D game engine featuring:

- **Vulkan 1.3** with Mesh Shaders (VK_EXT_mesh_shader)
- **Meshlet-based rendering** for efficient GPU-driven culling
- **Physically Based Rendering (PBR)** with configurable materials
- **Forward+ Clustered Lighting** for efficient many-light rendering
- **Dynamic day/night cycle** with procedural sun positioning
- **Volumetric clouds** using raymarching mesh shaders
- **GPU-simulated fire/smoke** with 3D fluid dynamics
- **Particle rain system** with wind integration
- **Particle snow system** with wind and turbulence effects
- **Procedural stars** rendered via mesh shaders
- **Skeletal animation** support (glTF skinned meshes)
- **First-person player physics** with collision detection
- **ECS architecture** using hecs for game entities
- **JSON/RON configuration** for scenes, triggers, and game state
- **MSAA anti-aliasing** (configurable 1x/2x/4x/8x)
- **Bloom post-processing** via compute shaders
- **Hi-Z occlusion culling** for performance optimization
- **Dynamic item pickup system** with model-based naming

---

## Technology Stack

| Component | Technology |
|-----------|------------|
| Language | Rust |
| Graphics API | Vulkan 1.3 (ash crate) |
| Windowing | winit |
| Math | glam |
| ECS | hecs |
| Model Loading | tobj (OBJ), gltf (glTF/GLB) |
| Meshlet Generation | meshopt |
| Image Loading | image crate |
| Font Rendering | rusttype |
| Shader Compilation | shaderc (runtime GLSL→SPIR-V) |
| Configuration | RON (Rusty Object Notation), JSON |
| Serialization | serde |

---

## Project Structure

```
r-rune/
├── src/
│   ├── main.rs                    # Application entry point
│   ├── engine/
│   │   ├── mod.rs                 # Engine core (~1700 lines)
│   │   ├── player.rs              # Player state and movement
│   │   ├── components.rs          # ECS components
│   │   ├── config.rs              # Physics/lighting config structs
│   │   ├── asset_loader.rs        # Async background asset loading (NEW)
│   │   ├── firearm.rs             # Weapon firing, tracers, bullet holes (NEW)
│   │   ├── input.rs               # Input handling and keybinds
│   │   ├── inventory.rs           # Inventory management
│   │   ├── overlay.rs             # Debug overlay text
│   │   ├── ai/
│   │   │   └── mod.rs             # AI state and walk tasks
│   │   ├── animation/
│   │   │   └── mod.rs             # Skeletal animation
│   │   ├── environment/           # Weather and environmental effects
│   │   │   ├── mod.rs             # Module exports
│   │   │   ├── clouds.rs          # Volumetric cloud renderer
│   │   │   ├── fire.rs            # GPU fire simulation
│   │   │   ├── rain.rs            # Particle rain system
│   │   │   ├── snow.rs            # Particle snow system
│   │   │   ├── stars.rs           # Procedural stars
│   │   │   ├── sun_sphere.rs      # Sun disc renderer
│   │   │   └── weather.rs         # Day/night cycle
│   │   ├── gui/
│   │   │   ├── mod.rs             # Module exports
│   │   │   ├── crosshair.rs       # Crosshair rendering
│   │   │   ├── hud.rs             # Health bar & inventory UI
│   │   │   ├── menu.rs            # Pause menu overlay
│   │   │   └── text.rs            # Dynamic text rendering
│   │   ├── physics/
│   │   │   ├── mod.rs             # Module exports
│   │   │   ├── capsule.rs         # Capsule collision utilities (NEW)
│   │   │   └── player_physics.rs  # Player collision/movement
│   │   ├── renderer/              # Renderer subsystem (~2300 lines total)
│   │   │   ├── mod.rs             # Renderer core (~1970 lines)
│   │   │   ├── bloom.rs           # Bloom post-processing
│   │   │   ├── composite.rs       # Final compositing pass
│   │   │   ├── hiz.rs             # Hi-Z pyramid resources
│   │   │   ├── init.rs            # Vulkan initialization helpers
│   │   │   ├── lighting.rs        # Forward+ clustered lighting
│   │   │   ├── skinned.rs         # Skinned actor management
│   │   │   ├── texture_manager.rs # On-demand texture loading with caching (NEW)
│   │   │   └── textures.rs        # Texture loading utilities
│   │   ├── sdf/
│   │   │   ├── mod.rs             # SDF renderer entry
│   │   │   └── loader.rs          # SDF model builder
│   │   └── state/
│   │       ├── mod.rs             # Game state, triggers
│   │       └── config.rs          # Scene/config loaders
│   └── vulkan/
│       ├── mod.rs                 # Module exports
│       ├── vk_buffers.rs          # Buffer creation utilities
│       ├── vk_device.rs           # Device selection
│       ├── vk_instance.rs         # Instance creation
│       ├── vk_memory.rs           # Image/depth resource creation
│       ├── vk_meshlets.rs         # Meshlet generation & loading (rayon parallelized)
│       ├── vk_pipeline.rs         # Pipeline creation
│       ├── vk_raytracing.rs       # Ray tracing acceleration structures
│       ├── vk_render_pass.rs      # Render pass setup
│       ├── vk_swapchain.rs        # Swapchain management
│       ├── vk_sync.rs             # Sync primitives
│       └── vk_textures.rs         # Texture loading
├── shader/
│   ├── meshlet.task               # Task shader for meshlet culling
│   ├── meshlet.mesh               # Mesh shader for meshlets
│   ├── meshlet.frag               # PBR fragment shader
│   ├── cloud.mesh / cloud.frag    # Volumetric clouds
│   ├── rain.mesh / rain.frag      # Rain particles
│   ├── snow.mesh / snow.frag      # Snow particles
│   ├── fire.mesh / fire.frag      # Fire rendering
│   ├── fire_sim.comp              # Fire fluid simulation
│   ├── stars.mesh / stars.frag    # Star field
│   ├── hiz.comp                   # Hi-Z pyramid generation
│   ├── bloom_down.comp            # Bloom downsampling
│   ├── bloom_up.comp              # Bloom upsampling
│   ├── composite.vert/frag        # Final compositing
│   ├── text.vert / text.frag      # UI text rendering
│   ├── menu.vert / menu.frag      # Menu panel shaders
│   ├── sdf.vert / sdf.frag        # SDF raymarching
│   └── ...                        # Additional shaders
└── assets/
    ├── config/
    │   ├── game.ron               # Main game configuration
    │   └── lighting.json          # Lighting presets
    ├── scenes/
    │   ├── menu.ron               # Main menu scene
    │   ├── scene2.ron             # Game world scene
    │   └── scene3.ron             # Additional scene
    ├── fonts/                     # TTF fonts
    ├── models/                    # OBJ/glTF models
    └── (textures at root)         # Texture images (grass.jpg, tree.png, etc.)
```

---

## Engine Startup Flow

The engine follows this startup sequence:

```
┌─────────────────────────────────────────────────────────────────┐
│                        main() Entry Point                       │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│ 1. Initialize Logging (env_logger::init())                     │
│ 2. Create Event Loop (winit::EventLoop::new())                 │
│ 3. Configure MSAA samples, fullscreen, crosshair               │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│         Engine::new() - Core Initialization                     │
│   ┌─────────────────────────────────────────────────────────┐  │
│   │ a. Load game.ron configuration                          │  │
│   │ b. Load lighting.json configuration                     │  │
│   │ c. Initialize Weather system                            │  │
│   │ d. Create Renderer (initializes Vulkan)                 │  │
│   │ e. Create environment renderers:                        │  │
│   │    - CloudRenderer                                      │  │
│   │    - RainRenderer                                       │  │
│   │    - StarsRenderer                                      │  │
│   │    - SunSphereRenderer                                  │  │
│   │    - FireRenderer                                       │  │
│   │    - SdfRenderer                                        │  │
│   │ f. Create GUI renderers:                                │  │
│   │    - TextRenderer                                       │  │
│   │    - MenuRenderer                                       │  │
│   │    - HudRenderer                                        │  │
│   │    - CrosshairRenderer                                  │  │
│   │ g. Initialize Player and PlayerState                    │  │
│   │ h. Initialize ECS World (hecs)                          │  │
│   │ i. Load initial scene (models, AI, triggers)            │  │
│   └─────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│               Renderer::new() - Vulkan Setup                    │
│   ┌─────────────────────────────────────────────────────────┐  │
│   │ a. Create window via winit                              │  │
│   │ b. Initialize Vulkan instance + validation layers       │  │
│   │ c. Create surface from window handle                    │  │
│   │ d. Select physical device (GPU)                         │  │
│   │ e. Create logical device with extensions:               │  │
│   │    - VK_EXT_mesh_shader                                 │  │
│   │    - VK_KHR_swapchain                                   │  │
│   │ f. Create swapchain and image views                     │  │
│   │ g. Create render pass with MSAA                         │  │
│   │ h. Create framebuffers                                  │  │
│   │ i. Create command pool and buffers                      │  │
│   │ j. Create descriptor pools and layouts                  │  │
│   │ k. Create graphics pipelines:                           │  │
│   │    - Meshlet pipeline (task + mesh + frag)              │  │
│   │    - Skinned mesh pipeline                              │  │
│   │    - Outline pipeline (inverted hull)                   │  │
│   │ l. Create compute pipelines:                            │  │
│   │    - Hi-Z pyramid generation                            │  │
│   │    - Bloom downscale/upscale                            │  │
│   │ m. Load textures and create samplers                    │  │
│   │ n. Load initial scene models as meshlets                │  │
│   │ o. Create sync primitives (semaphores, fences)          │  │
│   └─────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Event Loop (event_loop.run)                  │
│   ┌─────────────────────────────────────────────────────────┐  │
│   │ Each frame:                                             │  │
│   │  1. Process window events (resize, close)               │  │
│   │  2. Process input events (keyboard, mouse)              │  │
│   │  3. On RedrawRequested:                                 │  │
│   │     └─► engine.render(&input)                           │  │
│   └─────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Engine::render() Per-Frame                   │
│   ┌─────────────────────────────────────────────────────────┐  │
│   │  1. Calculate delta time (dt)                           │  │
│   │  2. Update game time and effect accumulators            │  │
│   │  3. Update Weather (sun position, sky color)            │  │
│   │  4. Update AI agents (ECS query)                        │  │
│   │  5. Update physics bodies (ECS query)                   │  │
│   │  6. Sync model offsets to renderer                      │  │
│   │  7. Update skinned animation                            │  │
│   │  8. Handle resize if needed                             │  │
│   │  9. Wait for in-flight fence                            │  │
│   │ 10. Acquire next swapchain image                        │  │
│   │ 11. update_camera() - Player physics + input            │  │
│   │ 12. Update debug overlay text                           │  │
│   │ 13. Sync inventory UI if open                           │  │
│   │ 14. record_command_buffer() - GPU commands              │  │
│   │ 15. Submit command buffer to graphics queue             │  │
│   │ 16. Present swapchain image                             │  │
│   │ 17. Advance frame counter                               │  │
│   └─────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│              Renderer::record_command_buffer()                  │
│   ┌─────────────────────────────────────────────────────────┐  │
│   │  1. Begin command buffer                                │  │
│   │  2. Fire simulation compute pass (if fire enabled)      │  │
│   │  3. Hi-Z pyramid generation (compute)                   │  │
│   │  4. Begin render pass                                   │  │
│   │  5. Set viewport and scissor                            │  │
│   │  6. Draw meshlets (task→mesh→frag pipeline)             │  │
│   │  7. Draw skinned meshes (if any)                        │  │
│   │  8. Draw outlined objects (inverted hull)               │  │
│   │  9. Draw environment (clouds, rain, stars, sun, fire)   │  │
│   │ 10. Draw SDF objects (raymarched)                       │  │
│   │ 11. Draw HUD (health bar, inventory panel)              │  │
│   │ 12. Draw text instances                                 │  │
│   │ 13. Draw menu overlay (if open)                         │  │
│   │ 14. Draw crosshair                                      │  │
│   │ 15. End render pass                                     │  │
│   │ 16. Bloom post-process (compute passes)                 │  │
│   │ 17. Final composite pass                                │  │
│   │ 18. End command buffer                                  │  │
│   └─────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Core Systems

### Engine Core

**File:** `src/engine/mod.rs` (1752 lines)

The `Engine` struct is the central coordinator. It owns all subsystems and orchestrates the game loop.

#### Key Fields

| Field | Type | Purpose |
|-------|------|---------|
| `renderer` | `Renderer` | Vulkan rendering backend |
| `player` | `Player` | First-person camera/physics |
| `player_state` | `PlayerState` | Health, inventory, equipped item |
| `world` | `hecs::World` | ECS entity storage |
| `world_colliders` | `Vec<Aabb>` | Static collision geometry |
| `physics` | `PhysicsConfig` | Gravity, friction, speeds |
| `weather` | `Weather` | Sun position, sky color |
| `game_state` | `GameState` | Flags, fired triggers |
| `triggers` | `Vec<Trigger>` | Spatial trigger zones |
| `text_renderer` | `TextRenderer` | Dynamic UI text |

#### Key Methods

| Method | Description |
|--------|-------------|
| `new()` | Full engine initialization |
| `render(&input)` | Per-frame update and render |
| `update_camera(&input, dt)` | Player physics, raycasting, interaction |
| `load_scene(name)` | Load scene from RON file |
| `handle_click(x, y)` | Process mouse click (menu/inventory/game) |
| `handle_mouse_delta(dx, dy)` | Camera look rotation |
| `toggle_inventory()` | Show/hide inventory UI |
| `process_triggers(eye)` | Check spatial triggers |
| `set_sun_enabled(bool)` | Toggle day/night rendering |
| `set_bloom_enabled(bool)` | Toggle bloom post-process |

---

### Renderer

**Directory:** `src/engine/renderer/` (~2300 lines across 7 files)

The renderer subsystem is organized into focused modules:

| Module | Lines | Purpose |
|--------|-------|---------|
| `mod.rs` | ~1770 | Core Renderer struct and command recording |
| `skinned.rs` | 398 | Animated actor types and GPU resource management |
| `init.rs` | 120 | Window, instance, swapchain, and pipeline creation |
| `hiz.rs` | 170 | Hi-Z pyramid for occlusion culling |
| `lighting.rs` | 495 | Forward+ clustered lighting system |
| `textures.rs` | 59 | Texture loading utilities |
| `bloom.rs` | ~200 | Bloom post-processing compute shaders |
| `composite.rs` | ~100 | Final HDR→SDR compositing |

#### Key Fields (in Renderer struct)

| Field | Type | Purpose |
|-------|------|---------|
| `instance` | `ash::Instance` | Vulkan instance |
| `device` | `ash::Device` | Logical device |
| `physical_device` | `vk::PhysicalDevice` | GPU handle |
| `swapchain` | `vk::SwapchainKHR` | Presentation swapchain |
| `swapchain_extent` | `vk::Extent2D` | Window dimensions |
| `render_pass` | `vk::RenderPass` | Main render pass |
| `pipeline` | `vk::Pipeline` | Meshlet graphics pipeline |
| `mesh_buffers` | `MeshBuffers` | GPU mesh data |
| `skinned_actors` | `Vec<SkinnedActor>` | Animated characters |

#### Key Methods (mod.rs)

| Method | Description |
|--------|-------------|
| `new(window, ...)` | Complete Vulkan initialization |
| `init_swapchain(...)` | Create/recreate swapchain |
| `init_pipeline(...)` | Create meshlet graphics pipeline |
| `recreate_swapchain()` | Handle window resize |
| `record_command_buffer(...)` | Build frame's GPU commands |
| `update_models(...)` | Load new scene meshlets |
| `update_skinned(dt)` | Advance skeletal animation |
| `set_skinned_actors(...)` | Load animated characters |

#### Submodule: skinned.rs

| Function | Description |
|----------|-------------|
| `build_skinned_vertices(mesh)` | Convert animation mesh to GPU vertices |
| `build_skinned_meshlets(mesh)` | Generate meshlets from skinned mesh |
| `resolve_gltf_texture(path)` | Resolve texture path for glTF models |
| `create_skinned_actor(...)` | Load and build GPU resources for character |
| `destroy_skinned_actor(...)` | Clean up skinned actor GPU resources |

#### Submodule: init.rs

| Function | Description |
|----------|-------------|
| `init_window(event_loop, fullscreen)` | Create game window with cursor grab |
| `init_instance(window)` | Create Vulkan instance and surface |
| `init_swapchain(...)` | Create swapchain and image views |
| `init_pipeline(...)` | Create meshlet graphics pipeline |
| `init_skinned_pipeline(...)` | Create skinned mesh pipeline |

#### Submodule: hiz.rs

| Function | Description |
|----------|-------------|
| `create_hiz_pipeline(device)` | Create Hi-Z compute pipeline |
| `create_hiz_sampler(device)` | Create nearest-neighbor sampler |
| `create_hiz_image_and_views(...)` | Create Hi-Z pyramid image and mip views |
| `destroy_hiz_image_resources(...)` | Clean up Hi-Z image resources |
| `free_hiz_descriptor_sets(...)` | Free descriptor sets before recreation |

#### Submodule: textures.rs

| Function | Description |
|----------|-------------|
| `load_textures(...)` | Load default textures (concrete, grass, tree, rock) |
| `set_skybox()` | Get default skybox color |

---

### Player System

**File:** `src/engine/player.rs` (108 lines)

#### Player Struct

Represents the first-person camera and movement state.

```rust
pub struct Player {
    pub position: Vec3,    // Feet position
    pub velocity: Vec3,    // Current velocity
    pub yaw: f32,          // Horizontal rotation (radians)
    pub pitch: f32,        // Vertical rotation (radians)
    pub on_ground: bool,   // Grounded state
    pub eye_height: f32,   // Camera height above feet (1.6m)
}
```

#### Methods

| Method | Description |
|--------|-------------|
| `new(feet_position)` | Create player at position |
| `eye_position()` | Get camera world position |
| `add_look(dx, dy, sensitivity)` | Rotate camera with clamping |
| `view_axes()` | Get (forward, right, up) vectors |

#### PlayerState Struct

Tracks gameplay state (health, inventory).

```rust
pub struct PlayerState {
    pub health: f32,
    pub max_health: f32,
    pub inventory: Vec<InventoryItem>,
    pub equipped_item: Option<String>,
}
```

---

### Physics System

**Directory:** `src/engine/physics/` (3 files, ~330 lines)

Implements player movement with capsule-based collision detection.

#### Files

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | 3 | Module exports |
| `capsule.rs` | ~150 | Capsule collision utilities |
| `player_physics.rs` | ~180 | Player movement and collision |

#### `step()` Function (player_physics.rs)

Main physics simulation step:

1. **Noclip Mode**: Free-fly movement if enabled
2. **Input Processing**: Convert WASD to wish direction
3. **Acceleration**: Apply ground/air acceleration (Quake-style)
4. **Friction**: Apply ground friction when stationary
5. **Gravity**: Apply downward acceleration
6. **Jump**: Apply vertical impulse if grounded
7. **Integration**: Move position by velocity × dt
8. **Capsule Collision**: Build player capsule, test against AABBs
9. **Iterative Resolution**: 4 iterations for stable collision handling
10. **Step-Up**: Auto-climb small obstacles (< 0.5m)
11. **Wall Sliding**: Cancel velocity in push direction
12. **Ground Check**: Update `on_ground` state

#### Capsule Collision (capsule.rs)

```rust
pub struct Capsule {
    pub bottom: Vec3,  // Bottom sphere center
    pub top: Vec3,     // Top sphere center
    pub radius: f32,   // Capsule radius
}
```

| Function | Description |
|----------|-------------|
| `Capsule::from_feet(pos, height, radius)` | Create capsule from feet position |
| `closest_point_on_segment(p, a, b)` | Find closest point on line segment |
| `capsule_aabb_penetration(capsule, aabb)` | Calculate push direction and depth |
| `capsule_aabb_sweep(capsule, velocity, aabb, dt)` | Sweep test for continuous collision |

#### Collision Algorithm

- Player represented as vertical **capsule** (radius 0.3m, height 1.7m)
- Penetration calculated as distance from capsule spine to AABB surface
- **Iterative resolution**: 4 passes ensure stable multi-collision handling
- Push direction normalized and used for wall sliding
- Special handling for **ramps** (interpolated height based on Z position)

---

### ECS Components

**File:** `src/engine/components.rs` (258 lines)

Components for the hecs ECS system.

| Component | Fields | Purpose |
|-----------|--------|---------|
| `Position` | `Vec3` | World position |
| `Rotation` | `Vec3` | Euler angles |
| `Scale` | `f32` | Uniform scale |
| `Velocity` | `Vec3` | Movement velocity |
| `StaticMesh` | `model_id, offset` | Reference to renderer mesh |
| `SkinnedMesh` | `resource_id` | Reference to animated mesh |
| `PhysicsBody` | `radius, active, on_ground, base_position` | Physics simulation |
| `AiAgent` | `state, current_task, wander_radius, speed` | AI behavior |
| `Interactable` | `action_name` | Pickup/use actions |

#### PhysicsBody Methods

| Method | Description |
|--------|-------------|
| `step(...)` | Subdivided physics simulation |
| `step_once(...)` | Single integration step with collision |

---

### AI System

**File:** `src/engine/ai/mod.rs` (111 lines)

#### AIState Struct

```rust
pub struct AIState {
    pub position: Vec3,
    pub forward: Vec3,
}
```

#### WalkTask Struct

Represents a movement goal with callbacks.

```rust
pub struct WalkTask {
    pub target: Vec3,
    pub speed: f32,
    pub reach_radius: f32,
    pub started: bool,
    pub on_start: Option<Box<dyn FnMut(&mut AIState)>>,
    pub on_reach: Option<Box<dyn FnMut(&mut AIState)>>,
}
```

| Method | Description |
|--------|-------------|
| `new(target, speed, radius, ...)` | Create walk task |
| `step(ai, dt)` | Advance toward target, returns `true` when reached |

#### `check_fov_and_notify()`

Utility for AI vision checks:
- Checks if target is within FOV cone and max distance
- Invokes callback when target spotted

---

## Environment Systems

### Weather System

**File:** `src/engine/environment/weather.rs` (101 lines)

Dynamic day/night cycle with procedural sun positioning.

#### Sun Struct

```rust
pub struct Sun {
    pub position: Vec3,      // World position
    pub direction: Vec3,     // Direction to origin
    pub color: [f32; 3],     // RGB color
    pub intensity: f32,      // Light intensity
}
```

#### Weather::update(time)

Updates sun based on 24-hour time:
- **06:00-20:00 (Day)**: Sun rises from east, peaks at noon, sets west
- **20:00-06:00 (Night)**: Sun below horizon, intensity = 0
- **Sunrise/Sunset**: Orange tint when sun is low
- **Sky Color**: Interpolated between night (dark blue) and day (bright blue)

---

### Cloud Renderer

**File:** `src/engine/environment/clouds.rs` (226 lines)

Volumetric clouds rendered via mesh shader raymarching.

#### Pipeline

- **Mesh Shader**: Generates fullscreen quad at far plane
- **Fragment Shader**: Raymarches through cloud volume using 3D noise
- **Blending**: Alpha blending over scene geometry

#### Push Constants

```rust
pub struct CloudPushConstants {
    pub camera_pos: [f32; 4],
    pub sun_dir: [f32; 4],
    pub sun_color: [f32; 4],
    pub inv_view_proj: Mat4,
    pub mvp: Mat4,
    pub wind_params: [f32; 4],
    pub time: f32,
    pub cloud_scale: f32,
    pub cloud_density: f32,
    pub cloud_absorption: f32,
    pub min_height: f32,
    pub max_height: f32,
}
```

---

### Rain Renderer

**File:** `src/engine/environment/rain.rs` (241 lines)

GPU particle rain system using mesh shaders.

#### Features

- Procedural rain drop placement (no CPU particle system)
- Wind influence on drop trajectory
- Configurable intensity, drop count, spawn radius
- Alpha blending for translucent drops

#### Push Constants

```rust
pub struct RainPushConstants {
    pub view_proj: Mat4,
    pub camera_time: [f32; 4],    // xyz = camera, w = time
    pub wind_params: [f32; 4],    // direction, speed, turbulence
    pub spawn_params: [f32; 4],   // radius, height, ground, fall_speed
    pub drop_params: [f32; 4],    // length, width, alpha, pad
}
```

---

### Fire Renderer

**File:** `src/engine/environment/fire.rs` (391 lines)

Real-time GPU fluid simulation for fire and smoke.

#### Architecture

1. **3D Volume Texture** (64³): Stores density/temperature/velocity
2. **Compute Shader** (`fire_sim.comp`): Eulerian fluid simulation
3. **Mesh Shader** (`fire.mesh`): Generates billboard at fire position
4. **Fragment Shader** (`fire.frag`): Raymarches volume with emission

#### Update Loop

```rust
pub fn update(&mut self, device, cmd, dt, time)
```
- Dispatches compute shader for simulation step
- Advects velocity and density fields
- Applies buoyancy and heat dissipation

---

### Stars Renderer

**File:** `src/engine/environment/stars.rs` (lines vary)

Procedural star field for night sky.

- Mesh shader generates star points
- Brightness modulated by time of day
- Random distribution using noise function

---

### Snow Renderer

**File:** `src/engine/environment/snow.rs` (240 lines)

GPU particle snow system using mesh shaders, similar to rain but with different physics.

#### Features

- Procedural snowflake placement (no CPU particle system)
- Wind influence with turbulence effects
- Configurable intensity, flake count, spawn radius
- Alpha blending for semi-transparent flakes
- Slower fall speed and more chaotic motion than rain

#### Push Constants

```rust
pub struct SnowPushConstants {
    pub view_proj: Mat4,
    pub camera_time: [f32; 4],    // xyz = camera, w = time
    pub wind_params: [f32; 4],    // direction, speed, turbulence
    pub spawn_params: [f32; 4],   // radius, height, ground, fall_speed
    pub flake_params: [f32; 4],   // size, pad, alpha_scale, pad
}
```

---

### Sun Sphere Renderer

**File:** `src/engine/environment/sun_sphere.rs` (lines vary)

Renders the sun disc as a billboard.

- Position tracks Weather::sun.position
- Glow effect with radial falloff
- Hidden when below horizon

---

## GUI Systems

### Text Renderer

**File:** `src/engine/gui/text.rs` (1017 lines)

Dynamic text and image rendering for UI.

#### TextInstance Struct

Each text element has its own GPU resources:

```rust
pub struct TextInstance {
    pub text: String,
    pub position: [f32; 2],
    pub vertex_buffer: vk::Buffer,
    pub vertex_count: u32,
    pub texture: Texture,
    pub descriptor_set: vk::DescriptorSet,
    pub action: Option<String>,
    pub bounds: [f32; 4],
}
```

#### Key Methods

| Method | Description |
|--------|-------------|
| `new(device, render_pass, font_path, msaa)` | Initialize renderer |
| `add_text(...)` | Create new text instance |
| `add_image(...)` | Create image sprite instance |
| `replace_text(slot, ...)` | Update existing text |
| `remove_text(idx, ...)` | Delete text instance |
| `record_commands(...)` | Render all instances |
| `record_commands_subset(...)` | Render specific instances |

#### Text Rasterization

Uses rusttype to rasterize TTF glyphs to greyscale texture:
1. Calculate bounding box for text
2. Rasterize each glyph to bitmap
3. Upload to GPU texture
4. Generate quad vertices with UVs

---

### HUD Renderer

**File:** `src/engine/gui/hud.rs` (471 lines)

In-game heads-up display.

#### Elements

- **Health Bar**: Horizontal bar with background and foreground
- **Inventory Panel**: Grid of item slots when inventory open

#### Constants

```rust
pub const INV_ROWS: usize = 4;
pub const INV_COLS: usize = 6;
pub const INV_SLOT_SIZE: f32 = 0.15;
pub const INV_SLOT_GAP: f32 = 0.02;
```

---

### Menu Renderer

**File:** `src/engine/gui/menu.rs` (396 lines)

Pause menu overlay with panels and slots.

- Dark semi-transparent background panel
- Header bar with gradient
- Interactive slot buttons

---

### Crosshair System

**File:** `src/engine/gui/crosshair.rs` (lines vary)

Configurable crosshair styles:
- **Dot**: Simple center point
- **Bars/Cross**: Traditional crosshair lines

---

## State & Configuration

### State Module

**File:** `src/engine/state/mod.rs` (63 lines)

#### Region Enum

Spatial volumes for triggers:

```rust
pub enum Region {
    Sphere { center: [f32; 3], radius: f32 },
    Aabb { min: [f32; 3], max: [f32; 3] },
}
```

#### FirePolicy Enum

```rust
pub enum FirePolicy {
    Once,   // Trigger fires once
    Repeat, // Trigger fires every entry
}
```

#### GameState Struct

```rust
pub struct GameState {
    pub flags: HashSet<String>,   // Set flags
    pub fired: HashSet<String>,   // Fired once-triggers
}
```

---

### Config Module

**File:** `src/engine/state/config.rs` (282 lines)

RON configuration structures.

#### GameStateConfig

Top-level configuration:

```rust
pub struct GameStateConfig {
    pub initial_time: f32,
    pub day_cycle_duration: f32,
    pub crosshair: CrosshairConfig,
    pub player_config: PlayerConfig,
    pub default_scene: String,
    pub clouds: CloudConfig,
    pub wind: WindConfig,
    pub rain: RainConfig,
    pub lighting: LightingConfig,
    pub scene_dir: String,
}
```

#### Scene

Scene definition:

```rust
pub struct Scene {
    pub models: Vec<ModelConfig>,
    pub ai: Vec<AIConfig>,
    pub triggers: Vec<TriggerConfig>,
    pub gui: Vec<GuiElementConfig>,
    pub wind: Option<WindConfig>,
}
```

#### ModelConfig

Individual model configuration:

```rust
pub struct ModelConfig {
    pub path: String,
    pub offset: [f32; 3],
    pub scale: f32,
    pub material_id: f32,
    pub collision: bool,
    pub collider_type: String,
    pub rigid_body: bool,
    pub interactable: Option<String>,
    pub opacity: f32,
    pub material: Option<MaterialProps>,
}
```

---

## SDF Rendering

**File:** `src/engine/sdf/loader.rs` (129 lines)

Signed Distance Field model builder for raymarched geometry.

#### SdfOp Enum

Operations for constructive solid geometry:

```rust
pub enum SdfOp {
    Sphere { radius, center, color },
    Box { half_extents, center, color },
    Cylinder { height, radius, center, color },
    Torus { thickness, radius, center, color },
    Union,
    Subtract,
    Intersect,
    SmoothUnion { k },
    SmoothSubtract { k },
    SmoothIntersect { k },
}
```

#### SdfModel Builder

Fluent builder pattern:

```rust
let model = SdfModel::new()
    .sphere(center, radius, color)
    .box_shape(center, half_extents, color)
    .smooth_union(0.5);
```

The model is serialized to a flat float array for GPU consumption.

---

## Vulkan Abstraction Layer

### vk_instance.rs

| Function | Description |
|----------|-------------|
| `create_instance(window)` | Create Vulkan instance with extensions |
| `create_surface(entry, instance, window)` | Create window surface |

### vk_device.rs

| Function | Description |
|----------|-------------|
| `pick_physical_device(...)` | Select GPU with required features |
| `create_logical_device(...)` | Create device with mesh shader extension |
| `find_queue_families(...)` | Find graphics/present queue families |

### vk_swapchain.rs

| Function | Description |
|----------|-------------|
| `create_swapchain(...)` | Create presentation swapchain |
| `choose_surface_format(...)` | Select SRGB format |
| `choose_present_mode(...)` | Select FIFO/Mailbox mode |

### vk_render_pass.rs

| Function | Description |
|----------|-------------|
| `create_render_pass(...)` | Create render pass with MSAA resolve |

### vk_pipeline.rs

| Function | Description |
|----------|-------------|
| `compile_shader(path, kind)` | Compile GLSL to SPIR-V at runtime |
| `create_pipeline(...)` | Create meshlet graphics pipeline |
| `create_skinned_pipeline(...)` | Create skeletal mesh pipeline |
| `create_outline_pipeline(...)` | Create inverted hull outline pipeline |
| `create_hiz_pipeline(...)` | Create Hi-Z compute pipeline |
| `create_compute_pipeline(...)` | Generic compute pipeline creation |
| `create_descriptor_set_layout(...)` | Create meshlet descriptor layout |

### vk_memory.rs

| Function | Description |
|----------|-------------|
| `find_memory_type(...)` | Find suitable memory type |
| `find_depth_format(...)` | Find supported depth format |
| `create_image(...)` | Create VkImage + memory |
| `create_image_with_samples(...)` | Create MSAA image |
| `create_image_mips(...)` | Create mipmapped image |
| `create_image_3d(...)` | Create 3D volume texture |
| `create_depth_resources(...)` | Create depth buffer |

### vk_buffers.rs

| Function | Description |
|----------|-------------|
| `create_buffer(...)` | Create buffer with optional data |
| `copy_buffer(...)` | Copy between buffers |
| `create_device_local_buffer_with_data(...)` | Staged upload to GPU |

### vk_textures.rs

| Function | Description |
|----------|-------------|
| `load_texture(path)` | Load image file to GPU texture |
| `transition_image_layout(...)` | Change image layout |
| `copy_buffer_to_image(...)` | Upload staging buffer to image |
| `create_storage_image_3d(...)` | Create 3D storage texture |

### vk_meshlets.rs

| Function | Description |
|----------|-------------|
| `load_model_obj(models)` | Load OBJ files, extract AABBs |
| `create_resources(...)` | Generate meshlets, upload to GPU |

#### MeshBuffers Struct

GPU resources for all scene meshlets:

```rust
pub struct MeshBuffers {
    pub meshlet_buffer: vk::Buffer,
    pub meshlet_count: u32,
    pub vertex_buffer: vk::Buffer,
    pub index_buffer: vk::Buffer,
    pub descriptor_set: vk::DescriptorSet,
    // ... memory handles
}
```

### vk_sync.rs

| Function | Description |
|----------|-------------|
| `create_semaphores(device, count)` | Create synchronization semaphores |
| `create_fences(device, count, signaled)` | Create CPU-GPU fences |

---

## Shader Reference

| Shader | Type | Purpose |
|--------|------|---------|
| `meshlet.task` | Task | Meshlet frustum/occlusion culling |
| `meshlet.mesh` | Mesh | Meshlet vertex fetch and primitive output |
| `meshlet.frag` | Fragment | PBR shading with shadows |
| `cloud.mesh` | Mesh | Generate fullscreen cloud quad |
| `cloud.frag` | Fragment | Volumetric cloud raymarching |
| `rain.mesh` | Mesh | Generate rain drop billboards |
| `rain.frag` | Fragment | Rain drop shading |
| `fire.mesh` | Mesh | Fire volume billboard |
| `fire.frag` | Fragment | Fire volume raymarching |
| `fire_sim.comp` | Compute | Fluid dynamics simulation |
| `stars.mesh` | Mesh | Star point generation |
| `stars.frag` | Fragment | Star rendering |
| `sun_sphere.vert/frag` | Vertex/Fragment | Sun disc billboard |
| `sdf.vert/frag` | Vertex/Fragment | SDF raymarching |
| `hiz.comp` | Compute | Hi-Z pyramid generation |
| `bloom_down.comp` | Compute | Bloom downsampling |
| `bloom_up.comp` | Compute | Bloom upsampling |
| `composite.vert/frag` | Vertex/Fragment | Final image compositing |
| `text.vert/frag` | Vertex/Fragment | UI text rendering |
| `menu.vert/frag` | Vertex/Fragment | Menu panel rendering |
| `fox.mesh` | Mesh | Skinned mesh shader |
| `fox.frag` | Fragment | Skinned mesh fragment |

---

## File-by-File Function Documentation

### src/main.rs

```rust
fn main()
```
**Purpose:** Application entry point.

**Flow:**
1. Initialize logging (`env_logger::init()`)
2. Create winit event loop
3. Configure MSAA, fullscreen, crosshair
4. Create `Engine` instance
5. Optionally skip menu (`load_scene("scene2")`)
6. Lock cursor to window
7. Enter event loop:
   - Handle `CloseRequested` → exit
   - Handle `Resized` → `engine.handle_resize()`
   - Handle `MouseMotion` → `engine.handle_mouse_delta()`
   - Handle `MouseInput` → `engine.handle_click()`
   - Handle `KeyboardInput` → update `InputState` / toggle features
   - Handle `RedrawRequested` → `engine.render(&input)`

---

### src/engine/mod.rs

#### `Engine::new(...)`
Creates the engine, initializing all subsystems.

#### `Engine::window()`
Returns reference to the winit Window.

#### `Engine::handle_resize()`
Marks framebuffer as needing recreation.

#### `Engine::handle_mouse_delta(dx, dy)`
Rotates player camera based on mouse movement.

#### `Engine::toggle_menu()`
Toggles pause menu visibility.

#### `Engine::set_debug(enabled)`
Shows/hides debug overlay text.

#### `Engine::set_crosshair_enabled(enabled)`
Shows/hides crosshair.

#### `Engine::set_bloom_enabled(enabled)`
Enables/disables bloom post-processing.

#### `Engine::handle_click(x, y)`
Dispatches click to menu, inventory, or game world.

#### `Engine::handle_inventory_click(x, y)`
Processes click on inventory slot, equips items.

#### `Engine::handle_game_click()`
Processes in-game click (place item, etc).

#### `Engine::update_equipped_text()`
Updates the "Equipped: X" UI text.

#### `Engine::sync_inventory_ui()`
Synchronizes inventory icon sprites with player inventory.

#### `Engine::rebuild_overlays()`
Recreates UI elements after resize.

#### `Engine::load_scene(name)`
Loads scene from RON file:
1. Parse scene file
2. Clear ECS world
3. Spawn model entities
4. Update renderer meshlets
5. Build interactables map
6. Load triggers
7. Update GUI elements
8. Load AI actors
9. Reset player if game scene
10. Load weather/environment

#### `Engine::update_camera(input, dt)`
Per-frame camera/player update:
1. Build scratch collider list
2. Run player physics
3. Push physics bodies with player
4. Raycast for interaction target
5. Show/hide interaction text
6. Handle pickup/drop
7. Update held item position
8. Build view/projection matrices
9. Process trigger zones
10. Return `FrameInput` for renderer

#### `Engine::render(input)`
Main render loop (see startup flow diagram).

#### `Engine::set_sun_enabled(enabled)`
Toggles sun/weather rendering.

#### `Engine::process_triggers(eye)`
Checks all triggers against player position, fires actions.

#### `Engine::random_task(radius, speed)`
Generates random AI walk task.

#### `Engine::update_debug_overlay()`
Updates FPS and position debug text.

#### `Engine::toggle_inventory()`
Opens/closes inventory panel.

#### `Engine::is_interface_open()`
Returns true if menu or inventory is open.

---

### src/engine/renderer/

The renderer is split into focused submodules (see Renderer section above for details).

#### mod.rs - Core Renderer
- `Renderer::new(...)` - Complete Vulkan initialization
- `Renderer::record_command_buffer(...)` - Build frame commands
- `Renderer::recreate_swapchain()` - Handle resize
- `Renderer::update_models(...)` - Load scene meshlets
- `Renderer::update_skinned(dt)` - Advance animations
- `Renderer::update_model_offsets(...)` - Physics integration

#### skinned.rs - Animated Actors
- `build_skinned_vertices(mesh)` - Convert to GPU format
- `build_skinned_meshlets(mesh)` - Generate meshlets
- `create_skinned_actor(...)` - Load character
- `destroy_skinned_actor(...)` - Cleanup

#### init.rs - Initialization
- `init_window(...)` - Window creation
- `init_instance(...)` - Vulkan setup
- `init_swapchain(...)` - Swapchain creation
- `init_pipeline(...)` - Pipeline creation

#### hiz.rs - Hi-Z Pyramid
- `create_hiz_pipeline(...)` - Compute pipeline
- `create_hiz_image_and_views(...)` - Pyramid creation
- `destroy_hiz_image_resources(...)` - Cleanup

#### textures.rs - Texture Loading
- `load_textures(...)` - Load default textures
- `set_skybox()` - Get skybox color

---

### src/engine/physics/player_physics.rs

#### `step(player, input, cfg, colliders, dt)`
Main physics simulation (documented in Physics section).

---

### src/engine/components.rs

#### `PhysicsBody::step(...)`
Subdivides large dt into MAX_STEP increments.

#### `PhysicsBody::step_once(...)`
Single physics step with gravity, collision, friction.

#### `resolve_sphere_aabb(pos, vel, radius, aabb)`
Sphere-AABB collision detection and resolution.

#### `resolve_player_push(pos, vel, radius, player, push_dir)`
Pushes physics body away from player when player walks into it.

---

### src/engine/ai/mod.rs

#### `AIState::new(position, forward)`
Create AI state at position facing direction.

#### `WalkTask::new(...)`
Create walk task with target, speed, callbacks.

#### `WalkTask::step(ai, dt)`
Advance AI toward target, returns true when reached.

#### `check_fov_and_notify(...)`
Check if target in AI's field of view and distance.

---

### src/engine/state/mod.rs

#### `Region::contains(point)`
Test if point is inside region (sphere or AABB).

#### `load_triggers(path)`
Load triggers from RON file.

---

### src/engine/state/config.rs

#### `GameStateConfig::load(path)`
Load main config from RON file.

#### `Scene::load(path)`
Load scene from RON file.

---

### src/engine/environment/weather.rs

#### `load_weather_scene()`
Create default Weather with initial sun position.

#### `default_sun()`
Create sun at 11:00 position.

#### `derive_sky_color(sun)`
Calculate sky color from sun position.

#### `Weather::update(time)`
Update sun position and sky color for given time of day.

---

### src/engine/environment/clouds.rs

#### `CloudRenderer::new(...)`
Create cloud pipeline.

#### `CloudRenderer::destroy(device)`
Cleanup GPU resources.

#### `CloudRenderer::record_commands(...)`
Record cloud render commands.

#### `create_pipeline(...)`
Create mesh shader pipeline for clouds.

---

### src/engine/environment/rain.rs

Same pattern as clouds:
- `RainRenderer::new(...)`
- `RainRenderer::destroy(device)`
- `RainRenderer::record_commands(...)`

---

### src/engine/environment/fire.rs

#### `FireRenderer::new(...)`
Create 3D volume texture, compute pipeline, render pipeline.

#### `FireRenderer::update(device, cmd, dt, time)`
Dispatch fluid simulation compute shader.

#### `FireRenderer::draw(...)`
Record fire volume rendering commands.

#### `FireRenderer::destroy(device)`
Cleanup all resources.

---

### src/engine/gui/text.rs

#### `TextRenderer::new(...)`
Create text pipeline, load font.

#### `TextRenderer::build_instance(...)`
Rasterize text to texture, create GPU resources.

#### `TextRenderer::build_image_instance(...)`
Load image file as sprite.

#### `TextRenderer::add_text(...)`
Add new text to render list.

#### `TextRenderer::remove_text(idx, ...)`
Remove text instance.

#### `TextRenderer::record_commands(...)`
Render all text instances.

#### `rasterize_text(font, text, px)`
Convert text to greyscale bitmap.

#### `upload_image(...)`
Upload bitmap to GPU texture.

---

### src/engine/gui/hud.rs

#### `HudRenderer::new(...)`
Create health bar and inventory panel geometry.

#### `HudRenderer::record_commands(...)`
Render HUD with current health percentage.

#### Helper functions for geometry:
- `create_health_bar_bg(...)`
- `create_health_bar_fg(...)`
- `create_inv_panel(...)`
- `create_inv_slots(...)`

---

### src/engine/gui/menu.rs

#### `MenuRenderer::new(...)`
Create menu panel geometry.

#### `MenuRenderer::record_commands(...)`
Render menu overlay.

---

### src/engine/sdf/loader.rs

#### `SdfModel::new()`
Create empty model.

#### `SdfModel::sphere(...)`
Add sphere primitive.

#### `SdfModel::box_shape(...)`
Add box primitive.

#### `SdfModel::cylinder(...)`
Add cylinder primitive.

#### `SdfModel::torus(...)`
Add torus primitive.

#### `SdfModel::union()`
Union of top two stack elements.

#### `SdfModel::subtract()`
Subtract top from second stack element.

#### `SdfModel::smooth_union(k)`
Smooth minimum of top two elements.

---

### src/vulkan/vk_buffers.rs

#### `create_buffer(...)`
Create VkBuffer with memory, optionally initialize data.

#### `begin_single_time_commands(...)`
Start one-time command buffer.

#### `end_single_time_commands(...)`
End and submit one-time commands.

#### `copy_buffer(...)`
GPU buffer-to-buffer copy.

#### `create_device_local_buffer_with_data(...)`
Staged upload: CPU → staging → device local.

---

### src/vulkan/vk_memory.rs

#### `find_memory_type(...)`
Find memory type index matching requirements.

#### `find_supported_format(...)`
Find format with required features.

#### `find_depth_format(...)`
Find suitable depth/stencil format.

#### `create_image(...)`
Create 2D image with memory.

#### `create_image_with_samples(...)`
Create MSAA image.

#### `create_image_mips(...)`
Create image with mipmap levels.

#### `create_image_3d(...)`
Create 3D volume texture.

#### `create_depth_resources(...)`
Create depth buffer with image view.

---

### src/vulkan/vk_meshlets.rs

#### `Aabb::new(min, max, collider_type)`
Create AABB with collision type.

#### `Aabb::ray_intersect(origin, direction)`
Slab method ray-AABB intersection.

#### `Aabb::inflated(margin)`
Expand AABB by margin.

#### `load_model_obj(models)`
Load OBJ files, apply transforms, extract collision AABBs.

#### `create_resources(...)`
Generate meshlets from vertices/indices, upload to GPU.

---

### src/vulkan/vk_pipeline.rs

#### `compile_shader(path, kind)`
Runtime GLSL→SPIR-V compilation using shaderc.

#### `create_descriptor_set_layout(...)`
Create layout for meshlet pipeline (textures, buffers).

#### `create_pipeline(...)`
Create meshlet task+mesh+fragment pipeline.

#### `create_outline_pipeline(...)`
Create inverted hull outline pipeline.

#### `create_skinned_descriptor_set_layout(...)`
Create layout for skinned mesh pipeline.

#### `create_skinned_pipeline(...)`
Create skeletal mesh pipeline.

#### `create_hiz_descriptor_set_layout(...)`
Create layout for Hi-Z compute.

#### `create_hiz_pipeline(...)`
Create Hi-Z pyramid compute pipeline.

#### `create_compute_pipeline(...)`
Generic compute pipeline creation.

---

### src/vulkan/vk_textures.rs

#### `load_texture(path)`
Load image file → staging buffer → VkImage.

#### `begin_one_time_commands(...)`
Start one-time command buffer.

#### `end_one_time_commands(...)`
Submit and wait for completion.

#### `transition_image_layout(...)`
Change image layout via pipeline barrier.

#### `copy_buffer_to_image(...)`
Copy staging buffer to image.

#### `create_storage_image_3d(...)`
Create 3D storage texture.

---

## Summary

The R-Rune engine is a sophisticated Vulkan 1.3 renderer featuring:

- **GPU-driven rendering** via task/mesh shaders for efficient meshlet processing
- **Modern graphics features** including volumetric clouds, GPU particle rain, and real-time fire simulation
- **First-person gameplay systems** with physics, inventory, and AI
- **Data-driven design** with RON configuration files
- **ECS architecture** for flexible entity management

The codebase is organized into clear subsystems with well-defined responsibilities, making it maintainable and extensible.
