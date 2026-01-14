use ash::{ext, khr, vk};

 
pub unsafe fn create_device(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    queue_family_index: u32,
) -> (ash::Device, vk::Queue, vk::Queue) {
    let priorities = [1.0];
    let queue_info = vk::DeviceQueueCreateInfo::default()
        .queue_family_index(queue_family_index)
        .queue_priorities(&priorities);

    let device_extension_names_raw = [
        khr::swapchain::NAME.as_ptr(),
        ext::mesh_shader::NAME.as_ptr(),
        khr::acceleration_structure::NAME.as_ptr(),
        vk::KHR_RAY_QUERY_NAME.as_ptr(),
        vk::KHR_BUFFER_DEVICE_ADDRESS_NAME.as_ptr(),
        vk::KHR_DEFERRED_HOST_OPERATIONS_NAME.as_ptr(),
        vk::KHR_SPIRV_1_4_NAME.as_ptr(),
        vk::KHR_SHADER_FLOAT_CONTROLS_NAME.as_ptr(),
    ];

    let mut mesh_features = vk::PhysicalDeviceMeshShaderFeaturesEXT::default()
        .mesh_shader(true)
        .task_shader(true);
    let mut accel_features =
        vk::PhysicalDeviceAccelerationStructureFeaturesKHR::default().acceleration_structure(true);
    let mut ray_query_features =
        vk::PhysicalDeviceRayQueryFeaturesKHR::default().ray_query(true);
    let mut bda_features =
        vk::PhysicalDeviceBufferDeviceAddressFeatures::default().buffer_device_address(true);
    let mut indexing_features = vk::PhysicalDeviceDescriptorIndexingFeatures::default()
        .runtime_descriptor_array(true)
        .descriptor_binding_partially_bound(true)
        .shader_sampled_image_array_non_uniform_indexing(true);

    let mut features2 = vk::PhysicalDeviceFeatures2::default()
        .push_next(&mut mesh_features)
        .push_next(&mut accel_features)
        .push_next(&mut ray_query_features)
        .push_next(&mut bda_features)
        .push_next(&mut indexing_features);

    let device_create_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(std::slice::from_ref(&queue_info))
        .enabled_extension_names(&device_extension_names_raw)
        .push_next(&mut features2);

    let device = instance
        .create_device(pdevice, &device_create_info, None)
        .expect("Device creation failed");
    let graphics_queue = device.get_device_queue(queue_family_index, 0);
    let present_queue = device.get_device_queue(queue_family_index, 0);

    (device, graphics_queue, present_queue)
}
