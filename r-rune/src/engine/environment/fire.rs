use ash::{ext, vk};
use std::mem::size_of;
use crate::vulkan::vk_textures::{self, Texture};

const VOLUME_SIZE: u32 = 64;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct FireSimPushConstants {
    dt: f32,
    time: f32,
    _pad: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct FireRenderPushConstants {
    mvp: glam::Mat4,
    camera_pos: glam::Vec4,  
    time: f32,
    _pad: [f32; 3],
}

pub struct FireRenderer {
    sim_pipeline_layout: vk::PipelineLayout,
    sim_pipeline: vk::Pipeline,
    render_pipeline_layout: vk::PipelineLayout,
    render_pipeline: vk::Pipeline,
    sim_descriptor_set_layout: vk::DescriptorSetLayout,
    render_descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    sim_descriptor_set: vk::DescriptorSet,
    render_descriptor_set: vk::DescriptorSet,
    
     
     
     
     
     
    density_texture_read: Texture,
    density_texture_write: Texture,
    velocity_texture_read: Texture,
    velocity_texture_write: Texture,
    pressure_texture_read: Texture,
    pressure_texture_write: Texture,
}

impl FireRenderer {
    pub fn new(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        render_pass: vk::RenderPass,
        msaa_samples: vk::SampleCountFlags,
    ) -> Self {
        unsafe {
             
            let format = vk::Format::R16G16B16A16_SFLOAT;
            let density_read = vk_textures::create_storage_image_3d(instance, physical_device, device, VOLUME_SIZE, VOLUME_SIZE, VOLUME_SIZE, format);
            let density_write = vk_textures::create_storage_image_3d(instance, physical_device, device, VOLUME_SIZE, VOLUME_SIZE, VOLUME_SIZE, format);
            let velocity_read = vk_textures::create_storage_image_3d(instance, physical_device, device, VOLUME_SIZE, VOLUME_SIZE, VOLUME_SIZE, format);
            let velocity_write = vk_textures::create_storage_image_3d(instance, physical_device, device, VOLUME_SIZE, VOLUME_SIZE, VOLUME_SIZE, format);
            let pressure_read = vk_textures::create_storage_image_3d(instance, physical_device, device, VOLUME_SIZE, VOLUME_SIZE, VOLUME_SIZE, format);
            let pressure_write = vk_textures::create_storage_image_3d(instance, physical_device, device, VOLUME_SIZE, VOLUME_SIZE, VOLUME_SIZE, format);

             
            let sim_bindings = [
                vk::DescriptorSetLayoutBinding::default()
                    .binding(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::COMPUTE),
                 vk::DescriptorSetLayoutBinding::default()
                    .binding(1)  
                    .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::COMPUTE),
            ];
            let sim_dsl_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&sim_bindings);
            let sim_descriptor_set_layout = device.create_descriptor_set_layout(&sim_dsl_info, None).unwrap();

            let render_bindings = [
                 vk::DescriptorSetLayoutBinding::default()
                    .binding(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            ];
            let render_dsl_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&render_bindings);
            let render_descriptor_set_layout = device.create_descriptor_set_layout(&render_dsl_info, None).unwrap();

             let pool_sizes = [
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_IMAGE,
                    descriptor_count: 10,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 10,
                },
            ];
            let pool_info = vk::DescriptorPoolCreateInfo::default()
                .pool_sizes(&pool_sizes)
                .max_sets(10);
            let descriptor_pool = device.create_descriptor_pool(&pool_info, None).unwrap();

            let sim_sets = device.allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(descriptor_pool)
                    .set_layouts(&[sim_descriptor_set_layout])
            ).unwrap();
            let sim_descriptor_set = sim_sets[0];
            
            let render_sets = device.allocate_descriptor_sets(
                 &vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(descriptor_pool)
                    .set_layouts(&[render_descriptor_set_layout])
            ).unwrap();
            let render_descriptor_set = render_sets[0];
            
             
             
             
             

             
             
            let sim_push_range = vk::PushConstantRange::default()
                .stage_flags(vk::ShaderStageFlags::COMPUTE)
                .offset(0)
                .size(size_of::<FireSimPushConstants>() as u32);
            let sim_layout_info = vk::PipelineLayoutCreateInfo::default()
                .set_layouts(std::slice::from_ref(&sim_descriptor_set_layout))
                .push_constant_ranges(std::slice::from_ref(&sim_push_range));
            let sim_pipeline_layout = device.create_pipeline_layout(&sim_layout_info, None).unwrap();
            
            let sim_code = crate::vulkan::vk_pipeline::compile_shader("shader/fire_sim.comp", shaderc::ShaderKind::Compute);
            let sim_module = device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&sim_code), None).unwrap();
            let sim_stage = vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::COMPUTE)
                .module(sim_module)
                .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap());
            let sim_pipeline_info = vk::ComputePipelineCreateInfo::default()
                .stage(sim_stage)
                .layout(sim_pipeline_layout);
            let sim_pipeline = device.create_compute_pipelines(vk::PipelineCache::null(), &[sim_pipeline_info], None).unwrap()[0];
            device.destroy_shader_module(sim_module, None);

             
            let render_push_range = vk::PushConstantRange::default()
                .stage_flags(vk::ShaderStageFlags::MESH_EXT | vk::ShaderStageFlags::FRAGMENT)
                .offset(0)
                .size(size_of::<FireRenderPushConstants>() as u32);
             let render_layout_info = vk::PipelineLayoutCreateInfo::default()
                .set_layouts(std::slice::from_ref(&render_descriptor_set_layout))
                .push_constant_ranges(std::slice::from_ref(&render_push_range));
            let render_pipeline_layout = device.create_pipeline_layout(&render_layout_info, None).unwrap();

            let mesh_code = crate::vulkan::vk_pipeline::compile_shader("shader/fire.mesh", shaderc::ShaderKind::Mesh);
            let frag_code = crate::vulkan::vk_pipeline::compile_shader("shader/fire.frag", shaderc::ShaderKind::Fragment);
            
            let mesh_module = device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&mesh_code), None).unwrap();
            let frag_module = device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&frag_code), None).unwrap();
            
            let main_name = std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap();
            let shader_stages = [
                vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::MESH_EXT).module(mesh_module).name(main_name),
                vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::FRAGMENT).module(frag_module).name(main_name),
            ];
            
             
            let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
                .color_write_mask(vk::ColorComponentFlags::RGBA)
                .blend_enable(true)
                .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::ONE)
                .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                .alpha_blend_op(vk::BlendOp::ADD);
             let color_blending = vk::PipelineColorBlendStateCreateInfo::default().attachments(std::slice::from_ref(&color_blend_attachment));

            let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
                .polygon_mode(vk::PolygonMode::FILL)
                .cull_mode(vk::CullModeFlags::NONE)
                .line_width(1.0);
            
            let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
                .rasterization_samples(msaa_samples);
                 
             let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
                .depth_test_enable(true)
                .depth_write_enable(false)  
                .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL);
                
            let viewport_state = vk::PipelineViewportStateCreateInfo::default().viewport_count(1).scissor_count(1);
            let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
            let dynamic_state = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
            
            let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
                .stages(&shader_stages)
                .viewport_state(&viewport_state)
                .rasterization_state(&rasterizer)
                .multisample_state(&multisampling)
                .color_blend_state(&color_blending)
                .depth_stencil_state(&depth_stencil)
                .dynamic_state(&dynamic_state)
                .layout(render_pipeline_layout)
                .render_pass(render_pass);
                
            let render_pipeline = device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None).unwrap()[0];
            
            device.destroy_shader_module(mesh_module, None);
            device.destroy_shader_module(frag_module, None);

            Self {
                sim_pipeline_layout,
                sim_pipeline,
                render_pipeline_layout,
                render_pipeline,
                sim_descriptor_set_layout,
                render_descriptor_set_layout,
                descriptor_pool,
                sim_descriptor_set,
                render_descriptor_set,
                density_texture_read: density_read,
                density_texture_write: density_write,
                velocity_texture_read: velocity_read,
                velocity_texture_write: velocity_write,
                pressure_texture_read: pressure_read,
                pressure_texture_write: pressure_write,
            }
        }
    }
    
    pub unsafe fn update(&mut self, device: &ash::Device, cmd: vk::CommandBuffer, dt: f32, time: f32) {
         
         
        device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, self.sim_pipeline);
        
         
         
         
         
        
        let image_info_read = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::GENERAL)
            .image_view(self.density_texture_read.view);
            
        let image_info_write = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::GENERAL)
            .image_view(self.density_texture_write.view);

        let write_sets = [
            vk::WriteDescriptorSet::default()
                .dst_set(self.sim_descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .image_info(std::slice::from_ref(&image_info_read)),
            vk::WriteDescriptorSet::default()
                .dst_set(self.sim_descriptor_set)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .image_info(std::slice::from_ref(&image_info_write)),
        ];
        device.update_descriptor_sets(&write_sets, &[]);
        
        device.cmd_bind_descriptor_sets(
            cmd, 
            vk::PipelineBindPoint::COMPUTE, 
            self.sim_pipeline_layout, 
            0, 
            &[self.sim_descriptor_set], 
            &[]
        );

        let push = FireSimPushConstants {
            dt,
            time,
            _pad: [0.0; 2],
        };
        device.cmd_push_constants(
            cmd, 
            self.sim_pipeline_layout, 
            vk::ShaderStageFlags::COMPUTE, 
            0, 
            bytemuck::bytes_of(&push)
        );

         
         
         
        
        let group_count = VOLUME_SIZE / 8;
        device.cmd_dispatch(cmd, group_count, group_count, group_count);
        
         
        let barrier = vk::MemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::SHADER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ);
        device.cmd_pipeline_barrier(
            cmd, 
            vk::PipelineStageFlags::COMPUTE_SHADER, 
            vk::PipelineStageFlags::FRAGMENT_SHADER | vk::PipelineStageFlags::COMPUTE_SHADER, 
            vk::DependencyFlags::empty(), 
            &[barrier], 
            &[], 
            &[]
        );
        
         
        std::mem::swap(&mut self.density_texture_read, &mut self.density_texture_write);
    }
    
    pub unsafe fn draw(
        &self, 
        instance: &ash::Instance,
        device: &ash::Device, 
        cmd: vk::CommandBuffer, 
        view: glam::Mat4, 
        proj: glam::Mat4, 
        camera_pos: glam::Vec3,
        time: f32,
        extent: vk::Extent2D
    ) {
        device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.render_pipeline);
        
        let viewport = vk::Viewport::default()
            .width(extent.width as f32)
            .height(extent.height as f32)
            .max_depth(1.0);
        device.cmd_set_viewport(cmd, 0, std::slice::from_ref(&viewport));
        
        let scissor = vk::Rect2D::default().extent(extent);
        device.cmd_set_scissor(cmd, 0, std::slice::from_ref(&scissor));
        
         
        let image_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::GENERAL)
            .image_view(self.density_texture_read.view)
            .sampler(self.density_texture_read.sampler);
            
         let write_set = vk::WriteDescriptorSet::default()
            .dst_set(self.render_descriptor_set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&image_info));
        device.update_descriptor_sets(&[write_set], &[]);
        
        device.cmd_bind_descriptor_sets(
            cmd, 
            vk::PipelineBindPoint::GRAPHICS, 
            self.render_pipeline_layout, 
            0, 
            &[self.render_descriptor_set], 
            &[]
        );
        
         
         
         
         
        let model = glam::Mat4::from_translation(glam::Vec3::new(-2.0, 1.0, 5.0)) * glam::Mat4::from_scale(glam::Vec3::splat(2.0));
        let mvp = proj * view * model;  
        
        let push = FireRenderPushConstants {
            mvp,
            camera_pos: glam::Vec4::from((camera_pos, 1.0)),
            time,
            _pad: [0.0; 3],
        };
        device.cmd_push_constants(
            cmd, 
            self.render_pipeline_layout, 
            vk::ShaderStageFlags::MESH_EXT | vk::ShaderStageFlags::FRAGMENT, 
            0, 
            bytemuck::bytes_of(&push)
        );
        
        let mesh_shader_fn = ext::mesh_shader::Device::new(instance, device); 
        mesh_shader_fn.cmd_draw_mesh_tasks(cmd, 1, 1, 1);
    }
    
    pub unsafe fn destroy(&self, device: &ash::Device) {
        device.destroy_pipeline(self.sim_pipeline, None);
        device.destroy_pipeline_layout(self.sim_pipeline_layout, None);
         
         
    }
}
