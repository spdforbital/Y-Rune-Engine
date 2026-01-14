 
 
 
 
 

use ash::vk;
use crate::vulkan::{vk_memory, vk_pipeline};

 
pub(crate) struct HizResources {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
    pub mip_views: Vec<vk::ImageView>,
    pub mips: u32,
    pub sampler: vk::Sampler,
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
}

 
pub(crate) unsafe fn create_hiz_pipeline(
    device: &ash::Device,
) -> (vk::Pipeline, vk::PipelineLayout, vk::DescriptorSetLayout) {
    let (layout, dsl) = vk_pipeline::create_hiz_descriptor_set_layout(device);
    let pipeline = vk_pipeline::create_hiz_pipeline(device, layout);
    (pipeline, layout, dsl)
}

 
pub(crate) unsafe fn create_hiz_sampler(device: &ash::Device) -> vk::Sampler {
    let sampler_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::NEAREST)
        .min_filter(vk::Filter::NEAREST)
        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE);
    device.create_sampler(&sampler_info, None).unwrap()
}

 
pub(crate) unsafe fn create_hiz_image_and_views(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    descriptor_pool: vk::DescriptorPool,
    hiz_descriptor_set_layout: vk::DescriptorSetLayout,
    hiz_sampler: vk::Sampler,
    depth_view: vk::ImageView,
    swapchain_width: u32,
    swapchain_height: u32,
) -> (vk::Image, vk::DeviceMemory, vk::ImageView, Vec<vk::ImageView>, u32, Vec<vk::DescriptorSet>) {
    let width = (swapchain_width / 2).max(1);
    let height = (swapchain_height / 2).max(1);
    let hiz_mips = (std::cmp::max(width, height) as f32).log2().floor() as u32 + 1;

    let (hiz_image, hiz_memory) = vk_memory::create_image_mips(
        instance,
        physical_device,
        device,
        width,
        height,
        hiz_mips,
        vk::Format::R32_SFLOAT,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );

     
    let hiz_view_info = vk::ImageViewCreateInfo::default()
        .image(hiz_image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(vk::Format::R32_SFLOAT)
        .subresource_range(vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(hiz_mips)
            .base_array_layer(0)
            .layer_count(1));
    let hiz_view = device.create_image_view(&hiz_view_info, None).unwrap();

     
    let mut hiz_mip_views = Vec::with_capacity(hiz_mips as usize);
    for i in 0..hiz_mips {
        let view_info = vk::ImageViewCreateInfo::default()
            .image(hiz_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R32_SFLOAT)
            .subresource_range(vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(i)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1));
        let view = device.create_image_view(&view_info, None).unwrap();
        hiz_mip_views.push(view);
    }

     
    let mut hiz_descriptor_sets = Vec::new();
    if hiz_mips > 0 {
        let hiz_layouts = vec![hiz_descriptor_set_layout; hiz_mips as usize];
        let hiz_alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&hiz_layouts);
        hiz_descriptor_sets = device.allocate_descriptor_sets(&hiz_alloc_info).unwrap();

        for i in 0..hiz_descriptor_sets.len() {
            let input_view = if i == 0 { depth_view } else { hiz_mip_views[i - 1] };
            let output_view = hiz_mip_views[i];

            let image_info_input = vk::DescriptorImageInfo::default()
                .image_layout(if i == 0 { vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL } else { vk::ImageLayout::GENERAL })
                .image_view(input_view)
                .sampler(hiz_sampler);

            let image_info_output = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::GENERAL)
                .image_view(output_view);

            let writes = [
                vk::WriteDescriptorSet::default()
                    .dst_set(hiz_descriptor_sets[i])
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&image_info_input)),
                vk::WriteDescriptorSet::default()
                    .dst_set(hiz_descriptor_sets[i])
                    .dst_binding(1)
                    .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                    .image_info(std::slice::from_ref(&image_info_output)),
            ];
            device.update_descriptor_sets(&writes, &[]);
        }
    }

    (hiz_image, hiz_memory, hiz_view, hiz_mip_views, hiz_mips, hiz_descriptor_sets)
}

 
pub(crate) unsafe fn destroy_hiz_image_resources(
    device: &ash::Device,
    image: vk::Image,
    memory: vk::DeviceMemory,
    view: vk::ImageView,
    mip_views: &[vk::ImageView],
) {
    for &mip_view in mip_views {
        device.destroy_image_view(mip_view, None);
    }
    device.destroy_image_view(view, None);
    device.destroy_image(image, None);
    device.free_memory(memory, None);
}

 
pub(crate) unsafe fn free_hiz_descriptor_sets(
    device: &ash::Device,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: &[vk::DescriptorSet],
) {
    if !descriptor_sets.is_empty() {
        let _ = device.free_descriptor_sets(descriptor_pool, descriptor_sets);
    }
}
