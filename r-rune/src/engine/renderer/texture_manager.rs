use std::collections::HashMap;
use ash::vk;
use crate::vulkan::vk_textures::{self, Texture};

/// Manages texture loading with caching and on-demand loading.
/// Textures are loaded once and reused across models that reference them.
pub struct TextureManager {
    textures: Vec<Texture>,
    path_to_index: HashMap<String, u32>,
    default_index: u32,
    // Vulkan resources
    instance: ash::Instance,
    device: ash::Device,
    physical_device: vk::PhysicalDevice,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
}

impl TextureManager {
    /// Create a new TextureManager with a default white texture.
    pub unsafe fn new(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
    ) -> Self {
        // Create a default 1x1 white texture
        let default_texture = create_default_texture(instance, physical_device, device, command_pool, queue);
        
        let mut textures = Vec::new();
        textures.push(default_texture);
        
        Self {
            textures,
            path_to_index: HashMap::new(),
            default_index: 0,
            instance: instance.clone(),
            device: device.clone(),
            physical_device,
            command_pool,
            queue,
        }
    }

    /// Get or load a texture by path. Returns the texture index.
    /// If the texture is already loaded, returns cached index.
    /// If loading fails or path is None, returns default texture index.
    pub unsafe fn get_or_load(&mut self, path: Option<&str>) -> u32 {
        let path = match path {
            Some(p) => p,
            None => return self.default_index,
        };

        // Check cache
        if let Some(&index) = self.path_to_index.get(path) {
            return index;
        }

        // Try to load
        match self.load_texture(path) {
            Ok(texture) => {
                let index = self.textures.len() as u32;
                self.textures.push(texture);
                self.path_to_index.insert(path.to_string(), index);
                println!("TextureManager: Loaded '{}' as index {}", path, index);
                index
            }
            Err(e) => {
                eprintln!("TextureManager: Failed to load '{}': {}", path, e);
                self.default_index
            }
        }
    }

    /// Load a texture from disk.
    unsafe fn load_texture(&self, path: &str) -> Result<Texture, String> {
        if !std::path::Path::new(path).exists() {
            return Err(format!("File not found: {}", path));
        }

        Ok(vk_textures::load_texture(
            &self.instance,
            self.physical_device,
            &self.device,
            self.command_pool,
            self.queue,
            path,
        ))
    }

    /// Get all loaded textures for descriptor binding.
    pub fn textures(&self) -> &[Texture] {
        &self.textures
    }

    /// Get the number of loaded textures.
    pub fn count(&self) -> u32 {
        self.textures.len() as u32
    }

    /// Preload a list of texture paths (useful for batch loading).
    pub unsafe fn preload(&mut self, paths: &[&str]) {
        for path in paths {
            self.get_or_load(Some(path));
        }
    }

    /// Get texture index for a path, or None if not loaded.
    pub fn get_index(&self, path: &str) -> Option<u32> {
        self.path_to_index.get(path).copied()
    }

    /// Clear all textures except the default.
    pub unsafe fn clear(&mut self) {
        // Keep only the default texture
        self.textures.truncate(1);
        self.path_to_index.clear();
    }
}

/// Create a 1x1 white default texture.
unsafe fn create_default_texture(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
) -> Texture {
    use crate::vulkan::{vk_buffers, vk_memory};

    let white_pixel: [u8; 4] = [255, 255, 255, 255];
    let width = 1u32;
    let height = 1u32;

    // Create staging buffer
    let buffer_size = 4u64;
    let (staging_buffer, staging_memory) = vk_buffers::create_buffer(
        instance,
        physical_device,
        device,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        vk::MemoryAllocateFlags::empty(),
        Some(&white_pixel),
    );

    // Create image
    let image_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D { width, height, depth: 1 })
        .mip_levels(1)
        .array_layers(1)
        .format(vk::Format::R8G8B8A8_SRGB)
        .tiling(vk::ImageTiling::OPTIMAL)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(vk::SampleCountFlags::TYPE_1);

    let image = device.create_image(&image_info, None).unwrap();
    let mem_reqs = device.get_image_memory_requirements(image);
    let mem_type = vk_memory::find_memory_type(
        instance,
        physical_device,
        mem_reqs.memory_type_bits,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );

    let alloc_info = vk::MemoryAllocateInfo::default()
        .allocation_size(mem_reqs.size)
        .memory_type_index(mem_type);
    let memory = device.allocate_memory(&alloc_info, None).unwrap();
    device.bind_image_memory(image, memory, 0).unwrap();

    // Transition and copy
    vk_textures::transition_image_layout(
        device,
        command_pool,
        queue,
        image,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    );

    vk_textures::copy_buffer_to_image(device, command_pool, queue, staging_buffer, image, width, height);

    vk_textures::transition_image_layout(
        device,
        command_pool,
        queue,
        image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );

    // Cleanup staging
    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_memory, None);

    // Create view
    let view_info = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(vk::Format::R8G8B8A8_SRGB)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    let view = device.create_image_view(&view_info, None).unwrap();

    // Create sampler
    let sampler_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::REPEAT)
        .address_mode_v(vk::SamplerAddressMode::REPEAT)
        .address_mode_w(vk::SamplerAddressMode::REPEAT)
        .anisotropy_enable(false)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
        .unnormalized_coordinates(false)
        .compare_enable(false)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .mip_lod_bias(0.0)
        .min_lod(0.0)
        .max_lod(0.0);
    let sampler = device.create_sampler(&sampler_info, None).unwrap();

    Texture {
        image,
        memory,
        view,
        sampler,
    }
}
