# Modern Feature Suggestions

This captures the current list of modern engine features to consider next, grouped by area and
including likely integration points in the codebase.

## Rendering
- GPU-driven visibility and LOD: finish Hi-Z occlusion (currently forced visible) and add
  indirect meshlet dispatch + per-meshlet error/LOD data.
  - Likely touchpoints: shader/meshlet.task, src/vulkan/vk_meshlets.rs, src/engine/renderer/mod.rs
- PBR material pipeline (glTF-style): basecolor/normal/metallic/roughness/ao/emissive textures,
  correct sRGB handling, per-material texture indices.
  - Likely touchpoints: shader/meshlet.frag, src/vulkan/vk_pipeline.rs,
    src/vulkan/vk_textures.rs, src/vulkan/vk_meshlets.rs, src/engine/state/config.rs
- Multi-light + IBL: clustered/forward+ lighting, reflection probes, optional shadow maps or
  ray-traced soft shadows.
  - Likely touchpoints: shader/meshlet.frag, src/engine/renderer/mod.rs, src/engine/config.rs
- TAA + motion vectors + denoise: add velocity buffer + history reprojection for stable
  ray-query AO/shadows.
  - Likely touchpoints: src/engine/renderer/mod.rs, shader/composite.frag (new TAA pass)
- Upscaling (FSR2/DLSS): integrate after TAA or replace it in the composite stage.
  - Likely touchpoints: src/engine/renderer/composite.rs, src/engine/renderer/mod.rs
- Volumetrics/fog: depth-aware fog unified with clouds/rain/fire for atmosphere.
  - Likely touchpoints: src/engine/environment/clouds.rs, src/engine/environment/rain.rs

## Engine / Gameplay
- Streaming + scene graph: async asset loading, LOD streaming, and world chunking.
  - Likely touchpoints: src/engine/mod.rs (new asset manager module)
- Animation upgrades: blend trees, state machines, IK, root motion (and GPU skinning later).
  - Likely touchpoints: src/engine/animation/mod.rs, shader/fox.mesh
- Robust physics: integrate a full solver (Rapier) for rigid bodies/constraints/character controller.
  - Likely touchpoints: src/engine/physics/player_physics.rs, src/engine/components.rs
- Navmesh + AI behaviors: pathfinding + steering + behavior trees.
  - Likely touchpoints: src/engine/ai/mod.rs

## Tooling / Workflow
- Shader/config hot-reload, in-engine editor gizmos, framegraph, GPU/CPU profiling.
  - Likely touchpoints: src/vulkan/vk_pipeline.rs, src/engine/renderer/mod.rs
