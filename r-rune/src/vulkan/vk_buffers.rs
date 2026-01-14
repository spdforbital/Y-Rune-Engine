use ash::vk;

use crate::vulkan::vk_memory::find_memory_type;

pub unsafe fn create_buffer(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
    alloc_flags: vk::MemoryAllocateFlags,
    data: Option<&[u8]>,
) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer_info = vk::BufferCreateInfo::default()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = device.create_buffer(&buffer_info, None).unwrap();

    let mem_requirements = device.get_buffer_memory_requirements(buffer);
    let memory_type = find_memory_type(
        instance,
        pdevice,
        mem_requirements.memory_type_bits,
        properties,
    );

    let alloc_info = vk::MemoryAllocateInfo::default()
        .allocation_size(mem_requirements.size)
        .memory_type_index(memory_type);

    let mut alloc_info = alloc_info;
    let mut flags_info = vk::MemoryAllocateFlagsInfo::default().flags(alloc_flags);
    if alloc_flags != vk::MemoryAllocateFlags::empty() {
        alloc_info = alloc_info.push_next(&mut flags_info);
    }

    let buffer_memory = device.allocate_memory(&alloc_info, None).unwrap();
    device.bind_buffer_memory(buffer, buffer_memory, 0).unwrap();

    if let Some(d) = data {
        let ptr = device
            .map_memory(buffer_memory, 0, size, vk::MemoryMapFlags::empty())
            .unwrap();
        let slice = std::slice::from_raw_parts_mut(ptr as *mut u8, size as usize);
        slice[0..d.len()].copy_from_slice(d);
        device.unmap_memory(buffer_memory);
    }

    (buffer, buffer_memory)
}

pub unsafe fn begin_single_time_commands(
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

pub unsafe fn end_single_time_commands(
    device: &ash::Device,
    queue: vk::Queue,
    pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
) {
    device.end_command_buffer(command_buffer).unwrap();

    let command_buffers = [command_buffer];
    let submit_info = vk::SubmitInfo::default().command_buffers(&command_buffers);
    device
        .queue_submit(queue, &[submit_info], vk::Fence::null())
        .unwrap();
    device.device_wait_idle().unwrap();

    device.free_command_buffers(pool, &command_buffers);
}

pub unsafe fn copy_buffer(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    src: vk::Buffer,
    dst: vk::Buffer,
    size: vk::DeviceSize,
) {
    let command_buffer = begin_single_time_commands(device, command_pool);
    let copy_region = vk::BufferCopy::default().size(size);
    device.cmd_copy_buffer(command_buffer, src, dst, std::slice::from_ref(&copy_region));
    end_single_time_commands(device, queue, command_pool, command_buffer);
}

pub unsafe fn create_device_local_buffer_with_data(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    data: &[u8],
    usage: vk::BufferUsageFlags,
) -> (vk::Buffer, vk::DeviceMemory) {
    create_device_local_buffer_with_data_flags(
        instance,
        pdevice,
        device,
        command_pool,
        queue,
        data,
        usage,
        vk::MemoryAllocateFlags::empty(),
    )
}

pub unsafe fn create_device_local_buffer_with_data_flags(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    data: &[u8],
    usage: vk::BufferUsageFlags,
    alloc_flags: vk::MemoryAllocateFlags,
) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer_size = data.len() as u64;

    let (staging_buffer, staging_memory) = create_buffer(
        instance,
        pdevice,
        device,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        vk::MemoryAllocateFlags::empty(),
        Some(data),
    );

    let (device_buffer, device_memory) = create_buffer(
        instance,
        pdevice,
        device,
        buffer_size,
        usage | vk::BufferUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        alloc_flags,
        None,
    );

    copy_buffer(
        device,
        command_pool,
        queue,
        staging_buffer,
        device_buffer,
        buffer_size,
    );

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_memory, None);

    (device_buffer, device_memory)
}
