use ash::vk;
use rusttype::{Font, Scale, point};
use image::GenericImageView;

use crate::vulkan::{vk_buffers, vk_memory};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}

pub struct TextInstance {
    pub image: vk::Image,
    pub image_memory: vk::DeviceMemory,
    pub image_view: vk::ImageView,
    pub descriptor_set: vk::DescriptorSet,
    pub vertex_buffer: vk::Buffer,
    pub vertex_memory: vk::DeviceMemory,
    pub vertex_count: u32,
    pub text: String,
    pub position: [f32; 2],
    pub scale: f32,
    pub width: u32,
    pub height: u32,
    pub action: Option<String>,
}

impl TextInstance {
    pub unsafe fn destroy(&self, device: &ash::Device, pool: vk::DescriptorPool) {
        device.destroy_buffer(self.vertex_buffer, None);
        device.free_memory(self.vertex_memory, None);
        device.destroy_image_view(self.image_view, None);
        device.destroy_image(self.image, None);
        device.free_memory(self.image_memory, None);
         
        device.free_descriptor_sets(pool, &[self.descriptor_set]).ok(); 
    }
}

pub struct TextRenderer {
    font: Font<'static>,
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    pub sampler: vk::Sampler,
    pub instances: Vec<TextInstance>,
}

impl TextRenderer {
    pub fn new(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        font_path: &str,
        msaa_samples: vk::SampleCountFlags,
    ) -> Self {
        let font_data = std::fs::read(font_path).expect("Failed to load font file");
        let font = Font::try_from_vec(font_data).expect("Invalid font");

        let sampler = create_sampler(device);
        let descriptor_set_layout = create_descriptor_set_layout(device);
        let (pipeline_layout, pipeline) =
            create_pipeline(device, render_pass, descriptor_set_layout, msaa_samples);

        Self {
            font,
            descriptor_set_layout,
            pipeline_layout,
            pipeline,
            sampler,
            instances: Vec::new(),
        }
    }

    pub unsafe fn destroy(&mut self, device: &ash::Device, pool: vk::DescriptorPool) {
        for instance in &self.instances {
            instance.destroy(device, pool);
        }
        self.instances.clear();

        device.destroy_pipeline(self.pipeline, None);
        device.destroy_pipeline_layout(self.pipeline_layout, None);
        device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        device.destroy_sampler(self.sampler, None);
    }
    
    pub unsafe fn clear_instances(&mut self, device: &ash::Device, pool: vk::DescriptorPool) {
        for instance in &self.instances {
            instance.destroy(device, pool);
        }
        self.instances.clear();
    }

