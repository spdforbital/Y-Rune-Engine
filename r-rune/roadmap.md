# Roadmap

## Game Idea
The game is a high-stakes, multiplayer (plus bots) survival-strategy hybrid. A player character, clad in a hazmat suit, beams down to an Earth-like map. Every player begins equipped with a "Human 3D Printer." 

The core loop involves collecting resources to "print" humans. These printed humans serve as peasants—your primary workforce. You can command them to harvest resources, build structures, or train them into specialized combat troops. The game features a deep technological progression system:
- **Stone Age**: Basic survival and manual resource gathering.
- **Medieval Epoch**: Swords, armor, and tactical troop formations.
- **Modern Era**: Firearms, combustion engines, and early automation.
- **Future Era**: Plasma weapons, mass destruction tools, and advanced robotics.

**Combat & Maturity**: The game targets a realistic, mature, and gory aesthetic. Combat includes advanced animations for decapitation and visceral damage. 

**Survival & Environment**: You start as a character who cannot wear armor—only carry weapons. If you die, you are out (last player standing wins). The world is dynamic, featuring a day/night cycle and extreme environmental hazards like floods, tornadoes, thunderstorms, and volcanic eruptions. These disasters can physically alter the terrain, destroy buildings, and directly harm or impede you and your troops.

---

## Engine Features
Current capabilities implemented in the R-Rune engine:
* [x] **Vulkan Mesh Shader Pipeline**: Highly efficient rendering with task + mesh shaders for GPU-driven culling.
* [x] **Meshlet-Based Rendering**: Uses meshopt library for cluster generation, enabling massive geometry counts.
* [x] **Physically Based Rendering (PBR)**: Cook-Torrance BRDF with GGX distribution for realistic materials.
* [x] **Ray-Traced Shadows**: Real-time soft shadows via Vulkan ray queries (VK_EXT_ray_query).
* [x] **Ray-Traced Ambient Occlusion (RTAO)**: 4-sample hemisphere AO for grounded contact shadows.
* [x] **Hi-Z Occlusion Culling**: Hierarchical depth pyramid for GPU-driven visibility determination.
* [x] **Bloom Post-Processing**: Compute shader-based downsample/upsample with Gaussian blur.
* [x] **MSAA Anti-Aliasing**: Configurable 1x/2x/4x/8x multisample anti-aliasing.
* [x] **Dynamic Weather System**: Procedural sun positioning with 24-hour day/night cycle.
* [x] **Volumetric Clouds**: Ray-marched 3D noise clouds with wind offset and density controls.
* [x] **Fire & Fluid Simulation**: GPU-accelerated 3D Eulerian fluid dynamics for fire/smoke (64³ volume).
* [x] **Rain Particle System**: Mesh shader-based rain with wind influence and configurable intensity.
* [x] **Procedural Stars**: Noise-based star field renderer with time-of-day brightness modulation.
* [x] **Sun Disc Renderer**: Billboard sun with glow effects and horizon culling.
* [x] **SDF (Signed Distance Field) Rendering**: Fragment shader raymarching for procedural shapes.
* [x] **ECS (Entity Component System)**: Scalable entity management using `hecs` library.
* [x] **Player Physics**: WASD movement with gravity, friction, air control, and capsule-AABB collision.
* [x] **Capsule Collision System**: Player represented as vertical capsule with iterative penetration resolution.
* [x] **Step-Up Mechanic**: Automatic climbing of obstacles up to 0.5m height.
* [x] **Basic AI State Machine**: Task-based AI with walk goals, FOV checks, and callbacks.
* [x] **Skeletal Animation**: glTF skinned mesh support with GPU-accelerated skinning.
* [x] **GUI System**: Dynamic text rendering, HUD (health bar), pause menu, and inventory panel.
* [x] **Font Rendering**: Runtime TTF rasterization using `rusttype` crate.
* [x] **Crosshair System**: Configurable crosshair styles (dot, bars, circle).
* [x] **Interaction System**: Raycasting-based object interaction (pickup, use) with visual feedback.
* [x] **Inventory System**: 4x6 grid inventory with item equipping and held item rendering.
* [x] **Trigger Zones**: Spatial triggers (AABB/Sphere) with fire policies (once/repeat) and actions.
* [x] **Scene Loading**: RON-based scene definitions with models, AI agents, triggers, and GUI.
* [x] **Configurable Input**: JSON keybinds for movement, jump, sprint, and interactions.
* [x] **Wind Simulation**: Global wind system affecting clouds, rain, and particle effects.
* [x] **Inverted Hull Outlines**: Mesh expansion technique for object selection highlights.
* [x] **Model Caching**: Binary cache system for processed meshlet data to accelerate load times.
* [x] **Snow Particle System**: Mesh shader-based snow with wind and turbulence effects.
* [x] **Forward+ Clustered Lighting**: Efficient many-light rendering with 3D cluster grid and GPU culling.
* [x] **Scene Lights Configuration**: Point and spot lights configurable per-scene in RON files.
* [x] **Dynamic Item Pickup Names**: Model path-based item naming for inventory items.
* [x] **Rayon Parallelization**: Parallel model loading, meshlet generation, and collision detection.
* [x] **Scene-Driven Textures**: TextureManager with on-demand loading and path-based caching.
* [x] **Async Asset Loading**: Background asset loading via worker threads with channel communication.
* [x] **Firearm System**: Weapon firing, bullet tracers, bullet holes, and recoil mechanics.
* [x] **Weapon Sway**: Configurable camera and movement-based weapon sway effects.

