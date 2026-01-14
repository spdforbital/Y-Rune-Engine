use ash::{khr, vk};

use crate::vulkan::vk_buffers;

pub struct Blas {
    pub accel: vk::AccelerationStructureKHR,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

pub struct RayTracingScene {
    pub accel: khr::acceleration_structure::Device,
    pub blas: vk::AccelerationStructureKHR,  
    pub tlas: vk::AccelerationStructureKHR,
    pub blas_buffer: vk::Buffer,
    pub blas_memory: vk::DeviceMemory,
    pub tlas_buffer: vk::Buffer,
    pub tlas_memory: vk::DeviceMemory,
    pub instance_buffer: vk::Buffer,
    pub instance_memory: vk::DeviceMemory,
}

impl RayTracingScene {
    pub unsafe fn destroy(&self, device: &ash::Device) {
        if self.blas != vk::AccelerationStructureKHR::null() {
            self.accel.destroy_acceleration_structure(self.blas, None);
        }
        if self.tlas != vk::AccelerationStructureKHR::null() {
            self.accel.destroy_acceleration_structure(self.tlas, None);
        }
        if self.blas_buffer != vk::Buffer::null() {
            device.destroy_buffer(self.blas_buffer, None);
        }
        if self.tlas_buffer != vk::Buffer::null() {
            device.destroy_buffer(self.tlas_buffer, None);
        }
        if self.instance_buffer != vk::Buffer::null() {
            device.destroy_buffer(self.instance_buffer, None);
        }
        if self.blas_memory != vk::DeviceMemory::null() {
            device.free_memory(self.blas_memory, None);
        }
        if self.tlas_memory != vk::DeviceMemory::null() {
            device.free_memory(self.tlas_memory, None);
        }
        if self.instance_memory != vk::DeviceMemory::null() {
            device.free_memory(self.instance_memory, None);
        }
    }

    pub unsafe fn update_tlas(
        &mut self,
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        instances: &[vk::AccelerationStructureInstanceKHR],
    ) {
         
        unsafe { device.device_wait_idle().unwrap(); }

         
        if self.tlas != vk::AccelerationStructureKHR::null() {
            self.accel.destroy_acceleration_structure(self.tlas, None);
        }
        if self.tlas_buffer != vk::Buffer::null() {
            device.destroy_buffer(self.tlas_buffer, None);
        }
        if self.tlas_memory != vk::DeviceMemory::null() {
            device.free_memory(self.tlas_memory, None);
        }
        if self.instance_buffer != vk::Buffer::null() {
            device.destroy_buffer(self.instance_buffer, None);
        }
        if self.instance_memory != vk::DeviceMemory::null() {
            device.free_memory(self.instance_memory, None);
        }

         
        let instance_bytes = std::slice::from_raw_parts(
            instances.as_ptr() as *const u8,
            instances.len() * std::mem::size_of::<vk::AccelerationStructureInstanceKHR>(),
        );
        let (instance_buffer, instance_memory) =
            vk_buffers::create_device_local_buffer_with_data_flags(
                instance,
                physical_device,
                device,
                command_pool,
                queue,
                instance_bytes,
                vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
                    | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
                vk::MemoryAllocateFlags::DEVICE_ADDRESS,
            );
        self.instance_buffer = instance_buffer;
        self.instance_memory = instance_memory;

         
        let instances_addr = buffer_address(device, instance_buffer);
        let instance_data = vk::AccelerationStructureGeometryInstancesDataKHR::default()
            .array_of_pointers(false)
            .data(vk::DeviceOrHostAddressConstKHR {
                device_address: instances_addr,
            });

        let tlas_geometry = vk::AccelerationStructureGeometryKHR::default()
            .geometry_type(vk::GeometryTypeKHR::INSTANCES)
            .geometry(vk::AccelerationStructureGeometryDataKHR {
                instances: instance_data,
            });

        let tlas_range = vk::AccelerationStructureBuildRangeInfoKHR::default()
            .primitive_count(instances.len() as u32)
            .primitive_offset(0)
            .first_vertex(0)
            .transform_offset(0);

        let tlas_build_info = vk::AccelerationStructureBuildGeometryInfoKHR::default()
            .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL)
            .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
            .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
            .geometries(std::slice::from_ref(&tlas_geometry));

        let mut tlas_size = vk::AccelerationStructureBuildSizesInfoKHR::default();
        self.accel.get_acceleration_structure_build_sizes(
            vk::AccelerationStructureBuildTypeKHR::DEVICE,
            &tlas_build_info,
            std::slice::from_ref(&tlas_range.primitive_count),
            &mut tlas_size,
        );

        let (tlas_buffer, tlas_memory) = create_accel_buffer(
            instance,
            physical_device,
            device,
            tlas_size.acceleration_structure_size,
        );
        self.tlas_buffer = tlas_buffer;
        self.tlas_memory = tlas_memory;

