 
 
 

use ash::vk;
use crate::vulkan::vk_textures;

 
pub(crate) unsafe fn load_textures(
    instance: &ash::Instance,
    device: &ash::Device,
    pdevice: vk::PhysicalDevice,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
) -> Vec<vk_textures::Texture> {
    let mut textures = Vec::new();
     
    textures.push(vk_textures::load_texture(
        instance,
        pdevice,
        device,
        command_pool,
        queue,
        "assets/concrete.png",
    ));
     
    textures.push(vk_textures::load_texture(
        instance,
        pdevice,
        device,
        command_pool,
        queue,
        "assets/grass.jpg",
    ));
     
    textures.push(vk_textures::load_texture(
        instance,
        pdevice,
        device,
        command_pool,
        queue,
        "assets/tree.png",
    ));
     
    textures.push(vk_textures::load_texture(
        instance,
        pdevice,
        device,
        command_pool,
        queue,
        "assets/rock.png",
    ));

     
    textures.push(vk_textures::load_texture(
        instance,
        pdevice,
        device,
        command_pool,
        queue,
        "assets/models/ak47/WPNT_AKM_AlbedoTransparency.png",
    ));
     
    textures.push(vk_textures::load_texture(
        instance,
        pdevice,
        device,
        command_pool,
        queue,
        "assets/models/ak47/WPNT_AKM_MetallicSmoothness.png",
    ));
     
    textures.push(vk_textures::load_texture(
        instance,
        pdevice,
        device,
        command_pool,
        queue,
        "assets/models/ak47/WPNT_AKM_Normal.png",
    ));
    textures
}

 
pub(crate) fn set_skybox() -> [f32; 4] {
    [0.4, 0.6, 1.0, 1.0]
}
