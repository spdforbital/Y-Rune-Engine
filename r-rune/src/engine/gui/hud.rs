use ash::vk;
use crate::vulkan::vk_buffers;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HudVertex {
    pub pos: [f32; 2],
}

pub struct HudRenderer {
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    
     
    health_bar_bg_buffer: vk::Buffer,
    health_bar_bg_memory: vk::DeviceMemory,
    health_bar_bg_count: u32,
    health_bar_fg_buffer: vk::Buffer,
    health_bar_fg_memory: vk::DeviceMemory,
    health_bar_fg_count: u32,

     
    inv_panel_buffer: vk::Buffer,
    inv_panel_memory: vk::DeviceMemory,
    inv_panel_count: u32,
    inv_slots_buffer: vk::Buffer,
    inv_slots_memory: vk::DeviceMemory,
    inv_slots_count: u32,

     
    hotbar_slots_buffer: vk::Buffer,
    hotbar_slots_memory: vk::DeviceMemory,
    hotbar_slots_count: u32,
}

pub const INV_ROWS: usize = 4;
pub const INV_COLS: usize = 6;
pub const INV_SLOT_SIZE: f32 = 0.15;
pub const INV_SLOT_GAP: f32 = 0.02;

impl HudRenderer {
    pub fn new(
        instance: &ash::Instance,
        device: &ash::Device,
        pdevice: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        render_pass: vk::RenderPass,
        msaa_samples: vk::SampleCountFlags,
    ) -> Self {
        let (pipeline_layout, pipeline) = create_pipeline(device, render_pass, msaa_samples);

        let (health_bar_bg_buffer, health_bar_bg_memory, health_bar_bg_count) =
            unsafe { create_health_bar_bg(instance, pdevice, device, command_pool, queue) };
        
        let (health_bar_fg_buffer, health_bar_fg_memory, health_bar_fg_count) =
            unsafe { create_health_bar_fg(instance, pdevice, device, command_pool, queue) };

        let (inv_panel_buffer, inv_panel_memory, inv_panel_count) =
            unsafe { create_inv_panel(instance, pdevice, device, command_pool, queue) };

        let (inv_slots_buffer, inv_slots_memory, inv_slots_count) =
            unsafe { create_inv_slots(instance, pdevice, device, command_pool, queue) };

        let (hotbar_slots_buffer, hotbar_slots_memory, hotbar_slots_count) =
            unsafe { create_hotbar_slots(instance, pdevice, device, command_pool, queue) };

        Self {
            pipeline_layout,
            pipeline,
            health_bar_bg_buffer,
            health_bar_bg_memory,
            health_bar_bg_count,
            health_bar_fg_buffer,
            health_bar_fg_memory,
            health_bar_fg_count,
            inv_panel_buffer,
            inv_panel_memory,
            inv_panel_count,
            inv_slots_buffer,
            inv_slots_memory,
            inv_slots_count,
            hotbar_slots_buffer,
            hotbar_slots_memory,
            hotbar_slots_count,
        }
    }

    pub unsafe fn destroy(&self, device: &ash::Device) {
        device.destroy_pipeline(self.pipeline, None);
        device.destroy_pipeline_layout(self.pipeline_layout, None);
        
        device.destroy_buffer(self.health_bar_bg_buffer, None);
        device.free_memory(self.health_bar_bg_memory, None);
        device.destroy_buffer(self.health_bar_fg_buffer, None);
        device.free_memory(self.health_bar_fg_memory, None);

        device.destroy_buffer(self.inv_panel_buffer, None);
        device.free_memory(self.inv_panel_memory, None);
        device.destroy_buffer(self.inv_slots_buffer, None);
        device.free_memory(self.inv_slots_memory, None);

        device.destroy_buffer(self.hotbar_slots_buffer, None);
        device.free_memory(self.hotbar_slots_memory, None);
    }

