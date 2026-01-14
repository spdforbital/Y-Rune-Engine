 
 
 

use std::sync::Arc;

use ash::{khr, vk};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder, Fullscreen, CursorGrabMode},
};

use crate::vulkan::{vk_instance, vk_pipeline, vk_swapchain};
use super::{WIDTH, HEIGHT};

 
pub fn init_window(event_loop: &EventLoop<()>, fullscreen: bool) -> Arc<Window> {
    let mut builder = WindowBuilder::new()
        .with_title("Vulkan Meshlet Engine")
        .with_inner_size(winit::dpi::LogicalSize::new(WIDTH, HEIGHT));
        
    if fullscreen {
        builder = builder.with_fullscreen(Some(Fullscreen::Borderless(None)));
    }
        
    let window = Arc::new(builder.build(event_loop).expect("Failed to create window"));
    
     
    let _ = window.set_cursor_grab(CursorGrabMode::Confined)
        .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked));
    window.set_cursor_visible(false);
    
    window
}

 
pub(crate) unsafe fn init_instance(
    window: &Window,
) -> (
    ash::Entry,
    ash::Instance,
    khr::surface::Instance,
    vk::SurfaceKHR,
) {
    let entry = ash::Entry::linked();
    let instance = vk_instance::create_instance(&entry, window);
    let surface_loader = khr::surface::Instance::new(&entry, &instance);
    let surface = ash_window::create_surface(
        &entry,
        &instance,
        window.display_handle().unwrap().as_raw(),
        window.window_handle().unwrap().as_raw(),
        None,
    )
    .unwrap();
    (entry, instance, surface_loader, surface)
}

 
pub(crate) unsafe fn init_swapchain(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    surface_loader: &khr::surface::Instance,
    surface: vk::SurfaceKHR,
    queue_family_index: u32,
    window: &Window,
) -> (
    khr::swapchain::Device,
    vk::SwapchainKHR,
    vk::Format,
    vk::Extent2D,
    Vec<vk::Image>,
    Vec<vk::ImageView>,
) {
    let size = window.inner_size();
    let (swapchain_loader, swapchain, format, extent, images) = vk_swapchain::create_swapchain(
        instance,
        device,
        physical_device,
        surface_loader,
        surface,
        queue_family_index,
        size.width,
        size.height,
    );
    let image_views = vk_swapchain::create_image_views(device, &images, format);
    (
        swapchain_loader,
        swapchain,
        format,
        extent,
        images,
        image_views,
    )
}

 
pub(crate) unsafe fn init_pipeline(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    msaa_samples: vk::SampleCountFlags,
) -> (vk::Pipeline, vk::PipelineLayout, vk::DescriptorSetLayout) {
    let (pipeline_layout, descriptor_set_layout) =
        vk_pipeline::create_descriptor_set_layout(device);
    let pipeline = vk_pipeline::create_pipeline(device, pipeline_layout, render_pass, msaa_samples);
    (pipeline, pipeline_layout, descriptor_set_layout)
}

 
pub(crate) unsafe fn init_skinned_pipeline(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    msaa_samples: vk::SampleCountFlags,
) -> (vk::Pipeline, vk::PipelineLayout, vk::DescriptorSetLayout) {
    let (layout, dsl) = vk_pipeline::create_skinned_descriptor_set_layout(device);
    let pipeline = vk_pipeline::create_skinned_pipeline(device, layout, render_pass, msaa_samples);
    (pipeline, layout, dsl)
}
