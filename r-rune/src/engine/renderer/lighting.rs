 
 
 
 

use ash::vk;
use crate::vulkan::{vk_buffers, vk_memory, vk_pipeline};

 
pub const TILE_SIZE: u32 = 16;
pub const DEPTH_SLICES: u32 = 24;
pub const MAX_LIGHTS_PER_CLUSTER: u32 = 256;
pub const MAX_LIGHTS: u32 = 1024;

 
pub const LIGHT_TYPE_POINT: u32 = 0;
pub const LIGHT_TYPE_SPOT: u32 = 1;

 
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GpuLight {
     
    pub position: [f32; 4],
     
    pub direction: [f32; 4],
     
    pub color: [f32; 4],
     
    pub light_type: u32,
     
    pub spot_inner: f32,
     
    pub _pad: [u32; 2],
}

impl Default for GpuLight {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0, 10.0],  
            direction: [0.0, -1.0, 0.0, 0.9],  
            color: [1.0, 1.0, 1.0, 1.0],  
            light_type: LIGHT_TYPE_POINT,
            spot_inner: 0.95,
            _pad: [0, 0],
        }
    }
}

impl GpuLight {
     
    pub fn point(position: glam::Vec3, radius: f32, color: glam::Vec3, intensity: f32) -> Self {
        Self {
            position: [position.x, position.y, position.z, radius],
            direction: [0.0, -1.0, 0.0, 1.0],
            color: [color.x, color.y, color.z, intensity],
            light_type: LIGHT_TYPE_POINT,
            spot_inner: 1.0,
            _pad: [0, 0],
        }
    }

     
    pub fn spot(
        position: glam::Vec3,
        direction: glam::Vec3,
        radius: f32,
        color: glam::Vec3,
        intensity: f32,
        inner_angle_deg: f32,
        outer_angle_deg: f32,
    ) -> Self {
        let dir = direction.normalize();
        Self {
            position: [position.x, position.y, position.z, radius],
            direction: [dir.x, dir.y, dir.z, outer_angle_deg.to_radians().cos()],
            color: [color.x, color.y, color.z, intensity],
            light_type: LIGHT_TYPE_SPOT,
            spot_inner: inner_angle_deg.to_radians().cos(),
            _pad: [0, 0],
        }
    }
}

 
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LightCullPushConstants {
    pub view: glam::Mat4,
    pub proj: glam::Mat4,
    pub cluster_size: [u32; 2],
    pub screen_size: [u32; 2],
    pub light_count: u32,
    pub depth_slices: u32,
    pub near_plane: f32,
    pub far_plane: f32,
}

 
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ClusterInfo {
    pub offset: u32,
    pub count: u32,
}

 
pub struct LightingSystem {
     
    pub light_buffer: vk::Buffer,
    pub light_buffer_memory: vk::DeviceMemory,
    
     
    pub cluster_info_buffer: vk::Buffer,
    pub cluster_info_memory: vk::DeviceMemory,
    
     
    pub light_index_buffer: vk::Buffer,
    pub light_index_memory: vk::DeviceMemory,
    
     
    pub cull_pipeline: vk::Pipeline,
    pub cull_pipeline_layout: vk::PipelineLayout,
    pub cull_descriptor_set_layout: vk::DescriptorSetLayout,
    pub cull_descriptor_set: vk::DescriptorSet,
    
     
    pub cluster_count_x: u32,
    pub cluster_count_y: u32,
    pub cluster_count_z: u32,
    
     
    pub light_count: u32,
}

impl LightingSystem {
     
    pub unsafe fn new(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        descriptor_pool: vk::DescriptorPool,
        screen_width: u32,
        screen_height: u32,
    ) -> Self {
        let cluster_count_x = (screen_width + TILE_SIZE - 1) / TILE_SIZE;
        let cluster_count_y = (screen_height + TILE_SIZE - 1) / TILE_SIZE;
        let cluster_count_z = DEPTH_SLICES;
        let total_clusters = cluster_count_x * cluster_count_y * cluster_count_z;
        
         
        let light_buffer_size = (MAX_LIGHTS as usize * std::mem::size_of::<GpuLight>()) as u64;
        let (light_buffer, light_buffer_memory) = vk_buffers::create_buffer(
            instance,
            physical_device,
            device,
            light_buffer_size,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::MemoryAllocateFlags::empty(),
            None,
        );
        
         
        let cluster_info_size = (total_clusters as usize * std::mem::size_of::<ClusterInfo>()) as u64;
        let (cluster_info_buffer, cluster_info_memory) = vk_buffers::create_buffer(
            instance,
            physical_device,
            device,
            cluster_info_size,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::MemoryAllocateFlags::empty(),
            None,
        );
        
         
        let light_index_size = (total_clusters as usize * MAX_LIGHTS_PER_CLUSTER as usize * 4) as u64;
        let (light_index_buffer, light_index_memory) = vk_buffers::create_buffer(
            instance,
            physical_device,
            device,
            light_index_size,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::MemoryAllocateFlags::empty(),
            None,
        );
        
         
        let (cull_pipeline, cull_pipeline_layout, cull_descriptor_set_layout) = 
            create_light_cull_pipeline(device);
        
         
        let layouts = [cull_descriptor_set_layout];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);
        let cull_descriptor_set = device.allocate_descriptor_sets(&alloc_info).unwrap()[0];
        
         
        let light_info = vk::DescriptorBufferInfo::default()
            .buffer(light_buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE);
        let cluster_info = vk::DescriptorBufferInfo::default()
            .buffer(cluster_info_buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE);
        let index_info = vk::DescriptorBufferInfo::default()
            .buffer(light_index_buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE);
        