    unsafe fn build_instance(
        &self,
        instance: &ash::Instance,
        device: &ash::Device,
        pdevice: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        descriptor_pool: vk::DescriptorPool,
        swapchain_extent: vk::Extent2D,
        text: &str,
        position: [f32; 2],
        scale: f32,
        action: Option<String>,
    ) -> TextInstance {
        let (bitmap, width, height) = rasterize_text(&self.font, text, 54.0);
        
        let (image, image_memory) = vk_memory::create_image(
            instance,
            pdevice,
            device,
            width,
            height,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        upload_image(
            instance,
            device,
            pdevice,
            command_pool,
            queue,
            image,
            width,
            height,
            &bitmap,
        );

        let image_view = create_image_view(device, image, vk::Format::R8G8B8A8_UNORM);
        let descriptor_set = allocate_descriptor_set(
            device,
            descriptor_pool,
            self.descriptor_set_layout,
            image_view,
            self.sampler,
        );

        // Adjust quad vertices based on position and scale
        // Position is in NDC [-1, 1], but rasterize_text uses pixels.
        // We'll map pixel width/height to NDC width/height
        let _aspect = swapchain_extent.width as f32 / swapchain_extent.height as f32;
         
         
        
         
         
         
        
        let (vertex_buffer, vertex_memory, vertex_count) = create_quad_vertices(
            instance,
            pdevice,
            device,
            command_pool,
            queue,
            swapchain_extent,
            width as f32 * scale,  
            height as f32 * scale,
            position,
        );

        TextInstance {
            image,
            image_memory,
            image_view,
            descriptor_set,
            vertex_buffer,
            vertex_memory,
            vertex_count,
            text: text.to_string(),
            position,
            scale,
            width,
            height,
            action,
        }
    }

    unsafe fn build_image_instance(
        &self,
        instance: &ash::Instance,
        device: &ash::Device,
        pdevice: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        descriptor_pool: vk::DescriptorPool,
        swapchain_extent: vk::Extent2D,
        path: &str,
        position: [f32; 2],
        target_size_px: [f32; 2],
    ) -> TextInstance {
        let img = image::io::Reader::open(path)
            .expect("Failed to open image file")
            .with_guessed_format()
            .expect("Failed to guess image format")
            .decode()
            .expect("Failed to decode image")
            .to_rgba8();
        let (width, height) = img.dimensions();
        let pixels = img.into_raw();

        let (image, image_memory) = vk_memory::create_image(
            instance,
            pdevice,
            device,
            width,
            height,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        upload_image(
            instance,
            device,
            pdevice,
            command_pool,
            queue,
            image,
            width,
            height,
            &pixels,
        );

        let image_view = create_image_view(device, image, vk::Format::R8G8B8A8_UNORM);
        let descriptor_set = allocate_descriptor_set(
            device,
            descriptor_pool,
            self.descriptor_set_layout,
            image_view,
            self.sampler,
        );

        let scale = if target_size_px[0] > 0.0 && target_size_px[1] > 0.0 {
            let scale_w = target_size_px[0] / width as f32;
            let scale_h = target_size_px[1] / height as f32;
            scale_w.min(scale_h)
        } else {
            1.0
        };

        let (vertex_buffer, vertex_memory, vertex_count) = create_quad_vertices(
            instance,
            pdevice,
            device,
            command_pool,
            queue,
            swapchain_extent,
            width as f32 * scale,
            height as f32 * scale,
            position,
        );

        TextInstance {
            image,
            image_memory,
            image_view,
            descriptor_set,
            vertex_buffer,
            vertex_memory,
            vertex_count,
            text: format!("image:{}", path),
            position,
            scale,
            width,
            height,
            action: None,
        }
    }

    pub unsafe fn add_text(
        &mut self,
        instance: &ash::Instance,
        device: &ash::Device,
        pdevice: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        descriptor_pool: vk::DescriptorPool,
        swapchain_extent: vk::Extent2D,
        text: &str,
        position: [f32; 2],
        scale: f32,
        action: Option<String>,
    ) {
        let inst = self.build_instance(
            instance,
            device,
            pdevice,
            command_pool,
            queue,
            descriptor_pool,
            swapchain_extent,
            text,
            position,
            scale,
            action,
        );
        self.instances.push(inst);
    }

    pub unsafe fn add_image(
        &mut self,
        instance: &ash::Instance,
        device: &ash::Device,
        pdevice: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        descriptor_pool: vk::DescriptorPool,
        swapchain_extent: vk::Extent2D,
        path: &str,
        position: [f32; 2],
        target_size_px: [f32; 2],
    ) -> usize {
        let inst = self.build_image_instance(
            instance,
            device,
            pdevice,
            command_pool,
            queue,
            descriptor_pool,
            swapchain_extent,
            path,
            position,
            target_size_px,
        );
        self.instances.push(inst);
        self.instances.len() - 1
    }

    pub unsafe fn replace_text(
        &mut self,
        slot: Option<usize>,
        instance: &ash::Instance,
        device: &ash::Device,
        pdevice: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        descriptor_pool: vk::DescriptorPool,
        swapchain_extent: vk::Extent2D,
        text: &str,
        position: [f32; 2],
        scale: f32,
        action: Option<String>,
    ) -> usize {
        let inst = self.build_instance(
            instance,
            device,
            pdevice,
            command_pool,
            queue,
            descriptor_pool,
            swapchain_extent,
            text,
            position,
            scale,
            action,
        );
        if let Some(idx) = slot {
            if idx < self.instances.len() {
                device.device_wait_idle().unwrap();
                self.instances[idx].destroy(device, descriptor_pool);
                self.instances[idx] = inst;
                return idx;
            }
        }
        self.instances.push(inst);
        self.instances.len() - 1
    }

    pub unsafe fn replace_image(
        &mut self,
        slot: Option<usize>,
        instance: &ash::Instance,
        device: &ash::Device,
        pdevice: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        descriptor_pool: vk::DescriptorPool,
        swapchain_extent: vk::Extent2D,
        path: &str,
        position: [f32; 2],
        target_size_px: [f32; 2],
    ) -> usize {
         
        if let Some(idx) = slot {
            if idx < self.instances.len() {
                 
                 
                 
            }
        }

        let inst = self.build_image_instance(
            instance,
            device,
            pdevice,
            command_pool,
            queue,
            descriptor_pool,
            swapchain_extent,
            path,
            position,
            target_size_px,
        );
        if let Some(idx) = slot {
            if idx < self.instances.len() {
                device.device_wait_idle().unwrap();
                self.instances[idx].destroy(device, descriptor_pool);
                self.instances[idx] = inst;
                return idx;
            }
        }
        self.instances.push(inst);
        self.instances.len() - 1
    }

    pub unsafe fn update_position(
        &mut self,
        idx: usize,
        instance: &ash::Instance,
        device: &ash::Device,
        pdevice: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        swapchain_extent: vk::Extent2D,
        position: [f32; 2],
    ) {
        if idx >= self.instances.len() {
            return;
        }

        let old_instance = &mut self.instances[idx];
        
         
         
         
         
         
         
         
         

         
        let width = old_instance.width as f32;
        let height = old_instance.height as f32;
        let scale = old_instance.scale;

        let (vertex_buffer, vertex_memory, vertex_count) = create_quad_vertices(
            instance,
            pdevice,
            device,
            command_pool,
            queue,
            swapchain_extent,
            width * scale,
            height * scale,
            position,
        );
        
         
         
         
         
         
         
         
         
        
        device.destroy_buffer(old_instance.vertex_buffer, None);
        device.free_memory(old_instance.vertex_memory, None);

        old_instance.vertex_buffer = vertex_buffer;
        old_instance.vertex_memory = vertex_memory;
        old_instance.vertex_count = vertex_count;
        old_instance.position = position;
    }

    pub unsafe fn remove_text(
        &mut self,
        idx: usize,
        device: &ash::Device,
        descriptor_pool: vk::DescriptorPool,
    ) {
        if idx < self.instances.len() {
            device.device_wait_idle().unwrap();
            self.instances[idx].destroy(device, descriptor_pool);
            self.instances.remove(idx);
        }
    }

    pub unsafe fn record_commands(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        extent: vk::Extent2D,
    ) {
        device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline);

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

        for instance in &self.instances {
             device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                std::slice::from_ref(&instance.descriptor_set),
                &[],
            );

            let vb = [instance.vertex_buffer];
            let offsets = [0u64];
            device.cmd_bind_vertex_buffers(command_buffer, 0, &vb, &offsets);
            device.cmd_draw(command_buffer, instance.vertex_count, 1, 0, 0);
        }
    }

