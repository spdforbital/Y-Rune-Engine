use ash::{khr, vk};

 
pub unsafe fn create_swapchain(
    instance: &ash::Instance,
    device: &ash::Device,
    pdevice: vk::PhysicalDevice,
    surface_loader: &khr::surface::Instance,
    surface: vk::SurfaceKHR,
    _queue_family_index: u32,
    width: u32,
    height: u32,
) -> (
    khr::swapchain::Device,
    vk::SwapchainKHR,
    vk::Format,
    vk::Extent2D,
    Vec<vk::Image>,
) {
    let capabilities = surface_loader
        .get_physical_device_surface_capabilities(pdevice, surface)
        .unwrap();
    let formats = surface_loader
        .get_physical_device_surface_formats(pdevice, surface)
        .unwrap();

    let format = formats
        .iter()
        .find(|f| {
            f.format == vk::Format::B8G8R8A8_SRGB
                && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or(&formats[0]);

    let present_modes = surface_loader
        .get_physical_device_surface_present_modes(pdevice, surface)
        .unwrap();
    let present_mode = present_modes
        .iter()
        .cloned()
        .find(|&p| p == vk::PresentModeKHR::IMMEDIATE)
        .unwrap_or(vk::PresentModeKHR::FIFO);

    let extent = if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        vk::Extent2D {
            width: width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    };

    let mut image_count = capabilities.min_image_count + 1;
    if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count {
        image_count = capabilities.max_image_count;
    }

    let create_info = vk::SwapchainCreateInfoKHR::default()
        .surface(surface)
        .min_image_count(image_count)
        .image_format(format.format)
        .image_color_space(format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .pre_transform(capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true);

    let swapchain_loader = khr::swapchain::Device::new(instance, device);
    let swapchain = swapchain_loader
        .create_swapchain(&create_info, None)
        .expect("Swapchain creation failed");
    let images = swapchain_loader.get_swapchain_images(swapchain).unwrap();

    (swapchain_loader, swapchain, format.format, extent, images)
}

pub unsafe fn create_image_views(
    device: &ash::Device,
    images: &[vk::Image],
    format: vk::Format,
) -> Vec<vk::ImageView> {
    images
        .iter()
        .map(|&image| {
            let create_info = vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            device.create_image_view(&create_info, None).unwrap()
        })
        .collect()
}