        let writes = [
            vk::WriteDescriptorSet::default()
                .dst_set(cull_descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(&light_info)),
            vk::WriteDescriptorSet::default()
                .dst_set(cull_descriptor_set)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(&cluster_info)),
            vk::WriteDescriptorSet::default()
                .dst_set(cull_descriptor_set)
                .dst_binding(2)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(&index_info)),
        ];
        device.update_descriptor_sets(&writes, &[]);
        
        Self {
            light_buffer,
            light_buffer_memory,
            cluster_info_buffer,
            cluster_info_memory,
            light_index_buffer,
            light_index_memory,
            cull_pipeline,
            cull_pipeline_layout,
            cull_descriptor_set_layout,
            cull_descriptor_set,
            cluster_count_x,
            cluster_count_y,
            cluster_count_z,
            light_count: 0,
        }
    }
    
     
    pub unsafe fn upload_lights(&mut self, device: &ash::Device, lights: &[GpuLight]) {
        self.light_count = lights.len().min(MAX_LIGHTS as usize) as u32;
        
        if self.light_count == 0 {
            return;
        }
        
        let size = (self.light_count as usize * std::mem::size_of::<GpuLight>()) as u64;
        let ptr = device.map_memory(
            self.light_buffer_memory,
            0,
            size,
            vk::MemoryMapFlags::empty(),
        ).unwrap() as *mut GpuLight;
        
        std::ptr::copy_nonoverlapping(lights.as_ptr(), ptr, self.light_count as usize);
        device.unmap_memory(self.light_buffer_memory);
    }
    
     
     
    pub unsafe fn update_dimensions(
        &mut self,
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        screen_width: u32,
        screen_height: u32,
    ) -> Option<(vk::Buffer, vk::DeviceMemory, vk::Buffer, vk::DeviceMemory)> {
        let new_cx = (screen_width + TILE_SIZE - 1) / TILE_SIZE;
        let new_cy = (screen_height + TILE_SIZE - 1) / TILE_SIZE;
        
        let old_total = self.cluster_count_x * self.cluster_count_y * self.cluster_count_z;
        let new_total = new_cx * new_cy * self.cluster_count_z;
        
        self.cluster_count_x = new_cx;
        self.cluster_count_y = new_cy;
        
        if new_total > old_total {
            let old_c_buf = self.cluster_info_buffer;
            let old_c_mem = self.cluster_info_memory;
            let old_i_buf = self.light_index_buffer;
            let old_i_mem = self.light_index_memory;

            let cluster_info_size = (new_total as usize * std::mem::size_of::<ClusterInfo>()) as u64;
            let (c_buf, c_mem) = vk_buffers::create_buffer(
                instance,
                physical_device,
                device,
                cluster_info_size,
                vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                vk::MemoryAllocateFlags::empty(),
                None,
            );
            
            let light_index_size = (new_total as usize * MAX_LIGHTS_PER_CLUSTER as usize * 4) as u64;
            let (i_buf, i_mem) = vk_buffers::create_buffer(
                instance,
                physical_device,
                device,
                light_index_size,
                vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                vk::MemoryAllocateFlags::empty(),
                None,
            );
            
            self.cluster_info_buffer = c_buf;
            self.cluster_info_memory = c_mem;
            self.light_index_buffer = i_buf;
            self.light_index_memory = i_mem;
            
             
            let cmd = vk_buffers::begin_single_time_commands(device, command_pool);
            device.cmd_fill_buffer(cmd, c_buf, 0, vk::WHOLE_SIZE, 0);
            device.cmd_fill_buffer(cmd, i_buf, 0, vk::WHOLE_SIZE, 0);
            vk_buffers::end_single_time_commands(device, queue, command_pool, cmd);
            
             
             let cluster_info = vk::DescriptorBufferInfo::default()
                .buffer(self.cluster_info_buffer)
                .offset(0)
                .range(vk::WHOLE_SIZE);
            let index_info = vk::DescriptorBufferInfo::default()
                .buffer(self.light_index_buffer)
                .offset(0)
                .range(vk::WHOLE_SIZE);
            
            let writes = [
                vk::WriteDescriptorSet::default()
                    .dst_set(self.cull_descriptor_set)
                    .dst_binding(1)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .buffer_info(std::slice::from_ref(&cluster_info)),
                vk::WriteDescriptorSet::default()
                    .dst_set(self.cull_descriptor_set)
                    .dst_binding(2)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .buffer_info(std::slice::from_ref(&index_info)),
            ];
            device.update_descriptor_sets(&writes, &[]);
            
            return Some((old_c_buf, old_c_mem, old_i_buf, old_i_mem));
        }
        None
    }
    
     
    pub unsafe fn record_cull(
        &self,
        device: &ash::Device,
        cmd: vk::CommandBuffer,
        view: glam::Mat4,
        proj: glam::Mat4,
        screen_width: u32,
        screen_height: u32,
        near: f32,
        far: f32,
    ) {
        if self.light_count == 0 {
            return;
        }
        
        device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, self.cull_pipeline);
        device.cmd_bind_descriptor_sets(
            cmd,
            vk::PipelineBindPoint::COMPUTE,
            self.cull_pipeline_layout,
            0,
            &[self.cull_descriptor_set],
            &[],
        );
        
        let push = LightCullPushConstants {
            view,
            proj,
            cluster_size: [TILE_SIZE, TILE_SIZE],
            screen_size: [screen_width, screen_height],
            light_count: self.light_count,
            depth_slices: DEPTH_SLICES,
            near_plane: near,
            far_plane: far,
        };
        
        device.cmd_push_constants(
            cmd,
            self.cull_pipeline_layout,
            vk::ShaderStageFlags::COMPUTE,
            0,
            std::slice::from_raw_parts(
                &push as *const _ as *const u8,
                std::mem::size_of::<LightCullPushConstants>(),
            ),
        );
        
         
        device.cmd_dispatch(cmd, self.cluster_count_x, self.cluster_count_y, 1);
    }
    
     
    pub unsafe fn destroy(&self, device: &ash::Device) {
        device.destroy_pipeline(self.cull_pipeline, None);
        device.destroy_pipeline_layout(self.cull_pipeline_layout, None);
        device.destroy_descriptor_set_layout(self.cull_descriptor_set_layout, None);
        device.destroy_buffer(self.light_buffer, None);
        device.free_memory(self.light_buffer_memory, None);
        device.destroy_buffer(self.cluster_info_buffer, None);
        device.free_memory(self.cluster_info_memory, None);
        device.destroy_buffer(self.light_index_buffer, None);
        device.free_memory(self.light_index_memory, None);
    }
}

 
unsafe fn create_light_cull_pipeline(
    device: &ash::Device,
) -> (vk::Pipeline, vk::PipelineLayout, vk::DescriptorSetLayout) {
     
    let bindings = [
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE),
        vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE),
        vk::DescriptorSetLayoutBinding::default()
            .binding(2)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE),
    ];
    
    let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
        .bindings(&bindings);
    let descriptor_set_layout = device.create_descriptor_set_layout(&layout_info, None).unwrap();
    
     
    let push_range = vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::COMPUTE)
        .offset(0)
        .size(std::mem::size_of::<LightCullPushConstants>() as u32);
    
    let layouts = [descriptor_set_layout];
    let layout_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(&layouts)
        .push_constant_ranges(std::slice::from_ref(&push_range));
    let pipeline_layout = device.create_pipeline_layout(&layout_info, None).unwrap();
    
     
    let shader_code = vk_pipeline::compile_shader(
        "shader/light_cull.comp",
        shaderc::ShaderKind::Compute,
    );
    
    let shader_module_info = vk::ShaderModuleCreateInfo::default()
        .code(&shader_code);
    let shader_module = device.create_shader_module(&shader_module_info, None).unwrap();
    
    let stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::COMPUTE)
        .module(shader_module)
        .name(c"main");
    
    let pipeline_info = vk::ComputePipelineCreateInfo::default()
        .stage(stage)
        .layout(pipeline_layout);
    
    let pipeline = device
        .create_compute_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
        .unwrap()[0];
    
    device.destroy_shader_module(shader_module, None);
    
    (pipeline, pipeline_layout, descriptor_set_layout)
}