    pub unsafe fn record_commands_subset(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        extent: vk::Extent2D,
        indices: &[usize],
    ) {
        device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline);

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

        for &idx in indices {
            if idx >= self.instances.len() {
                continue;
            }
            let instance = &self.instances[idx];
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                std::slice::from_ref(&instance.descriptor_set),
                &[],
            );

            let vb = [instance.vertex_buffer];
            let offsets = [0u64];
            device.cmd_bind_vertex_buffers(command_buffer, 0, &vb, &offsets);
            device.cmd_draw(command_buffer, instance.vertex_count, 1, 0, 0);
        }
    }
}




fn rasterize_text(font: &Font<'_>, text: &str, px: f32) -> (Vec<u8>, u32, u32) {
    let scale = Scale::uniform(px);
    let v_metrics = font.v_metrics(scale);
    let line_gap = v_metrics.line_gap;
    let line_height = (v_metrics.ascent - v_metrics.descent + line_gap).ceil();

    let mut all_glyphs = Vec::new();
    let mut width: f32 = 0.0;

    for (line_idx, line) in text.split('\n').enumerate() {
        let y_offset = v_metrics.ascent + line_idx as f32 * line_height;
        let glyphs: Vec<_> = font.layout(line, scale, point(0.0, y_offset)).collect();
        for g in &glyphs {
            if let Some(bb) = g.pixel_bounding_box() {
                width = width.max(bb.max.x as f32);
            }
        }
        all_glyphs.extend(glyphs);
    }

    let width = width.ceil() as u32;
    let height = (line_height * text.lines().count() as f32).ceil() as u32;

    let mut buffer = vec![0u8; (width * height * 4) as usize];

    for glyph in all_glyphs {
        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                let gx = x as i32 + bb.min.x;
                let gy = y as i32 + bb.min.y;
                if gx >= 0 && gy >= 0 {
                    let x = gx as u32;
                    let y = gy as u32;
                    if x < width && y < height {
                        let idx = ((y * width + x) * 4) as usize;
                        buffer[idx] = 255;
                        buffer[idx + 1] = 255;
                        buffer[idx + 2] = 255;
                        buffer[idx + 3] = (v * 255.0) as u8;
                    }
                }
            });
        }
    }

    (buffer, width, height)
}