        let tlas_create = vk::AccelerationStructureCreateInfoKHR::default()
            .buffer(tlas_buffer)
            .offset(0)
            .size(tlas_size.acceleration_structure_size)
            .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL);
        self.tlas = self.accel.create_acceleration_structure(&tlas_create, None).unwrap();

        let (scratch_buffer, scratch_memory) =
            create_scratch_buffer(instance, physical_device, device, tlas_size.build_scratch_size);
        let scratch_addr = buffer_address(device, scratch_buffer);

        let tlas_build_info = tlas_build_info
            .dst_acceleration_structure(self.tlas)
            .scratch_data(vk::DeviceOrHostAddressKHR {
                device_address: scratch_addr,
            });

        let tlas_ranges = [tlas_range];
        let tlas_range_refs = [&tlas_ranges[..]];
        let cmd = begin_one_time(device, command_pool);
        self.accel.cmd_build_acceleration_structures(
            cmd,
            std::slice::from_ref(&tlas_build_info),
            &tlas_range_refs,
        );
        end_one_time(device, queue, command_pool, cmd);

        device.destroy_buffer(scratch_buffer, None);
        device.free_memory(scratch_memory, None);
    }
}

pub unsafe fn build_blas(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
    vertex_count: u32,
    index_count: u32,
    vertex_stride: u64,
) -> Blas {
    build_blas_with_index_offset(
        instance,
        physical_device,
        device,
        command_pool,
        queue,
        vertex_buffer,
        index_buffer,
        vertex_count,
        index_count,
        vertex_stride,
        0,
    )
}

pub unsafe fn build_blas_with_index_offset(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
    max_vertex: u32,
    index_count: u32,
    vertex_stride: u64,
    index_offset_bytes: u64,
) -> Blas {
    let accel = khr::acceleration_structure::Device::new(instance, device);
    let vertex_addr = buffer_address(device, vertex_buffer);
    let index_addr = buffer_address(device, index_buffer) + index_offset_bytes;

    let triangles = vk::AccelerationStructureGeometryTrianglesDataKHR::default()
        .vertex_format(vk::Format::R32G32B32_SFLOAT)
        .vertex_data(vk::DeviceOrHostAddressConstKHR {
            device_address: vertex_addr,
        })
        .vertex_stride(vertex_stride)
        .max_vertex(max_vertex)
        .index_type(vk::IndexType::UINT32)
        .index_data(vk::DeviceOrHostAddressConstKHR {
            device_address: index_addr,
        })
        .transform_data(vk::DeviceOrHostAddressConstKHR { device_address: 0 });

    let blas_geometry = vk::AccelerationStructureGeometryKHR::default()
        .geometry_type(vk::GeometryTypeKHR::TRIANGLES)
        .geometry(vk::AccelerationStructureGeometryDataKHR { triangles })
        .flags(vk::GeometryFlagsKHR::OPAQUE);

    let max_prim_count = index_count / 3;
    let range = vk::AccelerationStructureBuildRangeInfoKHR::default()
        .primitive_count(max_prim_count)
        .primitive_offset(0)
        .first_vertex(0)
        .transform_offset(0);

    let build_info = vk::AccelerationStructureBuildGeometryInfoKHR::default()
        .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
        .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
        .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
        .geometries(std::slice::from_ref(&blas_geometry));

    let mut size_info = vk::AccelerationStructureBuildSizesInfoKHR::default();
    accel.get_acceleration_structure_build_sizes(
        vk::AccelerationStructureBuildTypeKHR::DEVICE,
        &build_info,
        std::slice::from_ref(&max_prim_count),
        &mut size_info,
    );

    let (blas_buffer, blas_memory) =
        create_accel_buffer(instance, physical_device, device, size_info.acceleration_structure_size);

    let as_create = vk::AccelerationStructureCreateInfoKHR::default()
        .buffer(blas_buffer)
        .offset(0)
        .size(size_info.acceleration_structure_size)
        .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL);
    let blas = accel.create_acceleration_structure(&as_create, None).unwrap();

    let (scratch_buffer, scratch_memory) =
        create_scratch_buffer(instance, physical_device, device, size_info.build_scratch_size);
    let scratch_addr = buffer_address(device, scratch_buffer);

    let build_info = build_info
        .dst_acceleration_structure(blas)
        .scratch_data(vk::DeviceOrHostAddressKHR {
            device_address: scratch_addr,
        });
    let ranges = [range];
    let range_refs = [&ranges[..]];

    let cmd = begin_one_time(device, command_pool);
    accel.cmd_build_acceleration_structures(
        cmd,
        std::slice::from_ref(&build_info),
        &range_refs,
    );
    end_one_time(device, queue, command_pool, cmd);

    device.destroy_buffer(scratch_buffer, None);
    device.free_memory(scratch_memory, None);

    Blas {
        accel: blas,
        buffer: blas_buffer,
        memory: blas_memory,
    }
}

