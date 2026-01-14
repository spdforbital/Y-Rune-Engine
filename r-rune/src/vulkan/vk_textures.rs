use ash::vk;

use crate::vulkan::{vk_buffers, vk_memory};

pub struct Texture {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
    pub sampler: vk::Sampler,
}

pub unsafe fn load_texture(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    path: &str,
) -> Texture {
    let img = image::io::Reader::open(path)
        .expect("Failed to open texture file")
        .with_guessed_format()
        .expect("Failed to guess texture format")
        .decode()
        .expect("Failed to decode texture")
        .to_rgba8();  
    let (width, height) = img.dimensions();
    let pixels = img.into_raw();
    let image_size = pixels.len() as u64;

    let (staging_buffer, staging_memory) = vk_buffers::create_buffer(
        instance,
        pdevice,
        device,
        image_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        vk::MemoryAllocateFlags::empty(),
        Some(&pixels),
    );

    let (image, memory) = vk_memory::create_image(
        instance,
        pdevice,
        device,
        width,
        height,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
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

    let view_info = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(vk::Format::R8G8B8A8_UNORM)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1),
        );
    let view = device.create_image_view(&view_info, None).unwrap();

    let sampler_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::REPEAT)
        .address_mode_v(vk::SamplerAddressMode::REPEAT)
        .address_mode_w(vk::SamplerAddressMode::REPEAT)
        .anisotropy_enable(false)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
        .unnormalized_coordinates(false)
        .compare_enable(false)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR);
    let sampler = device.create_sampler(&sampler_info, None).unwrap();

    Texture {
        image,
        memory,
        view,
        sampler,
    }
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
    device.queue_wait_idle(queue).unwrap();
    device.free_command_buffers(pool, &[command_buffer]);
}

pub unsafe fn transition_image_layout(
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

pub unsafe fn copy_buffer_to_image(
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

pub unsafe fn create_storage_image_3d(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    width: u32,
    height: u32,
    depth: u32,
    format: vk::Format,
) -> Texture {
    let (image, memory) = vk_memory::create_image_3d(
        instance,
        pdevice,
        device,
        width,
        height,
        depth,
        format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );

    let view_info = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_3D)
        .format(format)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1),
        );
    let view = device.create_image_view(&view_info, None).unwrap();

    let sampler_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_BORDER)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_BORDER)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_BORDER)
        .border_color(vk::BorderColor::FLOAT_TRANSPARENT_BLACK)
        .anisotropy_enable(false)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR);
        
    let sampler = device.create_sampler(&sampler_info, None).unwrap();

    Texture {
        image,
        memory,
        view,
        sampler,
    }
}