unsafe fn upload_image(
    instance: &ash::Instance,
    device: &ash::Device,
    pdevice: vk::PhysicalDevice,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    image: vk::Image,
    width: u32,
    height: u32,
    data: &[u8],
) {
    let buffer_size = data.len() as u64;
    let (staging_buffer, staging_memory) = vk_buffers::create_buffer(
        instance,
        pdevice,
        device,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        vk::MemoryAllocateFlags::empty(),
        Some(data),
    );

    transition_image_layout(
        device,
        command_pool,
        queue,
        image,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    );

    copy_buffer_to_image(
        device,
        command_pool,
        queue,
        staging_buffer,
        image,
        width,
        height,
    );

    transition_image_layout(
        device,
        command_pool,
        queue,
        image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_memory, None);
}

unsafe fn transition_image_layout(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
) {
    let command_buffer = begin_one_time_commands(device, command_pool);

    let (src_access_mask, dst_access_mask, src_stage, dst_stage) = match (old_layout, new_layout) {
        (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
            vk::AccessFlags::empty(),
            vk::AccessFlags::TRANSFER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
        ),
        (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
            vk::AccessFlags::TRANSFER_WRITE,
            vk::AccessFlags::SHADER_READ,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
        ),
        _ => panic!("Unsupported layout transition"),
    };

    let barrier = vk::ImageMemoryBarrier::default()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1),
        )
        .src_access_mask(src_access_mask)
        .dst_access_mask(dst_access_mask);

    device.cmd_pipeline_barrier(
        command_buffer,
        src_stage,
        dst_stage,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        std::slice::from_ref(&barrier),
    );

    end_one_time_commands(device, queue, command_pool, command_buffer);
}

unsafe fn copy_buffer_to_image(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    buffer: vk::Buffer,
    image: vk::Image,
    width: u32,
    height: u32,
) {
    let command_buffer = begin_one_time_commands(device, command_pool);
    let region = vk::BufferImageCopy::default()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(
            vk::ImageSubresourceLayers::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(0)
                .base_array_layer(0)
                .layer_count(1),
        )
        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
        .image_extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        });

    device.cmd_copy_buffer_to_image(
        command_buffer,
        buffer,
        image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        std::slice::from_ref(&region),
    );

    end_one_time_commands(device, queue, command_pool, command_buffer);
}

unsafe fn begin_one_time_commands(
    device: &ash::Device,
    pool: vk::CommandPool,
) -> vk::CommandBuffer {
    let alloc_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);
    let command_buffer = device.allocate_command_buffers(&alloc_info).unwrap()[0];
    let begin_info =
        vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    device
        .begin_command_buffer(command_buffer, &begin_info)
        .unwrap();
    command_buffer
}

unsafe fn end_one_time_commands(
    device: &ash::Device,
    queue: vk::Queue,
    pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
) {
    device.end_command_buffer(command_buffer).unwrap();
    let submit_info =
        vk::SubmitInfo::default().command_buffers(std::slice::from_ref(&command_buffer));
    device
        .queue_submit(queue, &[submit_info], vk::Fence::null())
        .unwrap();
    device.device_wait_idle().unwrap();
    device.free_command_buffers(pool, &[command_buffer]);
}

