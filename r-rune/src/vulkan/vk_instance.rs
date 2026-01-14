use ash::{khr, vk};
use std::ffi::CStr;
use winit::window::Window;

use raw_window_handle::HasDisplayHandle;

 
pub unsafe fn create_instance(entry: &ash::Entry, window: &Window) -> ash::Instance {
    let app_name = CStr::from_bytes_with_nul(b"Vulkan Meshlet Engine\0").unwrap();
    let app_info = vk::ApplicationInfo::default()
        .application_name(app_name)
        .application_version(0)
        .engine_name(app_name)
        .engine_version(0)
        .api_version(vk::make_api_version(0, 1, 3, 0));

    let extension_names =
        ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
            .unwrap()
            .to_vec();

    let create_info = vk::InstanceCreateInfo::default()
        .application_info(&app_info)
        .enabled_extension_names(&extension_names);

    entry
        .create_instance(&create_info, None)
        .expect("Instance creation error")
}

 
pub unsafe fn pick_physical_device(
    instance: &ash::Instance,
    surface_loader: &khr::surface::Instance,
    surface: vk::SurfaceKHR,
) -> (vk::PhysicalDevice, u32) {
    let pdevices = instance
        .enumerate_physical_devices()
        .expect("Physical device error");
    for &pdevice in pdevices.iter() {
        let properties = instance.get_physical_device_properties(pdevice);
        let mut mesh_features = vk::PhysicalDeviceMeshShaderFeaturesEXT::default();
        let mut accel_features = vk::PhysicalDeviceAccelerationStructureFeaturesKHR::default();
        let mut ray_query_features = vk::PhysicalDeviceRayQueryFeaturesKHR::default();
        let mut bda_features = vk::PhysicalDeviceBufferDeviceAddressFeatures::default();
        let mut features2 = vk::PhysicalDeviceFeatures2::default()
            .push_next(&mut mesh_features)
            .push_next(&mut accel_features)
            .push_next(&mut ray_query_features)
            .push_next(&mut bda_features);
        instance.get_physical_device_features2(pdevice, &mut features2);

        if mesh_features.mesh_shader == vk::TRUE
            && mesh_features.task_shader == vk::TRUE
            && accel_features.acceleration_structure == vk::TRUE
            && ray_query_features.ray_query == vk::TRUE
            && bda_features.buffer_device_address == vk::TRUE
        {
            let queue_families = instance.get_physical_device_queue_family_properties(pdevice);
            for (i, info) in queue_families.iter().enumerate() {
                if info.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    if surface_loader
                        .get_physical_device_surface_support(pdevice, i as u32, surface)
                        .unwrap()
                    {
                        println!(
                            "Selected GPU: {:?}",
                            CStr::from_ptr(properties.device_name.as_ptr())
                        );
                        return (pdevice, i as u32);
                    }
                }
            }
        }
    }
    panic!("No suitable GPU found (Mesh Shader required)!");
}
