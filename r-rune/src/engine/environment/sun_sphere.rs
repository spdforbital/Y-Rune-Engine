use ash::vk;
use std::mem::size_of;

const SUN_RADIUS: f32 = 50.0;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SunSpherePushConstants {
    pub proj: glam::Mat4,
    pub sun_view_pos_radius: [f32; 4],
    pub sun_color_intensity: [f32; 4],
    pub params: [f32; 4],  
}

pub struct SunSphereRenderer {
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}

impl SunSphereRenderer {
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
        device: &ash::Device,
        cmd: vk::CommandBuffer,
        extent: vk::Extent2D,
        view: glam::Mat4,
        proj: glam::Mat4,
        sun_pos: glam::Vec3,
        sun_color: glam::Vec3,
        sun_intensity: f32,
        time: f32,
    ) {
        let intensity = (sun_intensity / 40.0).clamp(0.0, 1.0);
        if intensity <= 0.01 {
            return;
        }

        let sun_view = view * sun_pos.extend(1.0);
        if sun_view.z >= -0.1 {
            return;
        }

        device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline);

        let viewport = vk::Viewport::default()
            .width(extent.width as f32)
            .height(extent.height as f32)
            .max_depth(1.0);
        device.cmd_set_viewport(cmd, 0, std::slice::from_ref(&viewport));

        let scissor = vk::Rect2D::default().extent(extent);
        device.cmd_set_scissor(cmd, 0, std::slice::from_ref(&scissor));

        let pc = SunSpherePushConstants {
            proj,
            sun_view_pos_radius: [sun_view.x, sun_view.y, sun_view.z, SUN_RADIUS],
            sun_color_intensity: [sun_color.x, sun_color.y, sun_color.z, intensity],
            params: [time, 0.9, 1.6, 1.8],
        };

        device.cmd_push_constants(
            cmd,
            self.pipeline_layout,
            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            0,
            bytemuck::bytes_of(&pc),
        );

        device.cmd_draw(cmd, 6, 1, 0, 0);
    }
}

fn create_pipeline(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    msaa_samples: vk::SampleCountFlags,
) -> (vk::PipelineLayout, vk::Pipeline) {
    let vert_code = crate::vulkan::vk_pipeline::compile_shader(
        "shader/sun_sphere.vert",
        shaderc::ShaderKind::Vertex,
    );
    let frag_code = crate::vulkan::vk_pipeline::compile_shader(
        "shader/sun_sphere.frag",
        shaderc::ShaderKind::Fragment,
    );

    let vert_module = unsafe {
        device
            .create_shader_module(
                &vk::ShaderModuleCreateInfo::default().code(&vert_code),
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
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_module)
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
        .src_color_blend_factor(vk::BlendFactor::ONE)
        .dst_color_blend_factor(vk::BlendFactor::ONE)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ONE)
        .alpha_blend_op(vk::BlendOp::ADD);
    let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
        .attachments(std::slice::from_ref(&color_blend_attachment));
    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state =
        vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

    let push_constant = vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
        .offset(0)
        .size(size_of::<SunSpherePushConstants>() as u32);
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
        device.destroy_shader_module(vert_module, None);
        device.destroy_shader_module(frag_module, None);
    }

    (pipeline_layout, pipeline)
}