    pub unsafe fn record_commands(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        extent: vk::Extent2D,
        health_percentage: f32,
        show_inventory: bool,
        active_hotbar_slot: usize,
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

         
         
        device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.health_bar_bg_buffer], &offsets);
        device.cmd_push_constants(
            command_buffer,
            self.pipeline_layout,
            vk::ShaderStageFlags::FRAGMENT,
            0,
            bytemuck::bytes_of(&[0.1f32, 0.1, 0.1, 0.8]),  
        );
        device.cmd_draw(command_buffer, self.health_bar_bg_count, 1, 0, 0);

         
         
         
         
         
         
        let bar_left_ndc = -0.95;
        let bar_width_ndc = 0.3;
        let bar_bottom_ndc = 0.85;  
         
         
         
         
         
         
        
        let width_pixels = extent.width as f32;
        let height_pixels = extent.height as f32;
        
         
         
         
        
         
         
         
        
        let current_health_fraction = health_percentage / 100.0;
        
         
        let clip_x = (-0.95 + 1.0) * 0.5 * width_pixels;
        let clip_w = (0.3 * current_health_fraction) * 0.5 * width_pixels;
        let clip_y = (0.85 + 1.0) * 0.5 * height_pixels;
        let clip_h = (0.07) * 0.5 * height_pixels;   
        
         
         
         
        
         
        device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.health_bar_fg_buffer], &offsets);
        
         
        let health_scissor = vk::Rect2D {
            offset: vk::Offset2D { 
                x: clip_x as i32, 
                 
                 
                 
                y: 0, 
            },
            extent: vk::Extent2D {
                width: clip_w as u32,
                height: extent.height,  
            }
        };
        device.cmd_set_scissor(command_buffer, 0, std::slice::from_ref(&health_scissor));

        device.cmd_push_constants(
            command_buffer,
            self.pipeline_layout,
            vk::ShaderStageFlags::FRAGMENT,
            0,
            bytemuck::bytes_of(&[0.2f32, 0.8, 0.2, 1.0]),  
        );
        device.cmd_draw(command_buffer, self.health_bar_fg_count, 1, 0, 0);

         
        device.cmd_set_scissor(command_buffer, 0, std::slice::from_ref(&scissor));

         
        if show_inventory {
             
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.inv_panel_buffer], &offsets);
            device.cmd_push_constants(
                command_buffer,
                self.pipeline_layout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&[0.0f32, 0.0, 0.0, 0.9]),  
            );
            device.cmd_draw(command_buffer, self.inv_panel_count, 1, 0, 0);

             
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.inv_slots_buffer], &offsets);
            device.cmd_push_constants(
                command_buffer,
                self.pipeline_layout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&[0.35f32, 0.35, 0.35, 0.9]),  
            );
            device.cmd_draw(command_buffer, self.inv_slots_count, 1, 0, 0);
        }

         
         
        
         
        device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.hotbar_slots_buffer], &offsets);
        device.cmd_push_constants(
            command_buffer,
            self.pipeline_layout,
            vk::ShaderStageFlags::FRAGMENT,
            0,
            bytemuck::bytes_of(&[0.2f32, 0.2, 0.2, 0.8]),  
        );
        device.cmd_draw(command_buffer, self.hotbar_slots_count, 1, 0, 0);

         
         
         
         
         
         
         
         
         
         
         
        
         
         
         
        let vertex_offset = (active_hotbar_slot as u32) * 6;
        if vertex_offset < self.hotbar_slots_count {
              
              
             device.cmd_push_constants(
                command_buffer,
                self.pipeline_layout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&[0.6f32, 0.6, 0.2, 0.5]),  
            );
            device.cmd_draw(command_buffer, 6, 1, vertex_offset, 0);
        }
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
        device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&vert_code), None).unwrap()
    };
    let frag_module = unsafe {
        device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&frag_code), None).unwrap()
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
        .stride(std::mem::size_of::<HudVertex>() as u32)
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
        device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None).unwrap()[0]
    };

    unsafe {
        device.destroy_shader_module(vert_module, None);
        device.destroy_shader_module(frag_module, None);
    }

    (pipeline_layout, pipeline)
}

 

unsafe fn create_buffer_helper(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    verts: &[HudVertex],
) -> (vk::Buffer, vk::DeviceMemory, u32) {
    let (buffer, memory) = vk_buffers::create_device_local_buffer_with_data(
        instance,
        pdevice,
        device,
        command_pool,
        queue,
        bytemuck::cast_slice(verts),
        vk::BufferUsageFlags::VERTEX_BUFFER,
    );
    (buffer, memory, verts.len() as u32)
}

