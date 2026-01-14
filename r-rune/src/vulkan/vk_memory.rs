use ash::vk;

pub struct DepthResources {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
    pub format: vk::Format,
}

pub unsafe fn find_memory_type(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    type_filter: u32,
    properties: vk::MemoryPropertyFlags,
) -> u32 {
    let mem_properties = instance.get_physical_device_memory_properties(pdevice);
    for i in 0..mem_properties.memory_type_count {
        if (type_filter & (1 << i)) != 0
            && (mem_properties.memory_types[i as usize].property_flags & properties) == properties
        {
            return i;
        }
    }
    panic!("Failed to find suitable memory type!");
}

pub unsafe fn find_supported_format(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    candidates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> vk::Format {
    for &format in candidates {
        let props = instance.get_physical_device_format_properties(pdevice, format);
        if tiling == vk::ImageTiling::LINEAR && props.linear_tiling_features.contains(features) {
            return format;
        } else if tiling == vk::ImageTiling::OPTIMAL
            && props.optimal_tiling_features.contains(features)
        {
            return format;
        }
    }
    panic!("Failed to find supported format");
}

pub unsafe fn find_depth_format(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
) -> vk::Format {
    find_supported_format(
        instance,
        pdevice,
        &[
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ],
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    )
}

pub unsafe fn create_image(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    width: u32,
    height: u32,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> (vk::Image, vk::DeviceMemory) {
    create_image_with_samples(
        instance,
        pdevice,
        device,
        width,
        height,
        format,
        tiling,
        usage,
        properties,
        vk::SampleCountFlags::TYPE_1,
    )
}

pub unsafe fn create_image_with_samples(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    width: u32,
    height: u32,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
    samples: vk::SampleCountFlags,
) -> (vk::Image, vk::DeviceMemory) {
    let image_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .samples(samples)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let image = device.create_image(&image_info, None).unwrap();
    let mem_requirements = device.get_image_memory_requirements(image);
    let memory_type = find_memory_type(
        instance,
        pdevice,
        mem_requirements.memory_type_bits,
        properties,
    );
    let alloc_info = vk::MemoryAllocateInfo::default()
        .allocation_size(mem_requirements.size)
        .memory_type_index(memory_type);

    let image_memory = device.allocate_memory(&alloc_info, None).unwrap();
    device.bind_image_memory(image, image_memory, 0).unwrap();

    (image, image_memory)
}

pub unsafe fn create_image_mips(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    width: u32,
    height: u32,
    mip_levels: u32,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> (vk::Image, vk::DeviceMemory) {
    let image_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(mip_levels)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .samples(vk::SampleCountFlags::TYPE_1)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let image = device.create_image(&image_info, None).unwrap();
    let mem_requirements = device.get_image_memory_requirements(image);
    let memory_type = find_memory_type(
        instance,
        pdevice,
        mem_requirements.memory_type_bits,
        properties,
    );
    let alloc_info = vk::MemoryAllocateInfo::default()
        .allocation_size(mem_requirements.size)
        .memory_type_index(memory_type);

    let image_memory = device.allocate_memory(&alloc_info, None).unwrap();
    device.bind_image_memory(image, image_memory, 0).unwrap();

    (image, image_memory)
}

pub unsafe fn create_depth_resources(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    extent: vk::Extent2D,
) -> DepthResources {
    create_depth_resources_with_samples(
        instance,
        pdevice,
        device,
        extent,
        vk::SampleCountFlags::TYPE_1,
    )
}

pub unsafe fn create_depth_resources_with_samples(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    extent: vk::Extent2D,
    samples: vk::SampleCountFlags,
) -> DepthResources {
    let depth_format = find_depth_format(instance, pdevice);
    let (image, memory) = create_image_with_samples(
        instance,
        pdevice,
        device,
        extent.width,
        extent.height,
        depth_format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        samples,
    );

    let view_info = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(depth_format)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::DEPTH)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1),
        );

    let view = device.create_image_view(&view_info, None).unwrap();
    DepthResources {
        image,
        memory,
        view,
        format: depth_format,
    }
}

pub unsafe fn destroy_depth_resources(device: &ash::Device, depth: &DepthResources) {
    device.destroy_image_view(depth.view, None);
    device.destroy_image(depth.image, None);
    device.free_memory(depth.memory, None);
}

pub unsafe fn create_image_3d(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    width: u32,
    height: u32,
    depth: u32,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> (vk::Image, vk::DeviceMemory) {
    let image_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_3D)
        .extent(vk::Extent3D {
            width,
            height,
            depth,
        })
        .mip_levels(1)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .samples(vk::SampleCountFlags::TYPE_1)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let image = device.create_image(&image_info, None).unwrap();
    let mem_requirements = device.get_image_memory_requirements(image);
    let memory_type = find_memory_type(
        instance,
        pdevice,
        mem_requirements.memory_type_bits,
        properties,
    );
    let alloc_info = vk::MemoryAllocateInfo::default()
        .allocation_size(mem_requirements.size)
        .memory_type_index(memory_type);

    let image_memory = device.allocate_memory(&alloc_info, None).unwrap();
    device.bind_image_memory(image, image_memory, 0).unwrap();

    (image, image_memory)
}
