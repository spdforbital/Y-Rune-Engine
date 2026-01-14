use ash::vk;

use crate::vulkan::vk_buffers;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MenuVertex {
    pub pos: [f32; 2],
}

pub struct MenuRenderer {
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    panel_buffer: vk::Buffer,
    panel_memory: vk::DeviceMemory,
    panel_count: u32,
    header_buffer: vk::Buffer,
    header_memory: vk::DeviceMemory,
    header_count: u32,
    slots_buffer: vk::Buffer,
    slots_memory: vk::DeviceMemory,
    slots_count: u32,
}

impl MenuRenderer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        instance: &ash::Instance,
        device: &ash::Device,
        pdevice: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        render_pass: vk::RenderPass,
        _extent: vk::Extent2D,
        msaa_samples: vk::SampleCountFlags,
    ) -> Self {
        let (panel_buffer, panel_memory, panel_count) =
            unsafe { create_panel(instance, pdevice, device, command_pool, queue) };
        let (header_buffer, header_memory, header_count) =
            unsafe { create_header(instance, pdevice, device, command_pool, queue) };
        let (slots_buffer, slots_memory, slots_count) =
            unsafe { create_slots(instance, pdevice, device, command_pool, queue) };

        let (pipeline_layout, pipeline) = create_pipeline(device, render_pass, msaa_samples);
        Self {
            pipeline_layout,
            pipeline,
            panel_buffer,
            panel_memory,
            panel_count,
            header_buffer,
            header_memory,
            header_count,
            slots_buffer,
            slots_memory,
            slots_count,
        }
    }

    pub unsafe fn destroy(&self, device: &ash::Device) {
        device.destroy_pipeline(self.pipeline, None);
        device.destroy_pipeline_layout(self.pipeline_layout, None);
        device.destroy_buffer(self.panel_buffer, None);
        device.free_memory(self.panel_memory, None);
        device.destroy_buffer(self.header_buffer, None);
        device.free_memory(self.header_memory, None);
        device.destroy_buffer(self.slots_buffer, None);
        device.free_memory(self.slots_memory, None);
    }

    pub unsafe fn record_commands(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        extent: vk::Extent2D,
    ) {
        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline,
        );

        let viewport = vk::Viewport::default()
            .x(0.0)
            .y(0.0)
            .width(extent.width as f32)
            .height(extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0);
        device.cmd_set_viewport(command_buffer, 0, std::slice::from_ref(&viewport));

        let scissor = vk::Rect2D::default()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(extent);
        device.cmd_set_scissor(command_buffer, 0, std::slice::from_ref(&scissor));

        let offsets = [0u64];

         
        let vb_panel = [self.panel_buffer];
        device.cmd_bind_vertex_buffers(command_buffer, 0, &vb_panel, &offsets);
        device.cmd_push_constants(
            command_buffer,
            self.pipeline_layout,
            vk::ShaderStageFlags::FRAGMENT,
            0,
            bytemuck::bytes_of(&[0.0f32, 0.0, 0.0, 0.9]),  
        );
        device.cmd_draw(command_buffer, self.panel_count, 1, 0, 0);

         
        let vb_header = [self.header_buffer];
        device.cmd_bind_vertex_buffers(command_buffer, 0, &vb_header, &offsets);
        device.cmd_push_constants(
            command_buffer,
            self.pipeline_layout,
            vk::ShaderStageFlags::FRAGMENT,
            0,
            bytemuck::bytes_of(&[0.05f32, 0.05, 0.07, 0.95]),  
        );
        device.cmd_draw(command_buffer, self.header_count, 1, 0, 0);

         
        let vb_slots = [self.slots_buffer];
        device.cmd_bind_vertex_buffers(command_buffer, 0, &vb_slots, &offsets);
        device.cmd_push_constants(
            command_buffer,
            self.pipeline_layout,
            vk::ShaderStageFlags::FRAGMENT,
            0,
            bytemuck::bytes_of(&[0.28f32, 0.3, 0.32, 0.85]),  
        );
        device.cmd_draw(command_buffer, self.slots_count, 1, 0, 0);
    }
}