pub unsafe fn build_model_blases(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
    model_infos: &[crate::vulkan::vk_meshlets::ModelRtInfo],
) -> Vec<Option<Blas>> {
    let index_stride = std::mem::size_of::<u32>() as u64;
    let vertex_stride = std::mem::size_of::<crate::vulkan::vk_meshlets::VertexData>() as u64;
    model_infos
        .iter()
        .map(|info| {
            if info.index_count == 0 || info.vertex_count == 0 {
                return None;
            }
            let index_offset_bytes = info.index_start as u64 * index_stride;
            let max_vertex = info.vertex_start + info.vertex_count;
            Some(build_blas_with_index_offset(
                instance,
                physical_device,
                device,
                command_pool,
                queue,
                vertex_buffer,
                index_buffer,
                max_vertex,
                info.index_count,
                vertex_stride,
                index_offset_bytes,
            ))
        })
        .collect()
}

unsafe fn begin_one_time(
    device: &ash::Device,
    pool: vk::CommandPool,
) -> vk::CommandBuffer {
    let alloc_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);
    let cmd = device.allocate_command_buffers(&alloc_info).unwrap()[0];
    let begin_info = vk::CommandBufferBeginInfo::default()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    device.begin_command_buffer(cmd, &begin_info).unwrap();
    cmd
}

unsafe fn end_one_time(
    device: &ash::Device,
    queue: vk::Queue,
    pool: vk::CommandPool,
    cmd: vk::CommandBuffer,
) {
    device.end_command_buffer(cmd).unwrap();
    let submit = vk::SubmitInfo::default().command_buffers(std::slice::from_ref(&cmd));
    device.queue_submit(queue, &[submit], vk::Fence::null()).unwrap();
    device.queue_wait_idle(queue).unwrap();
    device.free_command_buffers(pool, &[cmd]);
}

unsafe fn create_accel_buffer(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    size: vk::DeviceSize,
) -> (vk::Buffer, vk::DeviceMemory) {
    vk_buffers::create_buffer(
        instance,
        pdevice,
        device,
        size,
        vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
            | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::MemoryAllocateFlags::DEVICE_ADDRESS,
        None,
    )
}

unsafe fn create_scratch_buffer(
    instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    size: vk::DeviceSize,
) -> (vk::Buffer, vk::DeviceMemory) {
    vk_buffers::create_buffer(
        instance,
        pdevice,
        device,
        size,
        vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::MemoryAllocateFlags::DEVICE_ADDRESS,
        None,
    )
}

unsafe fn buffer_address(device: &ash::Device, buffer: vk::Buffer) -> u64 {
    let info = vk::BufferDeviceAddressInfo::default().buffer(buffer);
    device.get_buffer_device_address(&info)
}

pub unsafe fn build_scene_acceleration(
    vk_instance: &ash::Instance,
    pdevice: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
    vertex_count: u32,
    index_count: u32,
) -> RayTracingScene {
    let accel_loader = khr::acceleration_structure::Device::new(vk_instance, device);

     
    let static_blas = build_blas(
        vk_instance,
        pdevice,
        device,
        command_pool,
        queue,
        vertex_buffer,
        index_buffer,
        vertex_count,
        index_count,
        std::mem::size_of::<super::vk_meshlets::VertexData>() as u64,
    );

     
    let blas_addr = accel_loader.get_acceleration_structure_device_address(
        &vk::AccelerationStructureDeviceAddressInfoKHR::default().acceleration_structure(static_blas.accel),
    );

    let tlas_instance = vk::AccelerationStructureInstanceKHR {
        transform: vk::TransformMatrixKHR {
            matrix: [
                1.0, 0.0, 0.0, 0.0,  
                0.0, 1.0, 0.0, 0.0,  
                0.0, 0.0, 1.0, 0.0,
            ],
        },
        instance_custom_index_and_mask: vk::Packed24_8::new(0, 0xFF),
        instance_shader_binding_table_record_offset_and_flags: vk::Packed24_8::new(
            0,
            vk::GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE.as_raw() as u8,
        ),
        acceleration_structure_reference: vk::AccelerationStructureReferenceKHR {
            device_handle: blas_addr,
        },
    };

    let mut scene = RayTracingScene {
        accel: accel_loader,
        blas: static_blas.accel,
        tlas: vk::AccelerationStructureKHR::null(),  
        blas_buffer: static_blas.buffer,
        blas_memory: static_blas.memory,
        tlas_buffer: vk::Buffer::null(),
        tlas_memory: vk::DeviceMemory::null(),
        instance_buffer: vk::Buffer::null(),
        instance_memory: vk::DeviceMemory::null(),
    };
    
     
    scene.update_tlas(
        vk_instance,
        device,
        pdevice,
        command_pool,
        queue,
        &[tlas_instance],
    );

    scene
}