unsafe fn create_health_bar_bg(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
) -> (vk::Buffer, vk::DeviceMemory, u32) {
    let left = -0.96;
    let right = -0.64;
    let top = 0.93;  
    let bottom = 0.84;
    
     
     

    let verts = [
        HudVertex { pos: [left, bottom] },
        HudVertex { pos: [right, bottom] },
        HudVertex { pos: [right, top] },
        HudVertex { pos: [left, bottom] },
        HudVertex { pos: [right, top] },
        HudVertex { pos: [left, top] },
    ];
    create_buffer_helper(instance, pdevice, device, command_pool, queue, &verts)
}

unsafe fn create_health_bar_fg(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
) -> (vk::Buffer, vk::DeviceMemory, u32) {
     
    let left = -0.95;
    let right = -0.65;
    let top = 0.92;
    let bottom = 0.85;

    let verts = [
        HudVertex { pos: [left, bottom] },
        HudVertex { pos: [right, bottom] },
        HudVertex { pos: [right, top] },
        HudVertex { pos: [left, bottom] },
        HudVertex { pos: [right, top] },
        HudVertex { pos: [left, top] },
    ];
    create_buffer_helper(instance, pdevice, device, command_pool, queue, &verts)
}

unsafe fn create_inv_panel(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
) -> (vk::Buffer, vk::DeviceMemory, u32) {
     
    let left = -0.6;
    let right = 0.6;
    let top = -0.5;
    let bottom = 0.5;

    let verts = [
        HudVertex { pos: [left, bottom] },
        HudVertex { pos: [right, bottom] },
        HudVertex { pos: [right, top] },
        HudVertex { pos: [left, bottom] },
        HudVertex { pos: [right, top] },
        HudVertex { pos: [left, top] },
    ];
    create_buffer_helper(instance, pdevice, device, command_pool, queue, &verts)
}

unsafe fn create_inv_slots(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
) -> (vk::Buffer, vk::DeviceMemory, u32) {
    let rows = INV_ROWS;
    let cols = INV_COLS;
    let slot_size = INV_SLOT_SIZE;
    let gap = INV_SLOT_GAP;
    
     
    let total_w = cols as f32 * slot_size + (cols as f32 - 1.0) * gap;
    let total_h = rows as f32 * slot_size + (rows as f32 - 1.0) * gap;
    
    let start_x = -total_w * 0.5;
    let start_y = -total_h * 0.5;

    let mut verts = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let x0 = start_x + c as f32 * (slot_size + gap);
            let y0 = start_y + r as f32 * (slot_size + gap);
            let x1 = x0 + slot_size;
            let y1 = y0 + slot_size;
            
            verts.push(HudVertex { pos: [x0, y0] });
            verts.push(HudVertex { pos: [x1, y0] });
            verts.push(HudVertex { pos: [x1, y1] });
            verts.push(HudVertex { pos: [x0, y0] });
            verts.push(HudVertex { pos: [x1, y1] });
            verts.push(HudVertex { pos: [x0, y1] });
        }
    }
    create_buffer_helper(instance, pdevice, device, command_pool, queue, &verts)
}

unsafe fn create_hotbar_slots(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
) -> (vk::Buffer, vk::DeviceMemory, u32) {
    let slots = 5;
    let slot_size = 0.12;  
    let gap = 0.02;
    
     
     
    let total_w = slots as f32 * slot_size + (slots as f32 - 1.0) * gap;
    let bottom_y = 0.95;  
    let start_x = -total_w * 0.5;
    let start_y = bottom_y - slot_size;

    let mut verts = Vec::new();
    for i in 0..slots {
        let x0 = start_x + i as f32 * (slot_size + gap);
        let y0 = start_y;
        let x1 = x0 + slot_size;
        let y1 = y0 + slot_size;
        
        verts.push(HudVertex { pos: [x0, y0] });
        verts.push(HudVertex { pos: [x1, y0] });
        verts.push(HudVertex { pos: [x1, y1] });
        verts.push(HudVertex { pos: [x0, y0] });
        verts.push(HudVertex { pos: [x1, y1] });
        verts.push(HudVertex { pos: [x0, y1] });
    }
    create_buffer_helper(instance, pdevice, device, command_pool, queue, &verts)
}
