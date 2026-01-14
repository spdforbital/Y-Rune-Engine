use ash::vk;

pub unsafe fn create_command_pool(device: &ash::Device, queue_index: u32) -> vk::CommandPool {
    let create_info = vk::CommandPoolCreateInfo::default()
        .queue_family_index(queue_index)
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
    device.create_command_pool(&create_info, None).unwrap()
}

pub unsafe fn allocate_command_buffers(
    device: &ash::Device,
    pool: vk::CommandPool,
    count: u32,
) -> Vec<vk::CommandBuffer> {
    let alloc_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(count);
    device.allocate_command_buffers(&alloc_info).unwrap()
}

pub unsafe fn create_descriptor_pool(device: &ash::Device, max_frames: u32) -> vk::DescriptorPool {
    let pool_sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
             
            .descriptor_count(16 * max_frames),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(2 * max_frames + 1000),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
            .descriptor_count(max_frames + 64),
    ];

     
    let pool_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(&pool_sizes)
        .max_sets(max_frames + 1000)
        .max_sets(max_frames + 1000)
        .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET);

    device.create_descriptor_pool(&pool_info, None).unwrap()
}

pub unsafe fn create_ui_descriptor_pool(device: &ash::Device, max_frames: u32) -> vk::DescriptorPool {
    let pool_sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1000),  
    ];

    let pool_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(&pool_sizes)
        .max_sets(1000)
        .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET);

    device.create_descriptor_pool(&pool_info, None).unwrap()
}

pub unsafe fn allocate_descriptor_sets(
    device: &ash::Device,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    meshlet_buffer: vk::Buffer,
    vertex_buffer: vk::Buffer,
    meshlet_vertices_buffer: vk::Buffer,
    meshlet_triangles_buffer: vk::Buffer,
    textures: &[crate::vulkan::vk_textures::Texture],  
    hiz_view: vk::ImageView,
    hiz_sampler: vk::Sampler,
    tlas: vk::AccelerationStructureKHR,
    material_buffer: vk::Buffer,
    meshlet_params_buffer: vk::Buffer,
     
    light_buffer: vk::Buffer,
    cluster_info_buffer: vk::Buffer,
    light_index_buffer: vk::Buffer,
    max_frames: usize,
) -> Vec<vk::DescriptorSet> {
    let layouts = vec![descriptor_set_layout; max_frames];
    let alloc_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(descriptor_pool)
        .set_layouts(&layouts);

    let descriptor_sets = device.allocate_descriptor_sets(&alloc_info).unwrap();

    for i in 0..max_frames {
        let meshlet_info = vk::DescriptorBufferInfo::default()
            .buffer(meshlet_buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE);

        let vertex_info = vk::DescriptorBufferInfo::default()
            .buffer(vertex_buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE);

        let meshlet_vertices_info = vk::DescriptorBufferInfo::default()
            .buffer(meshlet_vertices_buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE);

        let meshlet_triangles_info = vk::DescriptorBufferInfo::default()
            .buffer(meshlet_triangles_buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE);

        let material_info = vk::DescriptorBufferInfo::default()
            .buffer(material_buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE);

        let meshlet_params_info = vk::DescriptorBufferInfo::default()
            .buffer(meshlet_params_buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE);

        let mut image_infos = Vec::with_capacity(textures.len());
        for tex in textures {
            image_infos.push(vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(tex.view)
                .sampler(tex.sampler));
        }

        let mut accel_info = vk::WriteDescriptorSetAccelerationStructureKHR::default()
            .acceleration_structures(std::slice::from_ref(&tlas));

        let hiz_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)  
              
             
            .image_view(hiz_view)
            .sampler(hiz_sampler);

        let light_info = vk::DescriptorBufferInfo::default()
            .buffer(light_buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE);
        
        let cluster_info_buf = vk::DescriptorBufferInfo::default()
            .buffer(cluster_info_buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE);
        
        let light_index_info = vk::DescriptorBufferInfo::default()
            .buffer(light_index_buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE);

         
         
        let writes = [
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets[i])
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(&meshlet_info)),
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets[i])
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(&vertex_info)),
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets[i])
                .dst_binding(2)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(&meshlet_vertices_info)),
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets[i])
                .dst_binding(3)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(&meshlet_triangles_info)),
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets[i])
                .dst_binding(4)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_infos),
             vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets[i])
                .dst_binding(7)
                .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                .descriptor_count(1)
                .push_next(&mut accel_info),  
             vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets[i])
                .dst_binding(8)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&hiz_info)),
             vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets[i])
                .dst_binding(10)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(&material_info)),
             vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets[i])
                .dst_binding(11)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(&light_info)),
             vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets[i])
                .dst_binding(12)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(&cluster_info_buf)),
             vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets[i])
                .dst_binding(13)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(&light_index_info)),
             vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets[i])
                .dst_binding(14)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(&meshlet_params_info)),
        ];
        device.update_descriptor_sets(&writes, &[]);
    }

    descriptor_sets
}

pub unsafe fn create_sync_objects(
    device: &ash::Device,
    max_frames: usize,
) -> (Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>) {
    let mut image_available = Vec::new();
    let mut render_finished = Vec::new();
    let mut in_flight = Vec::new();

    let semaphore_info = vk::SemaphoreCreateInfo::default();
    let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

    for _ in 0..max_frames {
        image_available.push(device.create_semaphore(&semaphore_info, None).unwrap());
        render_finished.push(device.create_semaphore(&semaphore_info, None).unwrap());
        in_flight.push(device.create_fence(&fence_info, None).unwrap());
    }

    (image_available, render_finished, in_flight)
}