fn create_image_view(device: &ash::Device, image: vk::Image, format: vk::Format) -> vk::ImageView {
    let view_info = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1),
        );

    unsafe { device.create_image_view(&view_info, None).unwrap() }
}

fn create_sampler(device: &ash::Device) -> vk::Sampler {
    let sampler_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
        .unnormalized_coordinates(false)
        .compare_enable(false)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR);

    unsafe { device.create_sampler(&sampler_info, None).unwrap() }
}

fn create_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
    let binding = vk::DescriptorSetLayoutBinding::default()
        .binding(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);

    let layout_info =
        vk::DescriptorSetLayoutCreateInfo::default().bindings(std::slice::from_ref(&binding));
    unsafe {
        device
            .create_descriptor_set_layout(&layout_info, None)
            .unwrap()
    }
}

fn allocate_descriptor_set(
    device: &ash::Device,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    image_view: vk::ImageView,
    sampler: vk::Sampler,
) -> vk::DescriptorSet {
    let alloc_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(descriptor_pool)
        .set_layouts(std::slice::from_ref(&descriptor_set_layout));

    let descriptor_set = unsafe { device.allocate_descriptor_sets(&alloc_info).unwrap()[0] };

    let image_info = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(image_view)
        .sampler(sampler);

    let descriptor_write = vk::WriteDescriptorSet::default()
        .dst_set(descriptor_set)
        .dst_binding(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(std::slice::from_ref(&image_info));

    unsafe { device.update_descriptor_sets(std::slice::from_ref(&descriptor_write), &[]) };
    descriptor_set
}

fn create_pipeline(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    descriptor_set_layout: vk::DescriptorSetLayout,
    msaa_samples: vk::SampleCountFlags,
) -> (vk::PipelineLayout, vk::Pipeline) {
    let vert_code =
        crate::vulkan::vk_pipeline::compile_shader("shader/text.vert", shaderc::ShaderKind::Vertex);
    let frag_code = crate::vulkan::vk_pipeline::compile_shader(
        "shader/text.frag",
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
        .stride(std::mem::size_of::<TextVertex>() as u32)
        .input_rate(vk::VertexInputRate::VERTEX);
    let attribute_descs = [
        vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(0),
        vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(8),
    ];
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

    let layout_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(std::slice::from_ref(&descriptor_set_layout));
    let pipeline_layout = unsafe { device.create_pipeline_layout(&layout_info, None).unwrap() };

    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(false)
        .depth_write_enable(false)
        .depth_compare_op(vk::CompareOp::ALWAYS)
        .stencil_test_enable(false);

    let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&input_assembly)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterizer)
        .multisample_state(&multisampling)
        .depth_stencil_state(&depth_stencil)
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

unsafe fn create_quad_vertices(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    extent: vk::Extent2D,
    tex_width: f32,
    tex_height: f32,
    position: [f32; 2],
) -> (vk::Buffer, vk::DeviceMemory, u32) {
    // Convert NDC position to pixel coordinates for calculation?
    // Or just work in NDC. 
    // The current logic converts pixel width/height to NDC.
    
    // Center at 'position' (NDC)
    let half_w_ndc = tex_width / extent.width as f32;  
    let half_h_ndc = tex_height / extent.height as f32;

    let left = position[0] - half_w_ndc;
    let right = position[0] + half_w_ndc;
    let top = position[1] - half_h_ndc;
    let bottom = position[1] + half_h_ndc;

    let verts = [
        TextVertex {
            pos: [left, bottom],
            uv: [0.0, 1.0],
        },
        TextVertex {
            pos: [right, bottom],
            uv: [1.0, 1.0],
        },
        TextVertex {
            pos: [right, top],
            uv: [1.0, 0.0],
        },
        TextVertex {
            pos: [left, bottom],
            uv: [0.0, 1.0],
        },
        TextVertex {
            pos: [right, top],
            uv: [1.0, 0.0],
        },
        TextVertex {
            pos: [left, top],
            uv: [0.0, 0.0],
        },
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
