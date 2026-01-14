# R-Rune Engine

A high-performance Vulkan-based game engine written in Rust, featuring Clustered Forward+ Lighting, Mesh Shaders, and procedural environment effects.

## Features
- **Modern Vulkan Pipeline**: Utilizes Mesh Shaders for geometric efficiency.
- **Clustered Forward+ Lighting**: Efficiently handles hundreds of point lights.
- **Atmospheric Effects**: Procedural volumetric clouds, rain, and photorealistic snow.
- **Physics & Interaction**: Custom rigid body physics and interaction system.

## Prerequisites
- **Rust**: [Install Rust](https://rustup.rs/) (2024 edition supported).
- **Vulkan SDK**: Ensure you have valid Vulkan drivers and the SDK installed.
- **Shader Compilation**: Requires `shaderc` dependencies (usually handled by the crate, but system libraries may be needed).

## Getting Started

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd r-rune
   ```

2. **Run the project**:
   ```bash
   cargo run --release
   ```
   *Note: Using `--release` is highly recommended for performance.*

## Controls
- **WASD**: Move
- **Shift**: Sprint
- **Space**: Jump
- **Mouse**: Look around
- **E**: Interact / Pick up item
- **F**: Toggle Inventory
- **G**: Drop held item
- **B**: Toggle Bloom
- **L**: Toggle Sun/Daylight
- **F3**: Toggle Debug Menu
- **Esc**: Exit

## Project Structure
- `assets/`: 3D models, textures, and scene configurations (`.ron`).
- `shader/`: Glsl shader source files (Mesh, Frag, Comp).
- `src/`: Rust source code.
- `src/vulkan/`: Low-level Vulkan abstraction layer.
- `src/engine/`: Core engine logic, renderer, and systems.