---

## Engine Ideas
Tech plan for future development to support the game vision:

### 1. Multiplayer & Networking
- Implement a client-server architecture (likely using `quinn` or `renet`).
- Server-side physics authority for anti-cheat.
- Snapshot interpolation and client-side prediction for smooth hazmat suit movement.

### 2. Strategic "Peasant" AI
- **Pathfinding**: High-performance multi-threaded pathfinding (A* or Flow Fields) for large groups of printed humans.
- **Command System**: Group selection, "RTS-style" control, and battle tactics implementation.
- **Task Scheduling**: Queue-based worker tasks (harvesting, building).

### 3. Advanced Gore & Animation
- **Dismemberment System**: Procedural mesh slicing or bone-based decapitation logic in shaders.
- **Blood Decals**: persistent, ray-traced blood splatters that adhere to terrain and models.
- **IK (Inverse Kinematics)**: Proper foot placement for hazmat suits on uneven terrain.

### 4. Dynamic Terrain & Disasters
- **Deformable Terrain**: Heightmap-based or mesh deformation to change the map based on volcanic eruptions or explosions.
- **Weather Physics**: Tornado wind forces that affect entities and "print-human" movement.
- **Water Physics**: Flooding simulation that interacts with the heightmap.

### 5. Economy & Tech Progression
- **Resource Management**: System for tracking wood, stone, and futuristic materials.
- **Epoch System**: Logic for unlocking new building types and weapon tiers based on gathered resources.
- **3D Printer Interface**: Specialized UI for managing human printing queues.

### 6. Photorealistic Optimization
- **Hardware Ray Tracing Expansion**: Full global illumination (GI) and reflections.
- **DLSS/FSR Integration**: Upscaling to maintain high frame rates at photorealistic settings.
- **LOD System**: Automatic meshlet-based LODs for scaling to massive troop numbers.
- **Temporal Anti-Aliasing (TAA)**: Smooth edges and reduce flickering in motion.
- **Virtual Shadow Maps**: High-quality shadows for large open worlds.

### 7. Building & Construction System
- **Modular Building Toolkit**: Epoch-specific building components (stone huts → wooden structures → metal fortifications).
- **Structural Integrity**: Physics-based collapse if foundations are destroyed.
- **Build Preview**: Ghost/wireframe preview before placing buildings.
- **Snapping Grid**: Smart placement system for connecting structures.

### 8. Combat & Weapons
- **Weapon Switching**: Hotkey-based weapon inventory for the hazmat suit character.
- **Ballistics System**: Realistic bullet trajectories with wind and gravity effects.
- **Explosive Physics**: Dynamic destruction with debris and shockwaves.
- **Melee Combat**: Directional attacks (slash, stab, overhead) with collision detection.
- **Armor Penetration**: Damage calculation based on weapon tier vs armor tier.

### 9. Audio System
- **3D Spatial Audio**: HRTF-based positional sound using `kira` or `oddio`.
- **Environmental Reverb**: Dynamic reverb based on building interiors and terrain.
- **Combat Sounds**: Layered weapon sounds, impact effects, and death screams.
- **Ambient Soundscapes**: Day/night cycle audio, weather sounds, and wildlife.

### 10. Advanced Rendering Effects
- **Subsurface Scattering (SSS)**: Realistic skin shading for printed humans.
- **Motion Blur**: Per-object motion blur for fast-moving troops and projectiles.
- **Depth of Field**: Cinematic focus for dramatic moments.
- **God Rays**: Volumetric light shafts through clouds and trees.
- **Particle Systems**: GPU-driven particles for explosions, dust, and magic effects.

### 11. Automation & Transportation
- **Conveyor Belts**: Resource transport between buildings.
- **Vehicles**: Epoch-appropriate transport (carts → trucks → hover platforms).
- **Automated Turrets**: Defensive structures that target enemies autonomously.
- **Drones/Robots**: Late-game automated workers and scouts.

### 12. Procedural Generation
- **Resource Spawning**: Distributed ore veins, forests, and material nodes.
- **Map Variation**: Procedurally generated terrain with biomes (desert, tundra, jungle).
- **Building Destruction**: Procedural debris and ruin generation after disasters.

### 13. Performance & Scalability
- **GPU Culling**: Frustum, occlusion, and distance culling on compute shaders.
- **Instanced Rendering**: Batch rendering for identical troops and objects.
- **Async Asset Streaming**: Background loading for large maps without stuttering.
- **Multi-threaded ECS**: Parallel system execution for physics, AI, and rendering prep.

### 14. Save/Load & Persistence
- **Compressed Save Format**: Binary serialization of world state, buildings, and troops.
- **Auto-save**: Periodic checkpoints to prevent progress loss.
- **Replay System**: Record matches for post-game analysis.

### 15. UI/UX Enhancements
- **Minimap**: Real-time overhead view with fog of war.
- **Radial Menus**: Context-sensitive command wheels for quick actions.
- **Tech Tree Visualization**: Interactive diagram showing epoch progression.
- **Damage Numbers**: Floating combat text for feedback.
- **Objective Markers**: Waypoints and quest indicators.
