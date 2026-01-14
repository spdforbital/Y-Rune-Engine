use ash::vk;

use crate::vulkan::vk_buffers;

#[derive(Clone, Copy)]
pub enum CrosshairStyle {
    Dot,
    Bars,
}

pub struct CrosshairRenderer {
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    dot_buffer: vk::Buffer,
    dot_memory: vk::DeviceMemory,
    dot_count: u32,
    bars_buffer: vk::Buffer,
    bars_memory: vk::DeviceMemory,
    bars_count: u32,
}

impl CrosshairRenderer {
    pub unsafe fn new(
        instance: &ash::Instance,
        device: &ash::Device,
        pdevice: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        render_pass: vk::RenderPass,
        scale: f32,
        msaa_samples: vk::SampleCountFlags,
    ) -> Self {
        let (pipeline_layout, pipeline) = create_pipeline(device, render_pass, msaa_samples);
        let (dot_buffer, dot_memory, dot_count) =
            create_dot(instance, pdevice, device, command_pool, queue, scale);
        let (bars_buffer, bars_memory, bars_count) =
            create_bars(instance, pdevice, device, command_pool, queue, scale);

        Self {
            pipeline_layout,
            pipeline,
            dot_buffer,
            dot_memory,
            dot_count,
            bars_buffer,
            bars_memory,
            bars_count,
        }
    }

    pub unsafe fn update_scale(
        &mut self,
        instance: &ash::Instance,
        device: &ash::Device,
        pdevice: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        scale: f32,
    ) {
        device.destroy_buffer(self.dot_buffer, None);
        device.free_memory(self.dot_memory, None);
        device.destroy_buffer(self.bars_buffer, None);
        device.free_memory(self.bars_memory, None);

        let (dot_buffer, dot_memory, dot_count) =
            create_dot(instance, pdevice, device, command_pool, queue, scale);
        let (bars_buffer, bars_memory, bars_count) =
            create_bars(instance, pdevice, device, command_pool, queue, scale);
        self.dot_buffer = dot_buffer;
        self.dot_memory = dot_memory;
        self.dot_count = dot_count;
        self.bars_buffer = bars_buffer;
        self.bars_memory = bars_memory;
        self.bars_count = bars_count;
    }

    pub unsafe fn destroy(&self, device: &ash::Device) {
        device.destroy_pipeline(self.pipeline, None);
        device.destroy_pipeline_layout(self.pipeline_layout, None);
        device.destroy_buffer(self.dot_buffer, None);
        device.free_memory(self.dot_memory, None);
        device.destroy_buffer(self.bars_buffer, None);
        device.free_memory(self.bars_memory, None);
    }

    pub unsafe fn record_commands(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        extent: vk::Extent2D,
        style: CrosshairStyle,
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
        let (buffer, count) = match style {
            CrosshairStyle::Dot => (self.dot_buffer, self.dot_count),
            CrosshairStyle::Bars => (self.bars_buffer, self.bars_count),
        };

        device.cmd_bind_vertex_buffers(command_buffer, 0, &[buffer], &offsets);
         
        let color = [1.0f32, 1.0, 1.0, 0.9];
        device.cmd_push_constants(
            command_buffer,
            self.pipeline_layout,
            vk::ShaderStageFlags::FRAGMENT,
            0,
            bytemuck::bytes_of(&color),
        );
        device.cmd_draw(command_buffer, count, 1, 0, 0);
    }
}

unsafe fn create_pipeline(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    msaa_samples: vk::SampleCountFlags,
) -> (vk::PipelineLayout, vk::Pipeline) {
    let vert_code =
        crate::vulkan::vk_pipeline::compile_shader("shader/menu.vert", shaderc::ShaderKind::Vertex);
    let frag_code =
        crate::vulkan::vk_pipeline::compile_shader("shader/menu.frag", shaderc::ShaderKind::Fragment);

    let vert_module = device
        .create_shader_module(
            &vk::ShaderModuleCreateInfo::default().code(&vert_code),
            None,
        )
        .unwrap();
    let frag_module = device
        .create_shader_module(
            &vk::ShaderModuleCreateInfo::default().code(&frag_code),
            None,
        )
        .unwrap();

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

    let vertex_binding = vk::VertexInputBindingDescription::default()
        .binding(0)
        .stride(std::mem::size_of::<[f32; 2]>() as u32)
        .input_rate(vk::VertexInputRate::VERTEX);
    let vertex_attribute = vk::VertexInputAttributeDescription::default()
        .location(0)
        .binding(0)
        .format(vk::Format::R32G32_SFLOAT)
        .offset(0);
    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(std::slice::from_ref(&vertex_binding))
        .vertex_attribute_descriptions(std::slice::from_ref(&vertex_attribute));

    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);

    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);
    let raster = vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .line_width(1.0);
    let multisample = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(msaa_samples);
    let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(false);
    let color_blend = vk::PipelineColorBlendStateCreateInfo::default()
        .attachments(std::slice::from_ref(&color_blend_attachment));
    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

    let push_range = vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        .offset(0)
        .size(std::mem::size_of::<[f32; 4]>() as u32);
    let layout_info = vk::PipelineLayoutCreateInfo::default()
        .push_constant_ranges(std::slice::from_ref(&push_range));
    let layout = device
        .create_pipeline_layout(&layout_info, None)
        .unwrap();

    let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&input_assembly)
        .viewport_state(&viewport_state)
        .rasterization_state(&raster)
        .multisample_state(&multisample)
        .color_blend_state(&color_blend)
        .dynamic_state(&dynamic)
        .layout(layout)
        .render_pass(render_pass);

    let pipeline = device
        .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
        .unwrap()[0];

    device.destroy_shader_module(vert_module, None);
    device.destroy_shader_module(frag_module, None);

    (layout, pipeline)
}

unsafe fn create_dot(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    scale: f32,
) -> (vk::Buffer, vk::DeviceMemory, u32) {
    let radius = 0.003 * scale;  
    let segments = 24;
    let mut verts = Vec::with_capacity(segments * 3);

    for i in 0..segments {
        let angle1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let angle2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

        verts.push([0.0, 0.0]);  
        verts.push([radius * angle1.cos(), radius * angle1.sin()]);
        verts.push([radius * angle2.cos(), radius * angle2.sin()]);
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

unsafe fn create_bars(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    scale: f32,
) -> (vk::Buffer, vk::DeviceMemory, u32) {
    let thickness = 0.003 * scale;
    let length = 0.02 * scale;
    let gap = 0.006 * scale;

    let mut verts: Vec<[f32; 2]> = Vec::new();
     
    verts.extend_from_slice(&quad(-gap - length, -thickness, -gap, thickness));
     
    verts.extend_from_slice(&quad(gap, -thickness, gap + length, thickness));
     
    verts.extend_from_slice(&quad(-thickness, gap, thickness, gap + length));
     
    verts.extend_from_slice(&quad(-thickness, -gap - length, thickness, -gap));

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

fn quad(left: f32, bottom: f32, right: f32, top: f32) -> [[f32; 2]; 6] {
    [
        [left, bottom],
        [right, bottom],
        [right, top],
        [left, bottom],
        [right, top],
        [left, top],
    ]
}
