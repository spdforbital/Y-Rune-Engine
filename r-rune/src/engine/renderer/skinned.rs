 
 
 
 

use std::path::Path;
use ash::vk;
use gltf::image::Source as GltfImageSource;
use meshopt::cluster;

use crate::engine::animation;
use crate::vulkan::{vk_buffers, vk_meshlets::Meshlet, vk_textures};

 
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct SkinnedVertex {
    pub pos: [f32; 3],
    pub _pad0: f32,
    pub normal: [f32; 3],
    pub _pad1: f32,
    pub uv: [f32; 2],
    pub _pad2: [f32; 2],
    pub joints: [u32; 4],
    pub weights: [f32; 4],
}

 
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct SkinnedPushConstants {
    pub mvp: glam::Mat4,
    pub sun_dir: [f32; 4],
    pub sun_color: [f32; 4],  
    pub model_pos_scale: [f32; 4],
    pub model_rot: [f32; 4],
}

 
pub struct SkinnedActorDesc {
    pub id: String,
    pub path: String,
    pub position: glam::Vec3,
    pub scale: Option<f32>,
    pub rotation_y_deg: Option<f32>,
}

 
pub(crate) struct SkinnedActor {
    pub id: String,
    pub position: glam::Vec3,
    pub scale: f32,
    pub rotation_y: f32,
    pub texture: vk_textures::Texture,
    pub descriptor_set: vk::DescriptorSet,
    pub vertex_buffer: vk::Buffer,
    pub vertex_memory: vk::DeviceMemory,
    pub meshlet_buffer: vk::Buffer,
    pub meshlet_memory: vk::DeviceMemory,
    pub meshlet_vertices_buffer: vk::Buffer,
    pub meshlet_vertices_memory: vk::DeviceMemory,
    pub meshlet_triangles_buffer: vk::Buffer,
    pub meshlet_triangles_memory: vk::DeviceMemory,
    pub meshlet_count: u32,
    pub bone_buffer: vk::Buffer,
    pub bone_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_memory: vk::DeviceMemory,
    pub vertex_count: u32,
    pub index_count: u32,
    pub instance: animation::SkinnedInstance,
}

 
pub(crate) fn build_skinned_vertices(mesh: &animation::SkinnedMesh) -> Vec<SkinnedVertex> {
    mesh.base_positions
        .iter()
        .zip(mesh.base_normals.iter())
        .zip(mesh.uvs.iter())
        .zip(mesh.joints.iter())
        .zip(mesh.weights.iter())
        .map(|((((p, n), uv), joints), weights)| SkinnedVertex {
            pos: [p.x, p.y, p.z],
            _pad0: 0.0,
            normal: [n.x, n.y, n.z],
            _pad1: 0.0,
            uv: [uv.x, uv.y],
            _pad2: [0.0, 0.0],
            joints: [
                joints[0] as u32,
                joints[1] as u32,
                joints[2] as u32,
                joints[3] as u32,
            ],
            weights: *weights,
        })
        .collect()
}

 
pub(crate) fn build_skinned_meshlets(
    mesh: &animation::SkinnedMesh,
) -> (Vec<Meshlet>, Vec<u32>, Vec<u32>) {
    let max_vertices = 64;
    let max_triangles = 124;

    let mut meshopt_meshlets = vec![cluster::Meshlet::default();
        cluster::build_meshlets_bound(mesh.indices.len(), max_vertices, max_triangles)];
    let actual = cluster::build_meshlets(
        &mut meshopt_meshlets,
        &mesh.indices,
        mesh.base_positions.len(),
        max_vertices,
        max_triangles,
    );
    meshopt_meshlets.truncate(actual);

     
    let mut meshlets_data = Vec::with_capacity(meshopt_meshlets.len());
    let mut meshlet_vertices = Vec::with_capacity(meshopt_meshlets.len() * max_vertices);
    let mut meshlet_triangles = Vec::with_capacity(meshopt_meshlets.len() * max_triangles * 3);

    for m in &meshopt_meshlets {
        let vertex_offset = meshlet_vertices.len() as u32;
        let triangle_offset = meshlet_triangles.len() as u32;

        let mut center = glam::Vec3::ZERO;
        let mut vertex_positions = Vec::new();
        for i in 0..m.vertex_count as usize {
            let v_idx = m.vertices[i] as u32;
            meshlet_vertices.push(v_idx);
            let pos = mesh.base_positions[v_idx as usize];
            center += pos;
            vertex_positions.push(pos);
        }
        if m.vertex_count > 0 {
            center /= m.vertex_count as f32;
        }

        let mut radius = 0.0f32;
        for pos in &vertex_positions {
            let dist = (*pos - center).length();
            if dist > radius {
                radius = dist;
            }
        }

        for i in 0..m.triangle_count as usize {
            meshlet_triangles.push(m.indices[i][0] as u32);
            meshlet_triangles.push(m.indices[i][1] as u32);
            meshlet_triangles.push(m.indices[i][2] as u32);
        }

        meshlets_data.push(Meshlet {
            vertex_offset,
            triangle_offset,
            vertex_count: m.vertex_count as u32,
            triangle_count: m.triangle_count as u32,
            center_radius: [center.x, center.y, center.z, radius],
        });
    }

    if meshlets_data.is_empty() {
        meshlets_data.push(Meshlet {
            vertex_offset: 0,
            triangle_offset: 0,
            vertex_count: 0,
            triangle_count: 0,
            center_radius: [0.0; 4],
        });
    }
    if meshlet_vertices.is_empty() {
        meshlet_vertices.push(0);
    }
    if meshlet_triangles.is_empty() {
        meshlet_triangles.push(0);
    }

    (meshlets_data, meshlet_vertices, meshlet_triangles)
}

 
pub(crate) fn resolve_gltf_texture(path: &str) -> Option<String> {
    let (doc, _, _) = gltf::import(path).ok()?;
    let parent = Path::new(path).parent()?;
    for image in doc.images() {
        if let GltfImageSource::Uri { uri, .. } = image.source() {
            if uri.starts_with("data:") {
                continue;
            }
            let tex_path = parent.join(uri);
            if tex_path.exists() {
                return tex_path.to_str().map(|s| s.to_string());
            }
        }
    }
    None
}

 
pub(crate) unsafe fn create_skinned_actor(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
    descriptor_pool: vk::DescriptorPool,
    skinned_descriptor_set_layout: vk::DescriptorSetLayout,
    desc: &SkinnedActorDesc,
) -> SkinnedActor {
    let skinned_mesh = animation::load_skinned_gltf(&desc.path);
    let skinned_instance = animation::SkinnedInstance::new(skinned_mesh, None);
    let skinned_vertices = build_skinned_vertices(&skinned_instance.mesh);
    let (skinned_meshlets, skinned_meshlet_vertices, skinned_meshlet_triangles) =
        build_skinned_meshlets(&skinned_instance.mesh);
    let skinned_meshlet_count = skinned_meshlets.len() as u32;

    let (skinned_vertex_buffer, skinned_vertex_memory) =
        vk_buffers::create_device_local_buffer_with_data_flags(
            instance,
            physical_device,
            device,
            command_pool,
            graphics_queue,
            bytemuck::cast_slice(&skinned_vertices),
            vk::BufferUsageFlags::STORAGE_BUFFER 
            | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
            | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            vk::MemoryAllocateFlags::DEVICE_ADDRESS,
        );

     

     
    let (skinned_index_buffer, skinned_index_memory) =
        vk_buffers::create_device_local_buffer_with_data_flags(
            instance,
            physical_device,
            device,
            command_pool,
            graphics_queue,
            bytemuck::cast_slice(&skinned_instance.mesh.indices),
            vk::BufferUsageFlags::STORAGE_BUFFER 
            | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
            | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            vk::MemoryAllocateFlags::DEVICE_ADDRESS,
        );


    let (skinned_meshlet_buffer, skinned_meshlet_memory) =
        vk_buffers::create_device_local_buffer_with_data(
            instance,
            physical_device,
            device,
            command_pool,
            graphics_queue,
            bytemuck::cast_slice(&skinned_meshlets),
            vk::BufferUsageFlags::STORAGE_BUFFER,
        );

    let (skinned_meshlet_vertices_buffer, skinned_meshlet_vertices_memory) =
        vk_buffers::create_device_local_buffer_with_data(
            instance,
            physical_device,
            device,
            command_pool,
            graphics_queue,
            bytemuck::cast_slice(&skinned_meshlet_vertices),
            vk::BufferUsageFlags::STORAGE_BUFFER,
        );

    let (skinned_meshlet_triangles_buffer, skinned_meshlet_triangles_memory) =
        vk_buffers::create_device_local_buffer_with_data(
            instance,
            physical_device,
            device,
            command_pool,
            graphics_queue,
            bytemuck::cast_slice(&skinned_meshlet_triangles),
            vk::BufferUsageFlags::STORAGE_BUFFER,
        );

    let initial_mats = skinned_instance.sample_matrices();
    let bone_count = initial_mats.len().max(1);
    let bone_buffer_size =
        (bone_count * std::mem::size_of::<glam::Mat4>()) as vk::DeviceSize;
    let (skinned_bone_buffer, skinned_bone_memory) = vk_buffers::create_buffer(
        instance,
        physical_device,
        device,
        bone_buffer_size,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        vk::MemoryAllocateFlags::empty(),
        Some(bytemuck::cast_slice(&initial_mats)),
    );

    let skinned_tex_path = Path::new(&desc.path)
        .with_file_name("Texture.png");
    let skinned_tex_path_str = resolve_gltf_texture(&desc.path)
        .or_else(|| skinned_tex_path.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "assets/models/soldier/Texture.png".to_string());
    let skinned_texture = vk_textures::load_texture(
        instance,
        physical_device,
        device,
        command_pool,
        graphics_queue,
        &skinned_tex_path_str,
    );

    let layouts = [skinned_descriptor_set_layout];
    let alloc_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(descriptor_pool)
        .set_layouts(&layouts);
    let descriptor_set = device.allocate_descriptor_sets(&alloc_info).unwrap()[0];

    let meshlet_info = vk::DescriptorBufferInfo::default()
        .buffer(skinned_meshlet_buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let vertex_info = vk::DescriptorBufferInfo::default()
        .buffer(skinned_vertex_buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let meshlet_vertices_info = vk::DescriptorBufferInfo::default()
        .buffer(skinned_meshlet_vertices_buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let meshlet_triangles_info = vk::DescriptorBufferInfo::default()
        .buffer(skinned_meshlet_triangles_buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let bone_info = vk::DescriptorBufferInfo::default()
        .buffer(skinned_bone_buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let tex_info = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(skinned_texture.view)
        .sampler(skinned_texture.sampler);

    let writes = [
        vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&meshlet_info)),
        vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(1)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&vertex_info)),
        vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(2)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&meshlet_vertices_info)),
        vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(3)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&meshlet_triangles_info)),
        vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(4)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&bone_info)),
        vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(5)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&tex_info)),
    ];
    device.update_descriptor_sets(&writes, &[]);

    SkinnedActor {
        id: desc.id.clone(),
        position: desc.position,
        scale: desc.scale.unwrap_or(0.01),
        rotation_y: desc
            .rotation_y_deg
            .map(|d| d.to_radians())
            .unwrap_or(std::f32::consts::PI),
        texture: skinned_texture,
        descriptor_set,
        vertex_buffer: skinned_vertex_buffer,
        vertex_memory: skinned_vertex_memory,
        meshlet_buffer: skinned_meshlet_buffer,
        meshlet_memory: skinned_meshlet_memory,
        meshlet_vertices_buffer: skinned_meshlet_vertices_buffer,
        meshlet_vertices_memory: skinned_meshlet_vertices_memory,
        meshlet_triangles_buffer: skinned_meshlet_triangles_buffer,
        meshlet_triangles_memory: skinned_meshlet_triangles_memory,
        meshlet_count: skinned_meshlet_count,
        bone_buffer: skinned_bone_buffer,
        bone_memory: skinned_bone_memory,
        index_buffer: skinned_index_buffer,
        index_memory: skinned_index_memory,
        vertex_count: skinned_vertices.len() as u32,
        index_count: skinned_instance.mesh.indices.len() as u32,
        instance: skinned_instance,
    }
}

 
pub(crate) unsafe fn destroy_skinned_actor(
    device: &ash::Device,
    descriptor_pool: vk::DescriptorPool,
    actor: SkinnedActor,
) {
    let _ = device.free_descriptor_sets(descriptor_pool, &[actor.descriptor_set]);
    device.destroy_buffer(actor.vertex_buffer, None);
    device.free_memory(actor.vertex_memory, None);
    device.destroy_buffer(actor.meshlet_buffer, None);
    device.free_memory(actor.meshlet_memory, None);
    device.destroy_buffer(actor.meshlet_vertices_buffer, None);
    device.free_memory(actor.meshlet_vertices_memory, None);
    device.destroy_buffer(actor.meshlet_triangles_buffer, None);
    device.free_memory(actor.meshlet_triangles_memory, None);
    device.destroy_buffer(actor.bone_buffer, None);
    device.free_memory(actor.bone_memory, None);
    device.destroy_buffer(actor.index_buffer, None);
    device.free_memory(actor.index_memory, None);
    device.destroy_sampler(actor.texture.sampler, None);
    device.destroy_image_view(actor.texture.view, None);
    device.destroy_image(actor.texture.image, None);
    device.free_memory(actor.texture.memory, None);
}