fn create_pipeline(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    msaa_samples: vk::SampleCountFlags,
) -> (vk::PipelineLayout, vk::Pipeline) {
    let vert_code =
        crate::vulkan::vk_pipeline::compile_shader("shader/menu.vert", shaderc::ShaderKind::Vertex);
    let frag_code = crate::vulkan::vk_pipeline::compile_shader(
        "shader/menu.frag",
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

    let binding_desc = vk::VertexInputBindingDescription::default()
        .binding(0)
        .stride(std::mem::size_of::<MenuVertex>() as u32)
        .input_rate(vk::VertexInputRate::VERTEX);
    let attribute_descs = [vk::VertexInputAttributeDescription::default()
        .binding(0)
        .location(0)
        .format(vk::Format::R32G32_SFLOAT)
        .offset(0)];
    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(std::slice::from_ref(&binding_desc))
        .vertex_attribute_descriptions(&attribute_descs);

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
        .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .alpha_blend_op(vk::BlendOp::ADD);
    let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
        .attachments(std::slice::from_ref(&color_blend_attachment));

    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state =
        vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

    let push_constant = vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        .offset(0)
        .size(16);
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

unsafe fn create_panel(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
) -> (vk::Buffer, vk::DeviceMemory, u32) {
     
    let left = -0.65;
    let right = 0.65;
    let top = 0.7;
    let bottom = -0.6;

    let verts = [
        MenuVertex {
            pos: [left, bottom],
        },
        MenuVertex {
            pos: [right, bottom],
        },
        MenuVertex { pos: [right, top] },
        MenuVertex {
            pos: [left, bottom],
        },
        MenuVertex { pos: [right, top] },
        MenuVertex { pos: [left, top] },
    ];

    let (buffer, memory) = vk_buffers::create_device_local_buffer_with_data(
        instance,
        pdevice,
        device,
        command_pool,
        queue,
        bytemuck::cast_slice(&verts),
        vk::BufferUsageFlags::VERTEX_BUFFER,
    );

    (buffer, memory, verts.len() as u32)
}

unsafe fn create_header(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
) -> (vk::Buffer, vk::DeviceMemory, u32) {
    let left = -0.65;
    let right = 0.65;
    let top = 0.7;
    let bottom = 0.52;

    let verts = [
        MenuVertex {
            pos: [left, bottom],
        },
        MenuVertex {
            pos: [right, bottom],
        },
        MenuVertex { pos: [right, top] },
        MenuVertex {
            pos: [left, bottom],
        },
        MenuVertex { pos: [right, top] },
        MenuVertex { pos: [left, top] },
    ];

    let (buffer, memory) = vk_buffers::create_device_local_buffer_with_data(
        instance,
        pdevice,
        device,
        command_pool,
        queue,
        bytemuck::cast_slice(&verts),
        vk::BufferUsageFlags::VERTEX_BUFFER,
    );

    (buffer, memory, verts.len() as u32)
}

unsafe fn create_slots(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
) -> (vk::Buffer, vk::DeviceMemory, u32) {
    let panel_left = -0.62;
    let panel_right = 0.62;
    let panel_bottom = -0.55;
    let header_bottom = 0.48;

    let cols = 5;
    let rows = 3;
    let slot_size = 0.18;
    let gap = 0.04;

    let available_width = panel_right - panel_left;
    let grid_width = cols as f32 * slot_size + (cols as f32 - 1.0) * gap;
    let start_x = panel_left + (available_width - grid_width) * 0.5;

    let available_height = header_bottom - panel_bottom;
    let grid_height = rows as f32 * slot_size + (rows as f32 - 1.0) * gap;
    let start_y = panel_bottom + (available_height - grid_height) * 0.5;

    let mut verts = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let x0 = start_x + c as f32 * (slot_size + gap);
            let y0 = start_y + r as f32 * (slot_size + gap);
            let x1 = x0 + slot_size;
            let y1 = y0 + slot_size;
            verts.push(MenuVertex { pos: [x0, y0] });
            verts.push(MenuVertex { pos: [x1, y0] });
            verts.push(MenuVertex { pos: [x1, y1] });
            verts.push(MenuVertex { pos: [x0, y0] });
            verts.push(MenuVertex { pos: [x1, y1] });
            verts.push(MenuVertex { pos: [x0, y1] });
        }
    }

    let (buffer, memory) = vk_buffers::create_device_local_buffer_with_data(
        instance,
        pdevice,
        device,
        command_pool,
        queue,
        bytemuck::cast_slice(&verts),
        vk::BufferUsageFlags::VERTEX_BUFFER,
    );

    (buffer, memory, verts.len() as u32)
}
