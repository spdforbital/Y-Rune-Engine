use ash::{ext, vk};
use std::mem::size_of;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CloudPushConstants {
    pub camera_pos: [f32; 4],
    pub sun_dir: [f32; 4],
    pub sun_color: [f32; 4],
    pub inv_view_proj: glam::Mat4,
    pub mvp: glam::Mat4,
    pub wind_params: [f32; 4],
    pub time: f32,
    pub cloud_scale: f32,
    pub cloud_density: f32,
    pub cloud_absorption: f32,
    pub min_height: f32,
    pub max_height: f32,
    pub _pad: [f32; 2],
}

pub struct CloudRenderer {
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}

impl CloudRenderer {
    pub fn new(device: &ash::Device, render_pass: vk::RenderPass, msaa_samples: vk::SampleCountFlags) -> Self {
        let (pipeline_layout, pipeline) = create_pipeline(device, render_pass, msaa_samples);
        Self {
            pipeline_layout,
            pipeline,
        }
    }

    pub unsafe fn destroy(&self, device: &ash::Device) {
        device.destroy_pipeline(self.pipeline, None);
        device.destroy_pipeline_layout(self.pipeline_layout, None);
    }
    pub unsafe fn record_commands(
        &self,
        instance: &ash::Instance,
        device: &ash::Device,
        cmd: vk::CommandBuffer,
        _extent: vk::Extent2D,
        camera_pos: glam::Vec3,
        sun_dir: glam::Vec3,
        sun_color: glam::Vec3,
        sun_intensity: f32,
        view: glam::Mat4,
        proj: glam::Mat4,
        time: f32,
        config: &crate::engine::state::config::CloudConfig,
        wind_dir: glam::Vec2,
        wind_speed: f32,
        wind_turbulence: f32,
    ) {
        device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline);

        let view_proj = proj * view;
        let inv_view_proj = view_proj.inverse();
        
        let wind_speed = wind_speed * config.wind_scale;
        let pc = CloudPushConstants {
            camera_pos: [camera_pos.x, camera_pos.y, camera_pos.z, 1.0],
            sun_dir: [sun_dir.x, sun_dir.y, sun_dir.z, 0.0],
            sun_color: [sun_color.x * sun_intensity, sun_color.y * sun_intensity, sun_color.z * sun_intensity, 1.0],
            inv_view_proj,
            mvp: view_proj,  
            wind_params: [wind_dir.x, wind_dir.y, wind_speed, wind_turbulence],
            time,
            cloud_scale: config.scale,
            cloud_density: config.density,
            cloud_absorption: config.absorption,
            min_height: config.min_height,
            max_height: config.max_height,
            _pad: [0.0; 2],
        };

        device.cmd_push_constants(
            cmd,
            self.pipeline_layout,
            vk::ShaderStageFlags::MESH_EXT | vk::ShaderStageFlags::FRAGMENT,
            0,
            bytemuck::bytes_of(&pc),
        );

        let mesh_fn = ext::mesh_shader::Device::new(instance, device);
        mesh_fn.cmd_draw_mesh_tasks(cmd, 1, 1, 1);
    }
}

fn create_pipeline(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    msaa_samples: vk::SampleCountFlags,
) -> (vk::PipelineLayout, vk::Pipeline) {
    let mesh_code = crate::vulkan::vk_pipeline::compile_shader(
        "shader/cloud.mesh",
        shaderc::ShaderKind::Mesh,
    );
    let frag_code = crate::vulkan::vk_pipeline::compile_shader(
        "shader/cloud.frag",
        shaderc::ShaderKind::Fragment,
    );

    let mesh_module = unsafe {
        device
            .create_shader_module(
                &vk::ShaderModuleCreateInfo::default().code(&mesh_code),
                None,
            )
            .unwrap()
    };
    let frag_module = unsafe {
        device
            .create_shader_module(
                &vk::ShaderModuleCreateInfo::default().code(&frag_code),
                None,
            )
            .unwrap()
    };

    let main_name = std::ffi::CString::new("main").unwrap();
    let shader_stages = [
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::MESH_EXT)
            .module(mesh_module)
            .name(&main_name),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_module)
            .name(&main_name),
    ];

    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();
    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);
    
    let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .line_width(1.0);
    
    let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(msaa_samples);
    
     
    let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(true)  
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)  
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD);
        
    let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
        .attachments(std::slice::from_ref(&color_blend_attachment));
        
    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state =
        vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

     
    let push_constant = vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::MESH_EXT | vk::ShaderStageFlags::FRAGMENT)
        .offset(0)
        .size(size_of::<CloudPushConstants>() as u32);
        
    let layout_info = vk::PipelineLayoutCreateInfo::default()
        .push_constant_ranges(std::slice::from_ref(&push_constant));
        
    let pipeline_layout = unsafe { device.create_pipeline_layout(&layout_info, None).unwrap() };

    let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&input_assembly)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterizer)
        .multisample_state(&multisampling)
        .color_blend_state(&color_blending)
        .dynamic_state(&dynamic_state)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0);
         
         
         
         
         
        
    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(true)  
        .depth_write_enable(false)
        .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
        .depth_bounds_test_enable(false)
        .stencil_test_enable(false);

     
    let pipeline_info = pipeline_info.depth_stencil_state(&depth_stencil);

    let pipeline = unsafe {
        device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                std::slice::from_ref(&pipeline_info),
                None,
            )
            .unwrap()[0]
    };

    unsafe {
        device.destroy_shader_module(mesh_module, None);
        device.destroy_shader_module(frag_module, None);
    }

    (pipeline_layout, pipeline)
}
