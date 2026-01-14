use ash::{vk, Device, Instance};
use std::mem::size_of;
use crate::vulkan::{vk_buffers, vk_pipeline, vk_sync};
use crate::engine::renderer::Renderer;
use crate::engine::renderer::FrameInput;

pub mod loader;
use loader::{SdfModel, OP_SPHERE, OP_BOX, OP_CYLINDER, OP_TORUS, OP_UNION, OP_SUB, OP_INTERSECT, OP_SMOOTH_UNION, OP_SMOOTH_SUB, OP_SMOOTH_INTERSECT};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SdfPushConstants {
    inv_view: glam::Mat4,
    inv_proj: glam::Mat4,
    camera_pos: glam::Vec4,
    params: glam::Vec4,  
    sun_dir: glam::Vec4,
    sun_color: glam::Vec4,
}

pub struct SdfRenderer {
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,
    
     
    data_buffer: vk::Buffer,
    data_memory: vk::DeviceMemory,
    data_size: u64,
}

impl SdfRenderer {
    pub fn new(
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        render_pass: vk::RenderPass,
        graphics_queue: vk::Queue,
        command_pool: vk::CommandPool,
        msaa_samples: vk::SampleCountFlags,
    ) -> Self {
         
        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        ];
        
        let dsl_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        let descriptor_set_layout = unsafe { device.create_descriptor_set_layout(&dsl_info, None).unwrap() };
        
         
        let push_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(size_of::<SdfPushConstants>() as u32);
            
        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(std::slice::from_ref(&descriptor_set_layout))
            .push_constant_ranges(std::slice::from_ref(&push_range));
        let pipeline_layout = unsafe { device.create_pipeline_layout(&layout_info, None).unwrap() };
        
         
         
         
         
         
         
         
        
        let vert_code = vk_pipeline::compile_shader("shader/sdf.vert", shaderc::ShaderKind::Vertex);
        let frag_code = vk_pipeline::compile_shader("shader/sdf.frag", shaderc::ShaderKind::Fragment);
        
        let vert_module = unsafe { device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&vert_code), None).unwrap() };
        let frag_module = unsafe { device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&frag_code), None).unwrap() };
        
        let main = std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap();
        let stages = [
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::VERTEX).module(vert_module).name(main),
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::FRAGMENT).module(frag_module).name(main),
        ];

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
        
        let viewport_state = vk::PipelineViewportStateCreateInfo::default().viewport_count(1).scissor_count(1);
        
         
         
         
        let blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false);
            
        let blend_state = vk::PipelineColorBlendStateCreateInfo::default().attachments(std::slice::from_ref(&blend_attachment));
        
        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::default()
            .cull_mode(vk::CullModeFlags::NONE)
            .line_width(1.0);
            
        let multisample_state = vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(msaa_samples);
        
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS);
            
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default();
        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::default().topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&blend_state)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(render_pass);
            
        let pipeline = unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None).unwrap()[0] };
        
        unsafe {
            device.destroy_shader_module(vert_module, None);
            device.destroy_shader_module(frag_module, None);
        }

         
        let model = Self::create_sample_vase();
        let (data_buffer, data_memory) = unsafe {
            vk_buffers::create_device_local_buffer_with_data(
                instance,
                physical_device,
                device,
                command_pool,
                graphics_queue,
                bytemuck::cast_slice(&model.data),
                vk::BufferUsageFlags::STORAGE_BUFFER,
            )
        };
        let data_size = (model.data.len() * 4) as u64;

         
        let pool_size = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::STORAGE_BUFFER,
            descriptor_count: 1,
        }];
        let pool_info = vk::DescriptorPoolCreateInfo::default().pool_sizes(&pool_size).max_sets(1);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None).unwrap() };
        
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
             .descriptor_pool(descriptor_pool)
             .set_layouts(std::slice::from_ref(&descriptor_set_layout));
        let descriptor_set = unsafe { device.allocate_descriptor_sets(&alloc_info).unwrap()[0] };
        
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(data_buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE);  
            
        let write = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&buffer_info));
            
        unsafe { device.update_descriptor_sets(&[write], &[]) };

        Self {
            pipeline_layout,
            pipeline,
            descriptor_set_layout,
            descriptor_pool,
            descriptor_set,
            data_buffer,
            data_memory,
            data_size,
        }
    }

    fn create_sample_vase() -> SdfModel {
         
         
         
         
        
         
         
         
         
         
         
         
         
         
        
        let color = glam::Vec3::new(0.8, 0.4, 0.3);
        let pos = glam::Vec3::new(2.0, 1.0, 5.0);  
        
        SdfModel::new()
            .cylinder(pos, 1.5, 0.6, color)  
            .sphere(pos - glam::Vec3::Y * 1.5, 0.7, color)  
            .smooth_union(0.5)
            .cylinder(pos + glam::Vec3::Y * 1.8, 0.5, 0.4, color)  
            .smooth_union(0.3)
             
            .cylinder(pos + glam::Vec3::Y * 0.2, 2.5, 0.5, color)  
            .smooth_subtract(0.1)
    }

    pub fn draw(&self, device: &Device, cmd: vk::CommandBuffer, input: &FrameInput, extent: vk::Extent2D) {
        unsafe {
            device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
            
            let viewport = vk::Viewport::default()
                .width(extent.width as f32)
                .height(extent.height as f32)
                .max_depth(1.0);
            device.cmd_set_viewport(cmd, 0, std::slice::from_ref(&viewport));
            let scissor = vk::Rect2D::default().extent(extent);
            device.cmd_set_scissor(cmd, 0, std::slice::from_ref(&scissor));
            
            device.cmd_bind_descriptor_sets(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[self.descriptor_set],
                &[],
            );
            
            let push = SdfPushConstants {
                inv_view: input.view.inverse(),
                inv_proj: input.proj.inverse(),
                camera_pos: glam::Vec4::from((input.camera_pos, 1.0)),
                params: glam::Vec4::new(input.dt, 0.0, 0.0, 0.0),
                sun_dir: glam::Vec4::from((input.sun_dir, 0.0)),
                sun_color: glam::Vec4::from((input.sun_color, 1.0)),
            };
            
            device.cmd_push_constants(
                cmd,
                self.pipeline_layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&push),
            );
            
             
            device.cmd_draw(cmd, 3, 1, 0, 0); 
        }
    }
    
    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_layout, None);
            device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            device.destroy_descriptor_pool(self.descriptor_pool, None);
            device.destroy_buffer(self.data_buffer, None);
            device.free_memory(self.data_memory, None);
        }
    }
}
