
use ash::{vk, Device};
use std::mem::size_of;

pub struct BloomRenderer {
    downsample_pipeline: vk::Pipeline,
    downsample_layout: vk::PipelineLayout,
    upsample_pipeline: vk::Pipeline,
    upsample_layout: vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,
    
     
    pub bloom_image: vk::Image,
    pub bloom_memory: vk::DeviceMemory,
    pub bloom_view: vk::ImageView,
    pub mips: u32,
    pub mip_views: Vec<vk::ImageView>,
    pub descriptor_sets: Vec<vk::DescriptorSet>,  
     
     
     
    
     
     
    pub down_sets: Vec<vk::DescriptorSet>,
    pub up_sets: Vec<vk::DescriptorSet>,
    sampler: vk::Sampler,
}

impl BloomRenderer {
    pub fn new(device: &Device, descriptor_pool: vk::DescriptorPool, width: u32, height: u32) -> Self {
         
        let down_layout = unsafe {
            let set_layout = create_descriptor_set_layout(device);
            device.create_pipeline_layout(&vk::PipelineLayoutCreateInfo::default().set_layouts(&[set_layout]), None).unwrap()
        };
        let down_pipeline = unsafe {
            crate::vulkan::vk_pipeline::create_compute_pipeline(
                device,
                down_layout,
                "shader/bloom_down.comp",
            )
        };

        let up_layout = unsafe {
            let set_layout = create_descriptor_set_layout(device);
            let push_constant = vk::PushConstantRange::default()
                .stage_flags(vk::ShaderStageFlags::COMPUTE)
                .offset(0)
                .size(4);  
            device.create_pipeline_layout(&vk::PipelineLayoutCreateInfo::default().set_layouts(&[set_layout]).push_constant_ranges(&[push_constant]), None).unwrap()
        };
        let up_pipeline = unsafe {
            crate::vulkan::vk_pipeline::create_compute_pipeline(
                device,
                up_layout,
                "shader/bloom_up.comp",
            )
        };
        
        let sampler = unsafe {
            device.create_sampler(&vk::SamplerCreateInfo::default()
                .mag_filter(vk::Filter::LINEAR)
                .min_filter(vk::Filter::LINEAR)
                .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE), None).unwrap()
        };
        
        let descriptor_set_layout = unsafe { create_descriptor_set_layout(device) };

