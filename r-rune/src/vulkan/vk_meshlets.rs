use ash::vk;
use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use meshopt::cluster;
use rayon::prelude::*;
use tobj;

use crate::vulkan::vk_buffers;

 
pub const FLOOR_Y_OFFSET: f32 = -0.5;
 

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct VertexData {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub data: [f32; 4],  
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Meshlet {
    pub vertex_offset: u32,
    pub triangle_offset: u32,
    pub vertex_count: u32,
    pub triangle_count: u32,
    pub center_radius: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct PushConstants {
    pub mvp: glam::Mat4,
    pub meshlet_count: u32,
    pub hovered_id: u32,
    pub outline_width: f32,
    pub meshlet_offset: u32,
    pub sun_dir: [f32; 4],       
    pub camera_pos: [f32; 4],
    pub sun_color: [f32; 4],
    pub screen_size: [u32; 2],
    pub _pad: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuMaterial {
    pub color: [f32; 4],  
    pub params: [f32; 4],  
    pub offset: [f32; 4],  
    pub base_position: [f32; 4],  
    pub rotation: [f32; 4],  
}


 
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColliderType {
    Solid,
    Stairs,
    Ladder,
    Ramp,
}

 
#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
    pub collider_type: ColliderType,
}

impl Aabb {
    pub fn new(min: Vec3, max: Vec3, collider_type: ColliderType) -> Self {
        Self { min, max, collider_type }
    }

     
     
    pub fn ray_intersect(&self, origin: Vec3, direction: Vec3) -> Option<f32> {
        let t1 = (self.min.x - origin.x) / direction.x;
        let t2 = (self.max.x - origin.x) / direction.x;
        let t3 = (self.min.y - origin.y) / direction.y;
        let t4 = (self.max.y - origin.y) / direction.y;
        let t5 = (self.min.z - origin.z) / direction.z;
        let t6 = (self.max.z - origin.z) / direction.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax >= tmin && tmax >= 0.0 {
            Some(tmin)
        } else {
            None
        }
    }
}

 
pub struct MeshletModelDesc {
     
    pub path: String,
    pub offset: [f32; 3],
    pub scale: f32,
    pub material_id: f32,
     
    pub collision_enabled: bool,
     
    pub collider_type: ColliderType,
    pub opacity: f32,
    pub color: [f32; 3],
    pub metalness: f32,
    pub roughness: f32,
}

pub struct MeshBuffers {
    pub meshlet_count: u32,
    pub meshlet_buffer: vk::Buffer,
    pub meshlet_buffer_memory: vk::DeviceMemory,
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub meshlet_vertices_buffer: vk::Buffer,
    pub meshlet_vertices_buffer_memory: vk::DeviceMemory,
    pub meshlet_triangles_buffer: vk::Buffer,
    pub meshlet_triangles_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub vertex_count: u32,
    pub index_count: u32,
    pub material_buffer: vk::Buffer,
    pub material_buffer_memory: vk::DeviceMemory,
    pub meshlet_params_buffer: vk::Buffer,
    pub meshlet_params_buffer_memory: vk::DeviceMemory,
    pub meshlet_ranges: Vec<(u32, u32)>,
    pub model_rt_infos: Vec<ModelRtInfo>,
}

#[derive(Clone, Copy, Debug)]
pub struct ModelRtInfo {
    pub index_start: u32,
    pub index_count: u32,
    pub vertex_start: u32,
    pub vertex_count: u32,
    pub base_position: Vec3,
}



 
 
fn load_model_obj(models: &[MeshletModelDesc]) -> (Vec<VertexData>, Vec<u32>, Vec<Aabb>, Vec<(u32, u32, usize)>) {
    let load_options = tobj::LoadOptions {
        triangulate: true,
        single_index: true,
        ..Default::default()
    };

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut colliders = Vec::new();
    let mut vertex_ranges = Vec::new();

    for (model_idx, model) in models.iter().enumerate() {
        println!("Loading OBJ: {} (offset={:?})", model.path, model.offset);
        let (objs, _) = tobj::load_obj(&model.path, &load_options).expect("Failed to load OBJ file");

        let mut instance_min = Vec3::splat(f32::MAX);
        let mut instance_max = Vec3::splat(f32::MIN);

        let model_start_vertex = vertices.len() as u32;
        println!("Models found: {}", objs.len());
        for m in objs.iter() {
            let mesh = &m.mesh;
            println!(
                "Mesh: {} vertices, {} indices",
                mesh.positions.len() / 3,
                mesh.indices.len()
            );
            let base_vertex = vertices.len() as u32;

             
            let mut min_p = [f32::MAX; 3];
            let mut max_p = [f32::MIN; 3];
            for i in 0..mesh.positions.len() / 3 {
                let px = mesh.positions[i * 3];
                let py = mesh.positions[i * 3 + 1];
                let pz = mesh.positions[i * 3 + 2];
                min_p[0] = min_p[0].min(px);
                min_p[1] = min_p[1].min(py);
                min_p[2] = min_p[2].min(pz);
                max_p[0] = max_p[0].max(px);
                max_p[1] = max_p[1].max(py);
                max_p[2] = max_p[2].max(pz);
            }
            let range_x = (max_p[0] - min_p[0]).max(1e-5);
            let range_z = (max_p[2] - min_p[2]).max(1e-5);

            for i in 0..mesh.positions.len() / 3 {
                 
                let px = mesh.positions[i * 3] * model.scale + model.offset[0];
                let py = mesh.positions[i * 3 + 1] * model.scale + model.offset[1];
                let pz = mesh.positions[i * 3 + 2] * model.scale + model.offset[2];
                
                let p_vec = Vec3::new(px, py, pz);
                instance_min = instance_min.min(p_vec);
                instance_max = instance_max.max(p_vec);

                let uv = if mesh.texcoords.len() >= (i * 2 + 2) {
                    [
                        mesh.texcoords[i * 2],
                        1.0 - mesh.texcoords[i * 2 + 1],
                    ]
                } else if model.material_id > 0.5 {
                    [px * 0.1, pz * 0.1]
                } else {
                    [(px - min_p[0]) / range_x, (pz - min_p[2]) / range_z]
                };

                let nx = if mesh.normals.len() >= (i * 3 + 3) {
                    mesh.normals[i * 3]
                } else { 0.0 };
                let ny = if mesh.normals.len() >= (i * 3 + 3) {
                    mesh.normals[i * 3 + 1]
                } else { 1.0 };
                let nz = if mesh.normals.len() >= (i * 3 + 3) {
                    mesh.normals[i * 3 + 2]
                } else { 0.0 };

                vertices.push(VertexData {
                    pos: [px, py, pz],
                    normal: [nx, ny, nz],
                    uv,
                    data: [model.material_id, model.opacity, model_idx as f32, 0.0],
                });
            }
            for idx in &mesh.indices {
                indices.push(idx + base_vertex);
            }
        }
        
        if model.collision_enabled {
             colliders.push(Aabb::new(instance_min, instance_max, model.collider_type));
             println!("Added collider ({:?}): min={:?}, max={:?}", model.collider_type, instance_min, instance_max);
        }
        
         
        vertex_ranges.push((model_start_vertex, (vertices.len() as u32) - model_start_vertex, model_idx));
    }

    (vertices, indices, colliders, vertex_ranges)
}

#[allow(clippy::too_many_arguments)]
 
 
 
 
 
 
pub unsafe fn create_resources(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    submit_queue: vk::Queue,
    models: &[MeshletModelDesc],
) -> (MeshBuffers, Vec<Aabb>) {
    let load_options = tobj::LoadOptions {
        triangulate: true,
        single_index: true,
        ..Default::default()
    };
    
    // Structure to hold per-model processing results
    struct ModelResult {
        model_idx: usize,
        vertices: Vec<VertexData>,
        indices: Vec<u32>,
        collider: Option<Aabb>,
        meshlets: Vec<crate::vulkan::vk_meshlets::Meshlet>,
        meshlet_vertices: Vec<u32>,  // Local indices into model_vertices
        meshlet_triangles: Vec<u32>,
        meshlet_model_indices: Vec<u32>,
        base_position: Vec3,
    }
    
    // Process each model in parallel
    let model_results: Vec<ModelResult> = models
        .par_iter()
        .enumerate()
        .map(|(model_idx, model)| {
            println!("Loading OBJ: {} (offset={:?})", model.path, model.offset);
            let (objs, _) = tobj::load_obj(&model.path, &load_options).expect("Failed to load OBJ file");
            
            let mut model_vertices = Vec::new();
            let mut model_indices = Vec::new();
            
            let mut instance_min = Vec3::splat(f32::MAX);
            let mut instance_max = Vec3::splat(f32::MIN);
            
            println!("Models found: {}", objs.len());
            
            for m in objs.iter() {
                let mesh = &m.mesh;
                let base_vertex_local = model_vertices.len() as u32;
                
                // Compute bounding box for UV generation
                let mut min_p = [f32::MAX; 3];
                let mut max_p = [f32::MIN; 3];
                for i in 0..mesh.positions.len() / 3 {
                    let px = mesh.positions[i * 3];
                    let py = mesh.positions[i * 3 + 1];
                    let pz = mesh.positions[i * 3 + 2];
                    min_p[0] = min_p[0].min(px);
                    min_p[1] = min_p[1].min(py);
                    min_p[2] = min_p[2].min(pz);
                    max_p[0] = max_p[0].max(px);
                    max_p[1] = max_p[1].max(py);
                    max_p[2] = max_p[2].max(pz);
                }
                let range_x = (max_p[0] - min_p[0]).max(1e-5);
                let range_z = (max_p[2] - min_p[2]).max(1e-5);
                
                for i in 0..mesh.positions.len() / 3 {
                    let px = mesh.positions[i * 3] * model.scale + model.offset[0];
                    let py = mesh.positions[i * 3 + 1] * model.scale + model.offset[1];
                    let pz = mesh.positions[i * 3 + 2] * model.scale + model.offset[2];
                    
                    let p_vec = Vec3::new(px, py, pz);
                    instance_min = instance_min.min(p_vec);
                    instance_max = instance_max.max(p_vec);
                    
                    let uv = if mesh.texcoords.len() >= (i * 2 + 2) {
                        [mesh.texcoords[i * 2], 1.0 - mesh.texcoords[i * 2 + 1]]
                    } else if model.material_id > 0.5 {
                        [px * 0.1, pz * 0.1]
                    } else {
                        [(px - min_p[0]) / range_x, (pz - min_p[2]) / range_z]
                    };
                    
                    let nx = if mesh.normals.len() >= (i * 3 + 3) {
                        mesh.normals[i * 3]
                    } else { 0.0 };
                    let ny = if mesh.normals.len() >= (i * 3 + 3) {
                        mesh.normals[i * 3 + 1]
                    } else { 1.0 };
                    let nz = if mesh.normals.len() >= (i * 3 + 3) {
                        mesh.normals[i * 3 + 2]
                    } else { 0.0 };
                    
                    model_vertices.push(VertexData {
                        pos: [px, py, pz],
                        normal: [nx, ny, nz],
                        uv,
                        data: [model.material_id, model.opacity, model_idx as f32, 0.0],
                    });
                }
                
                for idx in &mesh.indices {
                    model_indices.push(idx + base_vertex_local);
                }
            }
            
            let collider = if model.collision_enabled {
                println!("Added collider ({:?})", model.collider_type);
                Some(Aabb::new(instance_min, instance_max, model.collider_type))
            } else {
                None
            };
            
            // Generate meshlets for this model
            let max_vertices = 64;
            let max_triangles = 124;
            
            let mut meshopt_meshlets = vec![cluster::Meshlet::default(); cluster::build_meshlets_bound(model_indices.len(), max_vertices, max_triangles)];
            
            let meshlet_count = cluster::build_meshlets(
                &mut meshopt_meshlets,
                &model_indices,
                model_vertices.len(),
                max_vertices,
                max_triangles,
            );
            meshopt_meshlets.truncate(meshlet_count);
            
            let mut meshlets = Vec::with_capacity(meshlet_count);
            let mut meshlet_verts = Vec::new();
            let mut meshlet_tris = Vec::new();
            let mut meshlet_model_idxs = Vec::with_capacity(meshlet_count);
            
            for m in &meshopt_meshlets {
                let vertex_offset = meshlet_verts.len() as u32;
                let triangle_offset = meshlet_tris.len() as u32;
                
                // Calculate meshlet center and radius in parallel-safe way
                let mut center = Vec3::ZERO;
                let mut count = 0;
                for i in 0..m.vertex_count as usize {
                    let local_v_idx = m.vertices[i] as usize;
                    let pos = model_vertices[local_v_idx].pos;
                    center += Vec3::new(pos[0], pos[1], pos[2]);
                    count += 1;
                }
                if count > 0 { center /= count as f32; }
                
                let mut radius = 0.0f32;
                for i in 0..m.vertex_count as usize {
                    let local_v_idx = m.vertices[i] as usize;
                    let pos = model_vertices[local_v_idx].pos;
                    radius = radius.max((Vec3::new(pos[0], pos[1], pos[2]) - center).length());
                }
                
                meshlets.push(crate::vulkan::vk_meshlets::Meshlet {
                    vertex_offset,
                    triangle_offset,
                    vertex_count: m.vertex_count as u32,
                    triangle_count: m.triangle_count as u32,
                    center_radius: [center.x, center.y, center.z, radius],
                });
                
                meshlet_model_idxs.push(model_idx as u32);
                
                // Store local vertex indices (will be adjusted later)
                for i in 0..m.vertex_count as usize {
                    let local_v_idx = m.vertices[i] as u32;
                    meshlet_verts.push(local_v_idx);
                }
                for i in 0..m.triangle_count as usize {
                    meshlet_tris.push(m.indices[i][0] as u32);
                    meshlet_tris.push(m.indices[i][1] as u32);
                    meshlet_tris.push(m.indices[i][2] as u32);
                }
            }
            
            ModelResult {
                model_idx,
                vertices: model_vertices,
                indices: model_indices,
                collider,
                meshlets,
                meshlet_vertices: meshlet_verts,
                meshlet_triangles: meshlet_tris,
                meshlet_model_indices: meshlet_model_idxs,
                base_position: Vec3::from(model.offset),
            }
        })
        .collect();
    
    // Sort results by model_idx to ensure deterministic ordering
    let mut sorted_results: Vec<_> = model_results;
    sorted_results.sort_by_key(|r| r.model_idx);
    
    // Merge results sequentially (required for correct offsets)
    let mut global_vertices = Vec::new();
    let mut global_indices = Vec::new();
    let mut global_colliders = Vec::new();
    let mut meshlets_data = Vec::new();
    let mut meshlet_vertices = Vec::new();
    let mut meshlet_triangles = Vec::new();
    let mut meshlet_model_indices = Vec::new();
    let mut model_rt_infos = Vec::with_capacity(models.len());
    
    for result in sorted_results {
        let base_vertex_global = global_vertices.len() as u32;
        let base_meshlet_vertex = meshlet_vertices.len() as u32;
        let base_meshlet_triangle = meshlet_triangles.len() as u32;
        
        // Capture counts before moving
        let vertex_count = result.vertices.len() as u32;
        let index_count = result.indices.len() as u32;
        
        // Extend global vertices
        global_vertices.extend(result.vertices);
        
        // Extend global indices (adjusted by base_vertex_global)
        let index_start = global_indices.len() as u32;
        for idx in result.indices.iter() {
            global_indices.push(idx + base_vertex_global);
        }
        
        // Add collider if present
        if let Some(collider) = result.collider {
            global_colliders.push(collider);
        }
        
        // Extend meshlets with adjusted offsets
        for mut meshlet in result.meshlets {
            meshlet.vertex_offset += base_meshlet_vertex;
            meshlet.triangle_offset += base_meshlet_triangle;
            meshlets_data.push(meshlet);
        }
        
        // Extend meshlet vertices (adjusted by base_vertex_global)
        for v_idx in result.meshlet_vertices {
            meshlet_vertices.push(v_idx + base_vertex_global);
        }
        
        // Extend meshlet triangles (no adjustment needed, they're local to meshlet)
        meshlet_triangles.extend(result.meshlet_triangles);
        
        // Extend meshlet model indices
        meshlet_model_indices.extend(result.meshlet_model_indices);
        
        // Track RT info
        model_rt_infos.push(ModelRtInfo {
            index_start,
            index_count,
            vertex_start: base_vertex_global,
            vertex_count,
            base_position: result.base_position,
        });
    }
    
     
    let mut histogram = std::collections::HashMap::new();
    for &idx in &meshlet_model_indices {
        *histogram.entry(idx).or_insert(0) += 1;
    }
    println!("Refactored Histogram:");
    let mut sorted_keys: Vec<_> = histogram.keys().collect();
    sorted_keys.sort();
    for k in sorted_keys {
        println!("  Model {}: {} meshlets", k, histogram[k]);
    }

    let real_meshlet_count = meshlets_data.len() as u32;

     
    if meshlets_data.is_empty() {
        meshlets_data.push(Meshlet::zeroed());
        meshlet_model_indices.push(0);
    }
    if global_vertices.is_empty() {
        global_vertices.push(VertexData {
            pos: [0.0; 3],
            normal: [0.0; 3],
            uv: [0.0; 2],
            data: [0.0; 4],
        });
    }
    if meshlet_vertices.is_empty() {
        meshlet_vertices.push(0);
    }
    let mut meshlet_triangles_u32 = meshlet_triangles;
    if meshlet_triangles_u32.is_empty() {
        meshlet_triangles_u32.push(0);
    }
    if global_indices.is_empty() {
        global_indices.extend_from_slice(&[0, 0, 0]);
    }

    let buffer_usage = vk::BufferUsageFlags::STORAGE_BUFFER;

    let (meshlet_buffer, meshlet_memory) = vk_buffers::create_device_local_buffer_with_data(
        instance,
        physical_device,
        device,
        command_pool,
        submit_queue,
        bytemuck::cast_slice(&meshlets_data),
        buffer_usage,
    );

    let vertex_usage = buffer_usage
        | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
        | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
        | vk::BufferUsageFlags::VERTEX_BUFFER;
    let (vertex_buffer, vertex_memory) =
        vk_buffers::create_device_local_buffer_with_data_flags(
            instance,
            physical_device,
            device,
            command_pool,
            submit_queue,
            bytemuck::cast_slice(&global_vertices),
            vertex_usage,
            vk::MemoryAllocateFlags::DEVICE_ADDRESS,
        );

    let (meshlet_vertices_buffer, meshlet_vertices_buffer_memory) =
        vk_buffers::create_device_local_buffer_with_data(
            instance,
            physical_device,
            device,
            command_pool,
            submit_queue,
            bytemuck::cast_slice(&meshlet_vertices),
            buffer_usage,
        );

    let index_usage = vk::BufferUsageFlags::STORAGE_BUFFER 
        | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS 
        | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
        | vk::BufferUsageFlags::INDEX_BUFFER;
    let (index_buffer, index_buffer_memory) = vk_buffers::create_device_local_buffer_with_data_flags(
        instance,
        physical_device,
        device,
        command_pool,
        submit_queue,
        bytemuck::cast_slice(&global_indices),
        index_usage,
        vk::MemoryAllocateFlags::DEVICE_ADDRESS,
    );

    let (meshlet_triangles_buffer, meshlet_triangles_buffer_memory) =
        vk_buffers::create_device_local_buffer_with_data(
            instance,
            physical_device,
            device,
            command_pool,
            submit_queue,
            bytemuck::cast_slice(&meshlet_triangles_u32),
            buffer_usage,
        );

    let mut gpu_materials = Vec::new();
    for model in models {
         gpu_materials.push(GpuMaterial {
             color: [model.color[0], model.color[1], model.color[2], 1.0],
             params: [model.metalness, model.roughness, 0.0, 0.0],
             offset: [0.0; 4],
             base_position: [model.offset[0], model.offset[1], model.offset[2], 0.0],
             rotation: [0.0, 0.0, 0.0, 1.0],
         });
    }
    
    if gpu_materials.is_empty() {
        gpu_materials.push(GpuMaterial {
            color: [1.0, 1.0, 1.0, 1.0],
            params: [0.0, 0.5, 0.0, 0.0],
            offset: [0.0; 4],
            base_position: [0.0; 4],
            rotation: [0.0, 0.0, 0.0, 1.0],
        });
    }

    let material_usage = vk::BufferUsageFlags::STORAGE_BUFFER;
    let (material_buffer, material_buffer_memory) = vk_buffers::create_buffer(
        instance,
        physical_device,
        device,
        (gpu_materials.len() * std::mem::size_of::<GpuMaterial>()) as vk::DeviceSize,
        material_usage,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        vk::MemoryAllocateFlags::empty(),
        Some(bytemuck::cast_slice(&gpu_materials)),
    );

    let (meshlet_params_buffer, meshlet_params_buffer_memory) = vk_buffers::create_device_local_buffer_with_data(
        instance,
        physical_device,
        device,
        command_pool,
        submit_queue,
        bytemuck::cast_slice(&meshlet_model_indices),
        vk::BufferUsageFlags::STORAGE_BUFFER,
    );

    println!("Generated {} meshlets (Real count: {})", meshlets_data.len(), real_meshlet_count);

    let mut meshlet_ranges = vec![(0u32, 0u32); models.len()];
    if !meshlet_model_indices.is_empty() {
        let mut current_model = meshlet_model_indices[0] as usize;
        let mut start = 0;
        for (i, &model_idx) in meshlet_model_indices.iter().enumerate() {
             let idx = model_idx as usize;
             if idx != current_model {
                  
                 let count = (i as u32) - start;
                 if current_model < meshlet_ranges.len() {
                     meshlet_ranges[current_model] = (start, count);
                 }
                  
                 current_model = idx;
                 start = i as u32;
             }
        }
         
        let count = (meshlet_model_indices.len() as u32) - start;
        if current_model < meshlet_ranges.len() {
             meshlet_ranges[current_model] = (start, count);
        }
    }

    (
        MeshBuffers {
            meshlet_buffer,
            meshlet_buffer_memory: meshlet_memory,
            vertex_buffer,
            vertex_buffer_memory: vertex_memory,
            index_buffer,
            index_buffer_memory,
            meshlet_vertices_buffer,
            meshlet_vertices_buffer_memory,
            meshlet_triangles_buffer,
            meshlet_triangles_buffer_memory,
            meshlet_count: real_meshlet_count,
            vertex_count: global_vertices.len() as u32,
            index_count: global_indices.len() as u32,
            material_buffer,
            material_buffer_memory,
            meshlet_params_buffer,
            meshlet_params_buffer_memory,
            meshlet_ranges,
            model_rt_infos,
        },
        global_colliders,
    )
}
