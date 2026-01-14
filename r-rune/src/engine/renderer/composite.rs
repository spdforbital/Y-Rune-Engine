
use ash::{vk, Device};

pub struct CompositeRenderer {
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,
    sampler: vk::Sampler,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CompositePushConstants {
    pub bloom_strength: f32,
    pub bloom_enabled: u32,
}

impl CompositeRenderer {
    pub fn new(device: &Device, render_pass: vk::RenderPass, msaa_samples: vk::SampleCountFlags) -> Self {
        let (pipeline_layout, descriptor_set_layout) = unsafe { create_pipeline_layout(device) };
        let pipeline = unsafe { create_pipeline(device, pipeline_layout, render_pass, msaa_samples) };
        let sampler = unsafe {
            device.create_sampler(&vk::SamplerCreateInfo::default()
                .mag_filter(vk::Filter::LINEAR)
                .min_filter(vk::Filter::LINEAR)
                .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE), None).unwrap()
        };

        Self {
            pipeline,
            pipeline_layout,
            descriptor_set_layout,
            sampler,
        }
    }

    pub unsafe fn destroy(&self, device: &Device) {
        device.destroy_sampler(self.sampler, None);
        device.destroy_pipeline(self.pipeline, None);
        device.destroy_pipeline_layout(self.pipeline_layout, None);
        device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
    }
    
    pub unsafe fn record_commands(
        &self,
        device: &Device,
        cmd: vk::CommandBuffer,
        descriptor_set: vk::DescriptorSet,  
        scene_view: vk::ImageView,
        bloom_view: vk::ImageView,  
        bloom_strength: f32,
        bloom_enabled: bool,
    ) {
        device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
        
         
         
         
         
         
        update_descriptor_set(device, descriptor_set, scene_view, bloom_view, self.sampler);
        
        device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline_layout, 0, &[descriptor_set], &[]);
        
        let pc = CompositePushConstants {
            bloom_strength,
            bloom_enabled: if bloom_enabled { 1 } else { 0 },
        };
        device.cmd_push_constants(cmd, self.pipeline_layout, vk::ShaderStageFlags::FRAGMENT, 0, bytemuck::bytes_of(&pc));
        
        device.cmd_draw(cmd, 3, 1, 0, 0);
    }
    
    pub fn descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
        self.descriptor_set_layout
    }
}

unsafe fn create_pipeline_layout(device: &Device) -> (vk::PipelineLayout, vk::DescriptorSetLayout) {
    let bindings = [
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
    ];
    let set_layout = device.create_descriptor_set_layout(&vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings), None).unwrap();
    let pc = vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        .offset(0)
        .size(std::mem::size_of::<CompositePushConstants>() as u32);
        
    let pipeline_layout = device.create_pipeline_layout(&vk::PipelineLayoutCreateInfo::default().set_layouts(&[set_layout]).push_constant_ranges(&[pc]), None).unwrap();
    
    (pipeline_layout, set_layout)
}

unsafe fn create_pipeline(device: &Device, layout: vk::PipelineLayout, render_pass: vk::RenderPass, _msaa_samples: vk::SampleCountFlags) -> vk::Pipeline {
    let vert = crate::vulkan::vk_pipeline::compile_shader("shader/composite.vert", shaderc::ShaderKind::Vertex);
    let frag = crate::vulkan::vk_pipeline::compile_shader("shader/composite.frag", shaderc::ShaderKind::Fragment);
    
    let vert_module = device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&vert), None).unwrap();
    let frag_module = device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&frag), None).unwrap();
    
    let stages = [
        vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::VERTEX).module(vert_module).name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap()),
        vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::FRAGMENT).module(frag_module).name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap()),
    ];
    
    let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .line_width(1.0);
        
     
    let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(false);
    let color_blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(std::slice::from_ref(&color_blend_attachment));
    
     
    let depth = vk::PipelineDepthStencilStateCreateInfo::default().depth_test_enable(false).depth_write_enable(false);
    
    let viewport = vk::PipelineViewportStateCreateInfo::default().viewport_count(1).scissor_count(1);
    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
    
    let multisample = vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(vk::SampleCountFlags::TYPE_1);  
    
    let info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&stages)
        .rasterization_state(&rasterization)
        .color_blend_state(&color_blend)
        .depth_stencil_state(&depth)
        .viewport_state(&viewport)
        .dynamic_state(&dynamic)
        .multisample_state(&multisample)
        .layout(layout)
        .render_pass(render_pass);
        
    let pipeline = device.create_graphics_pipelines(vk::PipelineCache::null(), &[info], None).unwrap()[0];
    
    device.destroy_shader_module(vert_module, None);
    device.destroy_shader_module(frag_module, None);
    
    pipeline
}

unsafe fn update_descriptor_set(device: &Device, set: vk::DescriptorSet, scene_view: vk::ImageView, bloom_view: vk::ImageView, sampler: vk::Sampler) {
    let scene_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(scene_view).sampler(sampler);
    let bloom_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::GENERAL).image_view(bloom_view).sampler(sampler);  
    
    let writes = [
        vk::WriteDescriptorSet::default().dst_set(set).dst_binding(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&scene_info)),
        vk::WriteDescriptorSet::default().dst_set(set).dst_binding(1).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&bloom_info)),
    ];
    device.update_descriptor_sets(&writes, &[]);
}