        Self {
            downsample_pipeline: down_pipeline,
            downsample_layout: down_layout,
            upsample_pipeline: up_pipeline,
            upsample_layout: up_layout,
            descriptor_set_layout,
            bloom_image: vk::Image::null(),
            bloom_memory: vk::DeviceMemory::null(),
            bloom_view: vk::ImageView::null(),
            mips: 0,
            mip_views: Vec::new(),
            descriptor_sets: Vec::new(),
            down_sets: Vec::new(),
            up_sets: Vec::new(),
            sampler,
        }
    }
    
    pub unsafe fn resize(&mut self, instance: &ash::Instance, physical_device: vk::PhysicalDevice, device: &Device, descriptor_pool: vk::DescriptorPool, width: u32, height: u32) {
        if self.bloom_image != vk::Image::null() {
            device.destroy_image(self.bloom_image, None);
            device.free_memory(self.bloom_memory, None);
            device.destroy_image_view(self.bloom_view, None);
            for view in &self.mip_views {
                device.destroy_image_view(*view, None);
            }
        }
        
         
        let w = width / 2;
        let h = height / 2;
        self.mips = (std::cmp::max(w, h) as f32).log2().floor() as u32;
        if self.mips < 2 { self.mips = 2; }
        if self.mips > 6 { self.mips = 6; }  
        
        let (image, memory) = crate::vulkan::vk_memory::create_image_mips(
            instance,
            physical_device,
            device,
            w,
            h,
            self.mips,
            vk::Format::R16G16B16A16_SFLOAT,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );
        self.bloom_image = image;
        self.bloom_memory = memory;
        
        self.bloom_view = device.create_image_view(&vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R16G16B16A16_SFLOAT)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: self.mips,
                base_array_layer: 0,
                layer_count: 1,
            }), None).unwrap();
            
        self.mip_views.clear();
        for i in 0..self.mips {
             self.mip_views.push(device.create_image_view(&vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::R16G16B16A16_SFLOAT)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: i,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                }), None).unwrap());
        }
        
         
         
         
        let sets_needed = (self.mips - 1) * 2;
        let layouts = vec![self.descriptor_set_layout; sets_needed as usize];
        let sets = device.allocate_descriptor_sets(&vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts)).unwrap();
            
        self.down_sets = sets[0..(self.mips as usize - 1)].to_vec();
        self.up_sets = sets[(self.mips as usize - 1)..].to_vec();

        for i in 0..(self.mips - 1) {
             
            update_descriptor_set(device, self.down_sets[i as usize], self.mip_views[i as usize], self.mip_views[(i+1) as usize], self.sampler);
            
             
             
             
             
             
            update_descriptor_set(device, self.up_sets[i as usize], self.mip_views[(i+1) as usize], self.mip_views[i as usize], self.sampler);
        }
    }
    
    pub unsafe fn dispatch(&self, device: &Device, cmd: vk::CommandBuffer, _source_image: vk::Image) {
         
         
         
         
         
        
         
        device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, self.downsample_pipeline);
        for i in 0..(self.mips - 1) {
             let idx = i as usize;
              
              
              
             
             let set = self.down_sets[idx];
             device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::COMPUTE, self.downsample_layout, 0, &[set], &[]);
             
              
              
              
              
              
              
              
              
              
             device.cmd_dispatch(cmd, 256 >> (i+1), 256 >> (i+1), 1);
             
              
             let barrier = vk::ImageMemoryBarrier::default()
                 .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                 .dst_access_mask(vk::AccessFlags::SHADER_READ)
                 .old_layout(vk::ImageLayout::GENERAL)
                 .new_layout(vk::ImageLayout::GENERAL)
                 .image(self.bloom_image)
                 .subresource_range(vk::ImageSubresourceRange {
                     aspect_mask: vk::ImageAspectFlags::COLOR,
                     base_mip_level: i + 1,
                     level_count: 1,
                     base_array_layer: 0,
                     layer_count: 1,
                 });
             device.cmd_pipeline_barrier(cmd, vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::COMPUTE_SHADER, vk::DependencyFlags::empty(), &[], &[], &[barrier]);
        }
        
         
        device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, self.upsample_pipeline);
         
        for i in (0..(self.mips - 1)).rev() {
             let idx = i as usize;
             let set = self.up_sets[idx];
             device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::COMPUTE, self.upsample_layout, 0, &[set], &[]);
             
             let radius: f32 = 0.005; 
             device.cmd_push_constants(cmd, self.upsample_layout, vk::ShaderStageFlags::COMPUTE, 0, bytemuck::bytes_of(&radius));
             
             device.cmd_dispatch(cmd, 256 >> i, 256 >> i, 1);
             
               
               
             let barrier = vk::ImageMemoryBarrier::default()
                 .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                 .dst_access_mask(vk::AccessFlags::SHADER_READ)
                 .image(self.bloom_image)
                 .subresource_range(vk::ImageSubresourceRange {
                     aspect_mask: vk::ImageAspectFlags::COLOR,
                     base_mip_level: i,
                     level_count: 1,
                     base_array_layer: 0,
                     layer_count: 1,
                 });
             device.cmd_pipeline_barrier(cmd, vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::COMPUTE_SHADER, vk::DependencyFlags::empty(), &[], &[], &[barrier]);
        }
    }
}

unsafe fn create_descriptor_set_layout(device: &Device) -> vk::DescriptorSetLayout {
    let bindings = [
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE),
        vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE),
    ];
    device.create_descriptor_set_layout(&vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings), None).unwrap()
}

unsafe fn update_descriptor_set(device: &Device, set: vk::DescriptorSet, input_view: vk::ImageView, output_view: vk::ImageView, sampler: vk::Sampler) {
    let image_info_input = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::GENERAL)
        .image_view(input_view)
        .sampler(sampler);
    let image_info_output = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::GENERAL)
        .image_view(output_view);
        
    let writes = [
        vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&image_info_input)),
        vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(1)
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .image_info(std::slice::from_ref(&image_info_output)),
    ];
    device.update_descriptor_sets(&writes, &[]);
}
