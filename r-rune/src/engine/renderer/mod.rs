use std::sync::Arc;

use ash::{ext, khr, vk};
use winit::{
    event_loop::EventLoop,
    window::Window,
};

mod bloom;
mod composite;
mod hiz;
mod init;
pub mod lighting;
mod skinned;
mod textures;
pub mod texture_manager;

use crate::engine::renderer::bloom::BloomRenderer;
use crate::engine::renderer::composite::CompositeRenderer;
pub use skinned::SkinnedActorDesc;
use lighting::LightingSystem;
use skinned::{SkinnedActor, SkinnedPushConstants, create_skinned_actor, destroy_skinned_actor};

use crate::vulkan::{
    vk_buffers, vk_device, vk_instance, vk_memory,
    vk_meshlets::{self, Aabb, MeshBuffers, MeshletModelDesc, PushConstants},
    vk_pipeline, vk_raytracing, vk_render_pass, vk_swapchain, vk_sync, vk_textures,
};
use crate::engine::gui::crosshair::{CrosshairRenderer, CrosshairStyle};

use super::{
    environment::sun_sphere::SunSphereRenderer,
    environment::clouds::CloudRenderer,
    environment::fire::FireRenderer,
    sdf::SdfRenderer,
    environment::rain::RainRenderer,
    environment::snow::SnowRenderer,
    environment::stars::StarsRenderer,
    gui::{menu::MenuRenderer, text::TextRenderer, hud::HudRenderer},
    HEIGHT, MAX_FRAMES_IN_FLIGHT, WIDTH,
};

pub struct FrameInput {
    pub view: glam::Mat4,
    pub proj: glam::Mat4,
    pub camera_pos: glam::Vec3,
    pub sun_pos: glam::Vec3,
    pub sun_dir: glam::Vec3,
    pub sun_intensity: f32,
    pub sun_color: glam::Vec3,
    pub sphere_offset: glam::Vec3,
    pub sphere_model_idx: Option<u32>,  
    pub hovered_model_idx: Option<u32>,
    pub effect_time: f32,
    pub wind_time: f32,
    pub cloud_config: crate::engine::state::config::CloudConfig,
    pub rain_config: crate::engine::state::config::RainConfig,
    pub snow_config: crate::engine::state::config::SnowConfig,
    pub enable_snow: bool,
    pub enable_rain: bool,
    pub rain_time: f32,
    pub wind_dir: glam::Vec2,
    pub wind_speed: f32,
    pub wind_turbulence: f32,
    pub dt: f32,
}

 

pub struct Renderer {
    pub(crate) _entry: ash::Entry,
    pub(crate) instance: ash::Instance,
    pub(crate) surface_loader: khr::surface::Instance,
    pub(crate) surface: vk::SurfaceKHR,
    pub(crate) physical_device: vk::PhysicalDevice,
    pub(crate) queue_family_index: u32,
    pub(crate) device: ash::Device,
    pub(crate) graphics_queue: vk::Queue,
    pub(crate) present_queue: vk::Queue,
    pub(crate) swapchain_loader: khr::swapchain::Device,
    pub(crate) swapchain: vk::SwapchainKHR,
    pub(crate) swapchain_format: vk::Format,
    pub(crate) swapchain_extent: vk::Extent2D,
    pub(crate) swapchain_images: Vec<vk::Image>,
    pub(crate) swapchain_image_views: Vec<vk::ImageView>,
    pub(crate) render_pass: vk::RenderPass,
    pub(crate) pipeline_layout: vk::PipelineLayout,
    pub(crate) pipeline: vk::Pipeline,
    pub(crate) outline_pipeline: vk::Pipeline,
    pub(crate) swapchain_framebuffers: Vec<vk::Framebuffer>,
    pub(crate) command_pool: vk::CommandPool,
    pub(crate) command_buffers: Vec<vk::CommandBuffer>,
    pub(crate) image_available_semaphores: Vec<vk::Semaphore>,
    pub(crate) render_finished_semaphores: Vec<vk::Semaphore>,
    pub(crate) in_flight_fences: Vec<vk::Fence>,
    pub(crate) descriptor_set_layout: vk::DescriptorSetLayout,
    pub(crate) descriptor_pool: vk::DescriptorPool,
    pub(crate) ui_descriptor_pool: vk::DescriptorPool,
    pub(crate) descriptor_sets: Vec<vk::DescriptorSet>,

    pub(crate) textures: Vec<vk_textures::Texture>,
    pub(crate) sky_color: [f32; 4],
    pub(crate) mesh: MeshBuffers,
    pub(crate) raytracing: vk_raytracing::RayTracingScene,
    pub(crate) rt_model_blases: Vec<Option<vk_raytracing::Blas>>,
    pub(crate) rt_model_base_positions: Vec<glam::Vec3>,
    pub(crate) rt_model_offsets: Vec<glam::Vec3>,
    pub(crate) rt_model_rotations: Vec<glam::Quat>,
    pub(crate) skinned_pipeline_layout: vk::PipelineLayout,
    pub(crate) skinned_descriptor_set_layout: vk::DescriptorSetLayout,
    pub(crate) skinned_pipeline: vk::Pipeline,
    pub(crate) skinned_actors: Vec<SkinnedActor>,
    pub(crate) skinned_blases: std::collections::HashMap<String, vk_raytracing::Blas>,
    pub(crate) depth: vk_memory::DepthResources,
    pub(crate) hiz_image: vk::Image,
    pub(crate) hiz_memory: vk::DeviceMemory,
    pub(crate) hiz_view: vk::ImageView,
    pub(crate) hiz_pipeline_layout: vk::PipelineLayout,
    pub(crate) hiz_descriptor_set_layout: vk::DescriptorSetLayout,
    pub(crate) hiz_pipeline: vk::Pipeline,
    pub(crate) hiz_descriptor_sets: Vec<vk::DescriptorSet>,
    pub(crate) hiz_mips: u32,
    pub(crate) hiz_mip_views: Vec<vk::ImageView>,
    pub(crate) hiz_sampler: vk::Sampler,
    pub(crate) current_frame: usize,
    pub(crate) window: Arc<Window>,
    pub(crate) crosshair_renderer: crate::engine::gui::crosshair::CrosshairRenderer,
    pub(crate) crosshair_enabled: bool,
    pub(crate) crosshair_style: crate::engine::gui::crosshair::CrosshairStyle,
    pub(crate) crosshair_scale: f32,
    pub(crate) msaa_samples: vk::SampleCountFlags,
    pub(crate) color_msaa: Option<vk::Image>,
    pub(crate) color_msaa_memory: Option<vk::DeviceMemory>,
    pub(crate) color_msaa_view: Option<vk::ImageView>,

     
    pub(crate) hdr_image: vk::Image,
    pub(crate) hdr_memory: vk::DeviceMemory,
    pub(crate) hdr_view: vk::ImageView,
    pub(crate) hdr_format: vk::Format,

     
     
    pub(crate) composite_render_pass: vk::RenderPass,
    pub(crate) composite_framebuffers: Vec<vk::Framebuffer>,
    pub(crate) composite_renderer: CompositeRenderer,
    pub(crate) composite_descriptor_sets: Vec<vk::DescriptorSet>,
    
     
    pub(crate) bloom_renderer: BloomRenderer,
    pub(crate) bloom_enabled: bool,
    
     
    pub(crate) rain_renderer: crate::engine::environment::rain::RainRenderer,
    pub(crate) snow_renderer: crate::engine::environment::snow::SnowRenderer,

     
    pub(crate) lighting_system: LightingSystem,
}

impl Renderer {
     

    unsafe fn create_skinned_actor_internal(&self, desc: &SkinnedActorDesc) -> SkinnedActor {
        create_skinned_actor(
            &self.instance,
            &self.device,
            self.physical_device,
            self.command_pool,
            self.graphics_queue,
            self.descriptor_pool,
            self.skinned_descriptor_set_layout,
            desc,
        )
    }

    unsafe fn destroy_skinned_actor_internal(&self, actor: SkinnedActor) {
        destroy_skinned_actor(&self.device, self.descriptor_pool, actor);
    }

    pub unsafe fn set_skinned_actors(&mut self, actors: &[SkinnedActorDesc]) {
        let old = std::mem::take(&mut self.skinned_actors);
        for actor in old {
            self.destroy_skinned_actor_internal(actor);
        }
        for desc in actors {
            let actor = self.create_skinned_actor_internal(desc);
            self.skinned_actors.push(actor);
        }
    }

    pub unsafe fn update_skinned_actor_position(&mut self, id: &str, pos: glam::Vec3) {
        if let Some(actor) = self.skinned_actors.iter_mut().find(|a| a.id == id) {
            actor.position = pos;
        }
    }

     
     
    pub unsafe fn update_rt_tlas(&mut self) {
         
        let existing_ids: std::collections::HashSet<_> = self.skinned_blases.keys().cloned().collect();
        let current_ids: std::collections::HashSet<_> = self.skinned_actors.iter().map(|a| a.id.clone()).collect();

         
        for id in existing_ids.difference(&current_ids) {
            if let Some(blas) = self.skinned_blases.remove(id) {
                self.raytracing.accel.destroy_acceleration_structure(blas.accel, None);
                self.device.destroy_buffer(blas.buffer, None);
                self.device.free_memory(blas.memory, None);
            }
        }

         
        for actor in &self.skinned_actors {
            if !self.skinned_blases.contains_key(&actor.id) {
                let blas = vk_raytracing::build_blas(
                    &self.instance,
                    self.physical_device,
                    &self.device,
                    self.command_pool,
                    self.graphics_queue,
                    actor.vertex_buffer,
                    actor.index_buffer,
                    actor.vertex_count,
                    actor.index_count,
                    std::mem::size_of::<skinned::SkinnedVertex>() as u64,
                );
                self.skinned_blases.insert(actor.id.clone(), blas);
            }
        }

         
        let mut instances = Vec::new();

         
        for (idx, blas) in self.rt_model_blases.iter().enumerate() {
            let Some(blas) = blas else {
                continue;
            };
            let base = self
                .rt_model_base_positions
                .get(idx)
                .copied()
                .unwrap_or(glam::Vec3::ZERO);
            let offset = self
                .rt_model_offsets
                .get(idx)
                .copied()
                .unwrap_or(glam::Vec3::ZERO);
            let rotation = self
                .rt_model_rotations
                .get(idx)
                .copied()
                .unwrap_or(glam::Quat::IDENTITY);
            let translation = base + offset - (rotation * base);
            let t = glam::Mat4::from_rotation_translation(rotation, translation);
            let matrix = [
                t.x_axis.x, t.y_axis.x, t.z_axis.x, t.w_axis.x,
                t.x_axis.y, t.y_axis.y, t.z_axis.y, t.w_axis.y,
                t.x_axis.z, t.y_axis.z, t.z_axis.z, t.w_axis.z,
            ];
            let blas_addr = self.raytracing.accel.get_acceleration_structure_device_address(
                &vk::AccelerationStructureDeviceAddressInfoKHR::default().acceleration_structure(blas.accel),
            );
            instances.push(vk::AccelerationStructureInstanceKHR {
                transform: vk::TransformMatrixKHR { matrix },
                instance_custom_index_and_mask: vk::Packed24_8::new(0, 0xFF),
                instance_shader_binding_table_record_offset_and_flags: vk::Packed24_8::new(
                    0,
                    vk::GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE.as_raw() as u8,
                ),
                acceleration_structure_reference: vk::AccelerationStructureReferenceKHR {
                    device_handle: blas_addr,
                },
            });
        }

         
        for actor in &self.skinned_actors {
            if let Some(blas) = self.skinned_blases.get(&actor.id) {
                let blas_addr = self.raytracing.accel.get_acceleration_structure_device_address(
                    &vk::AccelerationStructureDeviceAddressInfoKHR::default().acceleration_structure(blas.accel),
                );

                 
                let t = glam::Mat4::from_translation(actor.position)
                    * glam::Mat4::from_rotation_y(actor.rotation_y)
                    * glam::Mat4::from_scale(glam::Vec3::splat(actor.scale));
                let matrix = [
                    t.x_axis.x, t.y_axis.x, t.z_axis.x, t.w_axis.x,
                    t.x_axis.y, t.y_axis.y, t.z_axis.y, t.w_axis.y,
                    t.x_axis.z, t.y_axis.z, t.z_axis.z, t.w_axis.z,
                ];

                instances.push(vk::AccelerationStructureInstanceKHR {
                    transform: vk::TransformMatrixKHR { matrix },
                    instance_custom_index_and_mask: vk::Packed24_8::new(0, 0xFF),
                    instance_shader_binding_table_record_offset_and_flags: vk::Packed24_8::new(
                        0,
                        vk::GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE.as_raw() as u8,
                    ),
                    acceleration_structure_reference: vk::AccelerationStructureReferenceKHR {
                        device_handle: blas_addr,
                    },
                });
            }
        }

        if instances.is_empty() && self.raytracing.blas != vk::AccelerationStructureKHR::null() {
            let static_blas_addr = self.raytracing.accel.get_acceleration_structure_device_address(
                &vk::AccelerationStructureDeviceAddressInfoKHR::default().acceleration_structure(self.raytracing.blas),
            );
            instances.push(vk::AccelerationStructureInstanceKHR {
                transform: vk::TransformMatrixKHR {
                    matrix: [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
                },
                instance_custom_index_and_mask: vk::Packed24_8::new(0, 0xFF),
                instance_shader_binding_table_record_offset_and_flags: vk::Packed24_8::new(
                    0,
                    vk::GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE.as_raw() as u8,
                ),
                acceleration_structure_reference: vk::AccelerationStructureReferenceKHR {
                    device_handle: static_blas_addr,
                },
            });
        }

         
        self.raytracing.update_tlas(
            &self.instance,
            &self.device,
            self.physical_device,
            self.command_pool,
            self.graphics_queue,
            &instances,
        );

         
        let mut accel_info = vk::WriteDescriptorSetAccelerationStructureKHR::default()
            .acceleration_structures(std::slice::from_ref(&self.raytracing.tlas));

        for i in 0..self.descriptor_sets.len() {
            let write = vk::WriteDescriptorSet::default()
                .dst_set(self.descriptor_sets[i])
                .dst_binding(7)  
                .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                .descriptor_count(1)
                .push_next(&mut accel_info);
            self.device.update_descriptor_sets(&[write], &[]);
        }

        if !self.skinned_actors.is_empty() {
            for actor in &self.skinned_actors {
                let mut accel_info = vk::WriteDescriptorSetAccelerationStructureKHR::default()
                    .acceleration_structures(std::slice::from_ref(&self.raytracing.tlas));
                let write = vk::WriteDescriptorSet::default()
                    .dst_set(actor.descriptor_set)
                    .dst_binding(6)  
                    .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                    .descriptor_count(1)
                    .push_next(&mut accel_info);
                self.device.update_descriptor_sets(&[write], &[]);
            }
        }
    }

    pub fn set_bloom_enabled(&mut self, enabled: bool) {
        self.bloom_enabled = enabled;
    }

     
    unsafe fn load_textures(
        instance: &ash::Instance,
        device: &ash::Device,
        pdevice: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
    ) -> Vec<vk_textures::Texture> {
        textures::load_textures(instance, device, pdevice, command_pool, queue)
    }

    fn set_skybox() -> [f32; 4] {
        textures::set_skybox()
    }

     
    pub unsafe fn new(
        window: Arc<Window>,
        maximize: bool,
        msaa_samples: vk::SampleCountFlags,
    ) -> Self {
        let models: &[MeshletModelDesc] = &[];
        let skinned_asset: Option<String> = None;
        let (entry, instance, surface_loader, surface) = Self::init_instance(&window);

        let (physical_device, queue_family_index) =
            vk_instance::pick_physical_device(&instance, &surface_loader, surface);
        
         
        let props = instance.get_physical_device_properties(physical_device);
        let counts = props.limits.framebuffer_color_sample_counts & props.limits.framebuffer_depth_sample_counts;
        let mut samples = msaa_samples;
        if !counts.contains(samples) {
            println!("Requested MSAA sample count {:?} not supported, falling back to 1", samples);
            samples = vk::SampleCountFlags::TYPE_1;
        }

        let (device, graphics_queue, present_queue) =
            vk_device::create_device(&instance, physical_device, queue_family_index);

            let (
                swapchain_loader,
                swapchain,
                swapchain_format,
                swapchain_extent,
                swapchain_images,
                swapchain_image_views,
            ) = Self::init_swapchain(
                &instance,
                &device,
                physical_device,
                &surface_loader,
                surface,
                queue_family_index,
                &window,
            );

             
            let hdr_format = vk::Format::R16G16B16A16_SFLOAT;
            let (hdr_image, hdr_memory) = vk_memory::create_image(
                &instance,
                physical_device,
                &device,
                swapchain_extent.width,
                swapchain_extent.height,
                hdr_format,
                vk::ImageTiling::OPTIMAL,
                vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            );
            let hdr_view = device
                .create_image_view(
                    &vk::ImageViewCreateInfo::default()
                        .image(hdr_image)
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(hdr_format)
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        }),
                    None,
                )
                .unwrap();

            let (color_msaa, color_msaa_memory, color_msaa_view) = if samples != vk::SampleCountFlags::TYPE_1 {
                let (image, memory) = vk_memory::create_image_with_samples(
                    &instance,
                    physical_device,
                    &device,
                    swapchain_extent.width,
                    swapchain_extent.height,
                    hdr_format,
                    vk::ImageTiling::OPTIMAL,
                    vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT,
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                    samples,
                );
                let view_info = vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(hdr_format)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                             .aspect_mask(vk::ImageAspectFlags::COLOR)
                             .base_mip_level(0)
                             .level_count(1)
                             .base_array_layer(0)
                             .layer_count(1)
                    );
                let view = device.create_image_view(&view_info, None).unwrap();
                (Some(image), Some(memory), Some(view))
            } else {
                (None, None, None)
            };

            let depth = vk_memory::create_depth_resources_with_samples(
                &instance,
                physical_device,
                &device,
                swapchain_extent,
                samples,
            );
            
             
            let render_pass = vk_render_pass::create_render_pass(
                &device,
                hdr_format,
                depth.format,
                samples,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,  
            );

             
            let composite_render_pass = vk_render_pass::create_render_pass(
                 &device,
                 swapchain_format,
                 depth.format,  
                 vk::SampleCountFlags::TYPE_1,
                 vk::ImageLayout::PRESENT_SRC_KHR,
            );
            
            let composite_framebuffers = swapchain_image_views
                .iter()
                .map(|&view| {
                      
                     let attachments = [view, depth.view];
                     let create_info = vk::FramebufferCreateInfo::default()
                         .render_pass(composite_render_pass)
                         .attachments(&attachments)
                         .width(swapchain_extent.width)
                         .height(swapchain_extent.height)
                         .layers(1);
                     unsafe { device.create_framebuffer(&create_info, None).unwrap() }
                })
                .collect::<Vec<_>>();


            let (pipeline, pipeline_layout, descriptor_set_layout) =
                Self::init_pipeline(&device, render_pass, samples);
            let outline_pipeline = vk_pipeline::create_outline_pipeline(&device, pipeline_layout, render_pass, samples);
            let (skinned_pipeline, skinned_pipeline_layout, skinned_descriptor_set_layout) =
                Self::init_skinned_pipeline(&device, render_pass, samples);
 
            let command_pool = vk_sync::create_command_pool(&device, queue_family_index);
            let command_buffers = vk_sync::allocate_command_buffers(
                &device,
                command_pool,
                MAX_FRAMES_IN_FLIGHT as u32,
            );
            let descriptor_pool =
                vk_sync::create_descriptor_pool(&device, MAX_FRAMES_IN_FLIGHT as u32);
            let ui_descriptor_pool = vk_sync::create_ui_descriptor_pool(&device, MAX_FRAMES_IN_FLIGHT as u32);
            
            let crosshair_scale = 1.0;
            let crosshair_renderer = CrosshairRenderer::new(
                &instance,
                &device,
                physical_device,
                command_pool,
                graphics_queue,
                render_pass,
                crosshair_scale,
                samples,
            );

             
            let (hiz_pipeline, hiz_pipeline_layout, hiz_descriptor_set_layout) = hiz::create_hiz_pipeline(&device);
            let hiz_sampler = hiz::create_hiz_sampler(&device);
            
            let (hiz_image, hiz_memory, hiz_view, hiz_mip_views, hiz_mips, hiz_descriptor_sets) = 
                hiz::create_hiz_image_and_views(
                    &instance,
                    &device,
                    physical_device,
                    descriptor_pool,
                    hiz_descriptor_set_layout,
                    hiz_sampler,
                    depth.view,
                    swapchain_extent.width,
                    swapchain_extent.height,
                );
             
            let textures = Self::load_textures(
                &instance,
                &device,
                physical_device,
                command_pool,
                graphics_queue,
            );
            
            let sky_color = Self::set_skybox();

            let (mesh, colliders) = vk_meshlets::create_resources(
                &instance,
                physical_device,
                &device,
                command_pool,
                graphics_queue,
                models,
            );
            
             
            let raytracing = vk_raytracing::build_scene_acceleration(
                &instance,
                physical_device,
                &device,
                command_pool,
                graphics_queue,
                mesh.vertex_buffer,
                mesh.index_buffer,
                mesh.vertex_count,
                mesh.index_count,
            );
            let rt_model_blases = vk_raytracing::build_model_blases(
                &instance,
                physical_device,
                &device,
                command_pool,
                graphics_queue,
                mesh.vertex_buffer,
                mesh.index_buffer,
                &mesh.model_rt_infos,
            );
            let rt_model_base_positions = mesh
                .model_rt_infos
                .iter()
                .map(|info| info.base_position)
                .collect::<Vec<_>>();
            let rt_model_offsets = vec![glam::Vec3::ZERO; rt_model_base_positions.len()];
            let rt_model_rotations = vec![glam::Quat::IDENTITY; rt_model_base_positions.len()];

             
            let lighting_system = LightingSystem::new(
                &instance,
                &device,
                physical_device,
                descriptor_pool,
                swapchain_extent.width,
                swapchain_extent.height,
            );

             
            let descriptor_sets = vk_sync::allocate_descriptor_sets(
                &device,
                descriptor_pool,
                descriptor_set_layout,
                mesh.meshlet_buffer,
                mesh.vertex_buffer,
                mesh.meshlet_vertices_buffer,
                mesh.meshlet_triangles_buffer,
                &textures,
                hiz_view,
                hiz_sampler,
                raytracing.tlas,
                mesh.material_buffer,
                mesh.meshlet_params_buffer,
                lighting_system.light_buffer,
                lighting_system.cluster_info_buffer,
                lighting_system.light_index_buffer,
                MAX_FRAMES_IN_FLIGHT,
            );

             
             

            let initial_skinned: Vec<SkinnedActorDesc> = skinned_asset
                .map(|p| SkinnedActorDesc {
                    id: "skinned0".to_string(),
                    path: p.to_string(),
                    position: glam::Vec3::ZERO,
                    scale: None,
                    rotation_y_deg: None,
                })
                .into_iter()
                .collect();



             
            let bloom_renderer = BloomRenderer::new(&device, descriptor_pool, swapchain_extent.width, swapchain_extent.height);
            let composite_renderer = CompositeRenderer::new(&device, composite_render_pass, vk::SampleCountFlags::TYPE_1);
            
            let composite_descriptor_sets = {
                 let layouts = vec![composite_renderer.descriptor_set_layout(); MAX_FRAMES_IN_FLIGHT];
                 let alloc_info = vk::DescriptorSetAllocateInfo::default()
                     .descriptor_pool(descriptor_pool)
                     .set_layouts(&layouts);
                 unsafe { device.allocate_descriptor_sets(&alloc_info).unwrap() }
            };

            let swapchain_framebuffers = {
                  
                  
                 let mut attachments = Vec::new();
                 if let Some(msaa_view) = color_msaa_view {
                     attachments.push(msaa_view);
                     attachments.push(depth.view);
                     attachments.push(hdr_view);
                 } else {
                     attachments.push(hdr_view);
                     attachments.push(depth.view);
                 }
                 let create_info = vk::FramebufferCreateInfo::default()
                     .render_pass(render_pass)
                     .attachments(&attachments)
                     .width(swapchain_extent.width)
                     .height(swapchain_extent.height)
                     .layers(1);
                  
                  
                 vec![device.create_framebuffer(&create_info, None).unwrap()]
            };

            let (image_available_semaphores, render_finished_semaphores, in_flight_fences) =
                vk_sync::create_sync_objects(&device, MAX_FRAMES_IN_FLIGHT);

            let rain_renderer = crate::engine::environment::rain::RainRenderer::new(&device, render_pass, msaa_samples);
            let snow_renderer = crate::engine::environment::snow::SnowRenderer::new(&device, render_pass, msaa_samples);

            let mut renderer = Self {
                _entry: entry,
                instance,
                surface_loader,
                surface,
                physical_device,
                queue_family_index,
                device,
                graphics_queue,
                present_queue,
                swapchain_loader,
                swapchain,
                swapchain_format,
                swapchain_extent,
                swapchain_images,
                swapchain_image_views,
                render_pass,
                descriptor_set_layout,
                pipeline_layout,
                pipeline,
                outline_pipeline,
                skinned_pipeline_layout,
                skinned_descriptor_set_layout,
                skinned_pipeline,
                skinned_actors: Vec::new(),
                skinned_blases: std::collections::HashMap::new(),
                swapchain_framebuffers,
                command_pool,
                command_buffers,
                image_available_semaphores,
                render_finished_semaphores,
                in_flight_fences,
                current_frame: 0,
                mesh,
                raytracing,
                rt_model_blases,
                rt_model_base_positions,
                rt_model_offsets,
                rt_model_rotations,
                depth,
                descriptor_pool,
                ui_descriptor_pool,
                descriptor_sets,
                textures,
                sky_color,
                hiz_mip_views,
                hiz_sampler,
                hiz_image,
                hiz_memory,
                hiz_view,
                hiz_pipeline,
                hiz_pipeline_layout,
                hiz_descriptor_set_layout,
                hiz_descriptor_sets,
                hiz_mips,
                window,
                crosshair_renderer,
                crosshair_enabled: false,
                crosshair_style: CrosshairStyle::Dot,
                crosshair_scale: 1.0,
                msaa_samples: samples,
                color_msaa,
                color_msaa_memory,
                color_msaa_view,
                
                hdr_image,
                hdr_memory,
                hdr_view,
                hdr_format,
                
                composite_render_pass,
                composite_framebuffers,
                composite_renderer,
                composite_descriptor_sets,
                snow_renderer,
                rain_renderer,
                bloom_renderer,
                bloom_enabled: true,
                
                lighting_system,
            };

            if !initial_skinned.is_empty() {
                renderer.set_skinned_actors(&initial_skinned);
            }

            renderer
        }


     
    pub fn init_window(event_loop: &EventLoop<()>, fullscreen: bool) -> Arc<Window> {
        init::init_window(event_loop, fullscreen)
    }

    unsafe fn init_instance(window: &Window) -> (ash::Entry, ash::Instance, khr::surface::Instance, vk::SurfaceKHR) {
        init::init_instance(window)
    }

    unsafe fn init_swapchain(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        surface_loader: &khr::surface::Instance,
        surface: vk::SurfaceKHR,
        queue_family_index: u32,
        window: &Window,
    ) -> (khr::swapchain::Device, vk::SwapchainKHR, vk::Format, vk::Extent2D, Vec<vk::Image>, Vec<vk::ImageView>) {
        init::init_swapchain(instance, device, physical_device, surface_loader, surface, queue_family_index, window)
    }

    unsafe fn init_pipeline(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        msaa_samples: vk::SampleCountFlags,
    ) -> (vk::Pipeline, vk::PipelineLayout, vk::DescriptorSetLayout) {
        init::init_pipeline(device, render_pass, msaa_samples)
    }

    unsafe fn init_skinned_pipeline(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        msaa_samples: vk::SampleCountFlags,
    ) -> (vk::Pipeline, vk::PipelineLayout, vk::DescriptorSetLayout) {
        init::init_skinned_pipeline(device, render_pass, msaa_samples)
    }

    pub unsafe fn recreate_swapchain(&mut self) {
        self.device.device_wait_idle().unwrap();

        self.crosshair_renderer.destroy(&self.device);
        
         
        hiz::destroy_hiz_image_resources(&self.device, self.hiz_image, self.hiz_memory, self.hiz_view, &self.hiz_mip_views);
         
        
        for &fb in &self.swapchain_framebuffers {
            self.device.destroy_framebuffer(fb, None);
        }
        vk_memory::destroy_depth_resources(&self.device, &self.depth);
        if let Some(view) = self.color_msaa_view {
            self.device.destroy_image_view(view, None);
        }
        
         
        unsafe {
            self.device.destroy_image_view(self.hdr_view, None);
            self.device.destroy_image(self.hdr_image, None);
            self.device.free_memory(self.hdr_memory, None);
        }

         
        for framebuffer in &self.composite_framebuffers {
            unsafe { self.device.destroy_framebuffer(*framebuffer, None) };
        }
        unsafe { self.device.destroy_render_pass(self.composite_render_pass, None); }
        if let Some(image) = self.color_msaa {
            self.device.destroy_image(image, None);
        }
        if let Some(memory) = self.color_msaa_memory {
            self.device.free_memory(memory, None);
        }
        self.color_msaa_view = None;
        self.color_msaa = None;
        self.color_msaa_memory = None;

        self.device.destroy_pipeline(self.pipeline, None);
        self.device.destroy_pipeline(self.skinned_pipeline, None);
        self.device.destroy_render_pass(self.render_pass, None);
        for &view in &self.swapchain_image_views {
            self.device.destroy_image_view(view, None);
        }
        self.swapchain_loader
            .destroy_swapchain(self.swapchain, None);

        let size = self.window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let (swapchain_loader, swapchain, swapchain_format, swapchain_extent, swapchain_images) =
            vk_swapchain::create_swapchain(
                &self.instance,
                &self.device,
                self.physical_device,
                &self.surface_loader,
                self.surface,
                self.queue_family_index,
                width,
                height,
            );

        let swapchain_image_views =
            vk_swapchain::create_image_views(&self.device, &swapchain_images, swapchain_format);
        let depth = vk_memory::create_depth_resources_with_samples(
            &self.instance,
            self.physical_device,
            &self.device,
            swapchain_extent,
            self.msaa_samples,
        );

         
        let (hdr_image, hdr_memory) = unsafe {
            vk_memory::create_image(
                &self.instance,
                self.physical_device,
                &self.device,
                swapchain_extent.width,
                swapchain_extent.height,
                self.hdr_format,
                vk::ImageTiling::OPTIMAL,
                vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )
        };
        self.hdr_image = hdr_image;
        self.hdr_memory = hdr_memory;
        
        self.hdr_view = unsafe {
            self.device.create_image_view(
                &vk::ImageViewCreateInfo::default()
                    .image(hdr_image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(self.hdr_format)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    }),
                None,
            )
            .unwrap()
        };

        let (color_msaa, color_msaa_memory, color_msaa_view) = if self.msaa_samples != vk::SampleCountFlags::TYPE_1 {
            let (image, memory) = unsafe {
                vk_memory::create_image_with_samples(
                    &self.instance,
                    self.physical_device,
                    &self.device,
                    swapchain_extent.width,
                    swapchain_extent.height,
                    self.hdr_format,
                    vk::ImageTiling::OPTIMAL,
                    vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT,
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                    self.msaa_samples,
                )
            };
            let view_info = vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(self.hdr_format)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1)
                );
            let view = unsafe { self.device.create_image_view(&view_info, None).unwrap() };
            (Some(image), Some(memory), Some(view))
        } else {
            (None, None, None)
        };
        self.color_msaa = color_msaa;
        self.color_msaa_memory = color_msaa_memory;
        self.color_msaa_view = color_msaa_view;

        let render_pass = unsafe {
            vk_render_pass::create_render_pass(
                &self.device,
                self.hdr_format,
                depth.format,
                self.msaa_samples,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            )
        };
        self.render_pass = render_pass;
        
        let composite_render_pass = unsafe {
             vk_render_pass::create_render_pass(
                 &self.device,
                 swapchain_format,
                 depth.format,
                 vk::SampleCountFlags::TYPE_1,
                 vk::ImageLayout::PRESENT_SRC_KHR,
             )
        };
        self.composite_render_pass = composite_render_pass;
        
         
        hiz::free_hiz_descriptor_sets(&self.device, self.descriptor_pool, &self.hiz_descriptor_sets);
        
         
        let (hiz_image, hiz_memory, hiz_view, hiz_mip_views, hiz_mips, hiz_descriptor_sets) = 
            hiz::create_hiz_image_and_views(
                &self.instance,
                &self.device,
                self.physical_device,
                self.descriptor_pool,
                self.hiz_descriptor_set_layout,
                self.hiz_sampler,
                depth.view,
                width,
                height,
            );
        
         
        let old_lighting_resources = self.lighting_system.update_dimensions(
            &self.instance,
            &self.device,
            self.physical_device,
            self.command_pool,
            self.graphics_queue,
            width,
            height,
        );

         
        for i in 0..self.descriptor_sets.len() {
              
             let hiz_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(hiz_view)
                .sampler(self.hiz_sampler);

            let mut writes = vec![
                vk::WriteDescriptorSet::default()
                    .dst_set(self.descriptor_sets[i])
                    .dst_binding(8)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&hiz_info))
            ];
            
             
            let cluster_info;  
            let index_info;
            
            if old_lighting_resources.is_some() {
                cluster_info = vk::DescriptorBufferInfo::default()
                     .buffer(self.lighting_system.cluster_info_buffer)
                     .offset(0)
                     .range(vk::WHOLE_SIZE);
                index_info = vk::DescriptorBufferInfo::default()
                     .buffer(self.lighting_system.light_index_buffer)
                     .offset(0)
                     .range(vk::WHOLE_SIZE);
                
                writes.push(vk::WriteDescriptorSet::default()
                    .dst_set(self.descriptor_sets[i])
                    .dst_binding(12)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .buffer_info(std::slice::from_ref(&cluster_info)));
                    
                writes.push(vk::WriteDescriptorSet::default()
                    .dst_set(self.descriptor_sets[i])
                    .dst_binding(13)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .buffer_info(std::slice::from_ref(&index_info)));
            }
                
            self.device.update_descriptor_sets(&writes, &[]);
        }
        
         
        if let Some((old_c_buf, old_c_mem, old_i_buf, old_i_mem)) = old_lighting_resources {
            self.device.destroy_buffer(old_c_buf, None);
            self.device.free_memory(old_c_mem, None);
            self.device.destroy_buffer(old_i_buf, None);
            self.device.free_memory(old_i_mem, None);
        }

        let pipeline =
            vk_pipeline::create_pipeline(&self.device, self.pipeline_layout, render_pass, self.msaa_samples);
        let skinned_pipeline =
            vk_pipeline::create_skinned_pipeline(
                &self.device,
                self.skinned_pipeline_layout,
                render_pass,
                self.msaa_samples,
            );
        let crosshair_renderer = CrosshairRenderer::new(
            &self.instance,
            &self.device,
            self.physical_device,
            self.command_pool,
            self.graphics_queue,
            render_pass,
            self.crosshair_scale,
            self.msaa_samples,
        );
        let swapchain_framebuffers = {
             let mut attachments = Vec::new();
             if let Some(msaa_view) = color_msaa_view {
                 attachments.push(msaa_view);
                 attachments.push(depth.view);
                 attachments.push(self.hdr_view);
             } else {
                 attachments.push(self.hdr_view);
                 attachments.push(depth.view);
             }
             let create_info = vk::FramebufferCreateInfo::default()
                 .render_pass(render_pass)
                 .attachments(&attachments)
                 .width(swapchain_extent.width)
                 .height(swapchain_extent.height)
                 .layers(1);
             vec![unsafe { self.device.create_framebuffer(&create_info, None).unwrap() }]
        };
        
        let composite_framebuffers = swapchain_image_views
            .iter()
            .map(|&view| {
                 let attachments = [view, depth.view];
                 let create_info = vk::FramebufferCreateInfo::default()
                     .render_pass(composite_render_pass)
                     .attachments(&attachments)
                     .width(swapchain_extent.width)
                     .height(swapchain_extent.height)
                     .layers(1);
                 unsafe { self.device.create_framebuffer(&create_info, None).unwrap() }
            })
            .collect();
            
         
        unsafe {
            self.bloom_renderer.resize(&self.instance, self.physical_device, &self.device, self.descriptor_pool, swapchain_extent.width, swapchain_extent.height);
        }

        self.swapchain_loader = swapchain_loader;
        self.swapchain = swapchain;
        self.swapchain_format = swapchain_format;
        self.swapchain_extent = swapchain_extent;
        self.swapchain_images = swapchain_images;
        self.swapchain_image_views = swapchain_image_views;
        self.render_pass = render_pass;
        self.pipeline = pipeline;
        self.skinned_pipeline = skinned_pipeline;
        self.swapchain_framebuffers = swapchain_framebuffers;
        self.composite_framebuffers = composite_framebuffers;
        self.depth = depth;
        
         
         
         
         
         
         
         
        self.outline_pipeline = vk_pipeline::create_outline_pipeline(&self.device, self.pipeline_layout, render_pass, self.msaa_samples);
        self.hiz_image = hiz_image;
        self.hiz_memory = hiz_memory;
        self.hiz_view = hiz_view;
        self.hiz_mip_views = hiz_mip_views;  
        self.hiz_mips = hiz_mips;
        self.hiz_descriptor_sets = hiz_descriptor_sets;
        self.crosshair_renderer = crosshair_renderer;
    }

    pub unsafe fn resize_if_requested(&mut self, requested: bool) -> bool {
        if requested {
            self.recreate_swapchain();
            return true;
        }
        false
    }

    pub unsafe fn update_skinned(&mut self, dt: f32) {
        for actor in &mut self.skinned_actors {
            actor.instance.advance(dt);
            let skin_mats = actor.instance.sample_matrices();

            if skin_mats.is_empty() {
                continue;
            }

            let size =
                (skin_mats.len() * std::mem::size_of::<glam::Mat4>()) as vk::DeviceSize;
            let ptr = self
                .device
                .map_memory(
                    actor.bone_memory,
                    0,
                    size,
                    vk::MemoryMapFlags::empty(),
                )
                .unwrap();
            let slice = std::slice::from_raw_parts_mut(ptr as *mut u8, size as usize);
            slice.copy_from_slice(bytemuck::cast_slice(&skin_mats));
            self.device.unmap_memory(actor.bone_memory);
        }

    }

    pub unsafe fn update_model_offsets(&mut self, bodies: &[(usize, glam::Vec3)]) {
        if bodies.is_empty() {
             return;
        }

        let ptr = self.device.map_memory(self.mesh.material_buffer_memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()).unwrap();
        let model_count = self.mesh.meshlet_ranges.len();
        let material_slice = std::slice::from_raw_parts_mut(
            ptr as *mut crate::vulkan::vk_meshlets::GpuMaterial,
            model_count,
        );
        
        for (idx, offset) in bodies {
              
             if *idx < model_count {
                 if let Some(rt_offset) = self.rt_model_offsets.get_mut(*idx) {
                     *rt_offset = *offset;
                 }
                 (*material_slice.as_mut_ptr().add(*idx)).offset = [offset.x, offset.y, offset.z, 0.0];
             }
        }
        
        self.device.unmap_memory(self.mesh.material_buffer_memory);
    }

    pub unsafe fn update_model_rotations(&mut self, rotations: &[(usize, glam::Quat)]) {
        if rotations.is_empty() {
            return;
        }

        let ptr = self.device.map_memory(self.mesh.material_buffer_memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()).unwrap();
        let model_count = self.mesh.meshlet_ranges.len();
        let material_slice = std::slice::from_raw_parts_mut(
            ptr as *mut crate::vulkan::vk_meshlets::GpuMaterial,
            model_count,
        );

        for (idx, rotation) in rotations {
            if *idx < model_count {
                if let Some(rt_rotation) = self.rt_model_rotations.get_mut(*idx) {
                    *rt_rotation = *rotation;
                }
                (*material_slice.as_mut_ptr().add(*idx)).rotation = [
                    rotation.x,
                    rotation.y,
                    rotation.z,
                    rotation.w,
                ];
            }
        }

        self.device.unmap_memory(self.mesh.material_buffer_memory);
    }

    unsafe fn destroy_rt_model_blases(&mut self) {
        for blas in self.rt_model_blases.drain(..) {
            if let Some(blas) = blas {
                self.raytracing
                    .accel
                    .destroy_acceleration_structure(blas.accel, None);
                self.device.destroy_buffer(blas.buffer, None);
                self.device.free_memory(blas.memory, None);
            }
        }
    }

     
     
     
     
     
    pub unsafe fn update_models(&mut self, models: &[MeshletModelDesc]) -> Vec<Aabb> {
        self.device.device_wait_idle().unwrap();

         
        self.destroy_rt_model_blases();
        self.raytracing.destroy(&self.device);
        self.device
            .free_memory(self.mesh.meshlet_buffer_memory, None);
        self.device.destroy_buffer(self.mesh.meshlet_buffer, None);
        self.device
            .free_memory(self.mesh.vertex_buffer_memory, None);
        self.device.destroy_buffer(self.mesh.vertex_buffer, None);
        self.device
            .free_memory(self.mesh.meshlet_vertices_buffer_memory, None);
        self.device
            .destroy_buffer(self.mesh.meshlet_vertices_buffer, None);
        self.device
            .free_memory(self.mesh.meshlet_triangles_buffer_memory, None);
        self.device
            .destroy_buffer(self.mesh.meshlet_triangles_buffer, None);
        self.device
            .free_memory(self.mesh.index_buffer_memory, None);
        self.device.destroy_buffer(self.mesh.index_buffer, None);

         
        let (mesh, colliders) = vk_meshlets::create_resources(
            &self.instance,
            self.physical_device,
            &self.device,
            self.command_pool,
            self.graphics_queue,
            models,
        );
        let raytracing = vk_raytracing::build_scene_acceleration(
            &self.instance,
            self.physical_device,
            &self.device,
            self.command_pool,
            self.graphics_queue,
            mesh.vertex_buffer,
            mesh.index_buffer,
            mesh.vertex_count,
            mesh.index_count,
        );
        let rt_model_blases = vk_raytracing::build_model_blases(
            &self.instance,
            self.physical_device,
            &self.device,
            self.command_pool,
            self.graphics_queue,
            mesh.vertex_buffer,
            mesh.index_buffer,
            &mesh.model_rt_infos,
        );
        let rt_model_base_positions = mesh
            .model_rt_infos
            .iter()
            .map(|info| info.base_position)
            .collect::<Vec<_>>();
        let rt_model_offsets = vec![glam::Vec3::ZERO; rt_model_base_positions.len()];
        let rt_model_rotations = vec![glam::Quat::IDENTITY; rt_model_base_positions.len()];

        self.mesh = mesh;
        self.raytracing = raytracing;
        self.rt_model_blases = rt_model_blases;
        self.rt_model_base_positions = rt_model_base_positions;
        self.rt_model_offsets = rt_model_offsets;
        self.rt_model_rotations = rt_model_rotations;
        self.skinned_blases = std::collections::HashMap::new();

         
        let descriptor_sets = vk_sync::allocate_descriptor_sets(
            &self.device,
            self.descriptor_pool,
            self.descriptor_set_layout,
            self.mesh.meshlet_buffer,
            self.mesh.vertex_buffer,
            self.mesh.meshlet_vertices_buffer,
            self.mesh.meshlet_triangles_buffer,
            &self.textures,
             
             
             
            self.hiz_view,
            self.hiz_sampler,
            self.raytracing.tlas,
            self.mesh.material_buffer,
            self.mesh.meshlet_params_buffer,
            self.lighting_system.light_buffer,
            self.lighting_system.cluster_info_buffer,
            self.lighting_system.light_index_buffer,
            MAX_FRAMES_IN_FLIGHT,
        );
        self.descriptor_sets = descriptor_sets;

        colliders
    }

    pub unsafe fn record_command_buffer(
        &self,
        image_index: usize,
        current_frame: usize,
        frame: &FrameInput,
        text: Option<(&TextRenderer, Option<&[usize]>)>,
        menu: Option<&MenuRenderer>,
        cloud_renderer: Option<&CloudRenderer>,
        rain_renderer: Option<&RainRenderer>,
        stars_renderer: Option<&StarsRenderer>,
        sun_sphere_renderer: Option<&SunSphereRenderer>,
        mut fire_renderer: Option<&mut FireRenderer>,
        sdf_renderer: Option<&SdfRenderer>,
        hud_renderer: Option<(&HudRenderer, f32, bool, usize)>,  
    ) {
        let command_buffer = self.command_buffers[current_frame];

        self.device
            .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
            .unwrap();

        let begin_info =
            vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::empty());
        self.device
            .begin_command_buffer(command_buffer, &begin_info)
            .unwrap();
            
         
        self.lighting_system.record_cull(
            &self.device,
            command_buffer,
            frame.view,
            frame.proj,
            self.swapchain_extent.width,
            self.swapchain_extent.height,
            0.1,  
            1000.0,  
        );
        
         
        let memory_barrier = vk::MemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::SHADER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ);
            
        self.device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[memory_barrier],
            &[],
            &[],
        );
            
         
        if let Some(fr) = &mut fire_renderer {
            fr.update(&self.device, command_buffer, frame.dt.max(0.001), frame.effect_time);
        }

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: self.sky_color,
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        let render_pass_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(self.swapchain_framebuffers[0])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_extent,
            })
            .clear_values(&clear_values);

        self.device.cmd_begin_render_pass(
            command_buffer,
            &render_pass_info,
            vk::SubpassContents::INLINE,
        );

        self.device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline,
        );

        let viewport = vk::Viewport::default()
            .x(0.0)
            .y(0.0)
            .width(self.swapchain_extent.width as f32)
            .height(self.swapchain_extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0);
        self.device
            .cmd_set_viewport(command_buffer, 0, std::slice::from_ref(&viewport));

        let scissor = vk::Rect2D::default()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(self.swapchain_extent);
        self.device
            .cmd_set_scissor(command_buffer, 0, std::slice::from_ref(&scissor));

        self.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline_layout,
            0,
            &[self.descriptor_sets[current_frame]],
            &[],
        );

        let sphere_idx = frame.sphere_model_idx.map(|idx| idx as f32).unwrap_or(-1.0);
        let push_constants = PushConstants {
            mvp: frame.proj * frame.view,
            meshlet_count: self.mesh.meshlet_count,
            hovered_id: frame.hovered_model_idx.unwrap_or(u32::MAX),
            outline_width: 0.0,
            meshlet_offset: 0,
            sun_dir: [
                frame.sun_dir.x,
                frame.sun_dir.y,
                frame.sun_dir.z,
                frame.sun_intensity,
            ],
            camera_pos: [
                frame.camera_pos.x,
                frame.camera_pos.y,
                frame.camera_pos.z,
                0.0,
            ],
            sun_color: [
                frame.sun_color.x,
                frame.sun_color.y,
                frame.sun_color.z,
                1.0,
            ],
            screen_size: [
                self.swapchain_extent.width,
                self.swapchain_extent.height,
            ],
            _pad: [0, 0],
        };

        self.device.cmd_push_constants(
            command_buffer,
            self.pipeline_layout,
            vk::ShaderStageFlags::MESH_EXT
                | vk::ShaderStageFlags::TASK_EXT
                | vk::ShaderStageFlags::FRAGMENT,
            0,
            bytemuck::bytes_of(&push_constants),
        );

        let mesh_shader_fn = ext::mesh_shader::Device::new(&self.instance, &self.device);

         
        if let Some(hovered_idx) = frame.hovered_model_idx {
             let idx = hovered_idx as usize;
             if idx < self.mesh.meshlet_ranges.len() {
                 let (start, count) = self.mesh.meshlet_ranges[idx];
                 if count > 0 {
                    self.device.cmd_bind_pipeline(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.outline_pipeline,
                    );
                    
                    let mut outline_pc = push_constants;
                    outline_pc.outline_width = 0.03;  
                    outline_pc.meshlet_offset = start; 

                    self.device.cmd_push_constants(
                        command_buffer,
                        self.pipeline_layout,
                        vk::ShaderStageFlags::MESH_EXT
                            | vk::ShaderStageFlags::TASK_EXT
                            | vk::ShaderStageFlags::FRAGMENT,
                        0,
                        bytemuck::bytes_of(&outline_pc),
                    );

                    let outline_group_count = (count + 31) / 32;
                    mesh_shader_fn.cmd_draw_mesh_tasks(
                        command_buffer,
                        outline_group_count,
                        1,
                        1,
                    );
                 }
             }
        }

         
        self.device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline,
        );
         
         
        self.device.cmd_push_constants(
            command_buffer,
            self.pipeline_layout,
            vk::ShaderStageFlags::MESH_EXT
                | vk::ShaderStageFlags::TASK_EXT
                | vk::ShaderStageFlags::FRAGMENT,
            0,
            bytemuck::bytes_of(&push_constants),
        );

        let group_count = (self.mesh.meshlet_count + 31) / 32;
        mesh_shader_fn.cmd_draw_mesh_tasks(command_buffer, group_count, 1, 1);

        if let Some(fr) = &fire_renderer {
             unsafe {
                 fr.draw(
                     &self.instance,
                     &self.device,
                     command_buffer,
                     frame.view,
                     frame.proj,
                     frame.camera_pos,
                     frame.effect_time,
                     self.swapchain_extent,
                 );
             }
        }

         
        if let Some(sdf) = sdf_renderer {
             sdf.draw(&self.device, command_buffer, frame, self.swapchain_extent);
        }

         
        if let Some(sun_sphere) = sun_sphere_renderer {
            sun_sphere.record_commands(
                &self.device,
                command_buffer,
                self.swapchain_extent,
                frame.view,
                frame.proj,
                frame.sun_pos,
                frame.sun_color,
                frame.sun_intensity,
                frame.effect_time,
            );
        }

         
        if let Some(stars) = stars_renderer {
            stars.record_commands(
                &self.instance,
                &self.device,
                command_buffer,
                self.swapchain_extent,
                frame.view,
                frame.proj,
                frame.camera_pos,
                frame.wind_time,
                frame.sun_intensity,
            );
        }

         
        if let Some(clouds) = cloud_renderer {
             unsafe {
                 clouds.record_commands(
                    &self.instance,
                    &self.device,
                    command_buffer,
                    self.swapchain_extent,
                    frame.camera_pos,
                    frame.sun_dir,
                    frame.sun_color,
                    frame.sun_intensity,
                    frame.view,
                    frame.proj,
                    frame.wind_time,
                    &frame.cloud_config,
                    frame.wind_dir,
                    frame.wind_speed,
                    frame.wind_turbulence,
                );
            }
        }
         
        if frame.enable_rain {
             self.rain_renderer.record_commands(
                &self.instance,
                &self.device,
                command_buffer,
                self.swapchain_extent,
                frame.view,
                frame.proj,
                frame.camera_pos,
                frame.rain_time,
                &frame.rain_config,
                frame.wind_dir,
                frame.wind_speed,
                frame.wind_turbulence,
                1.0,  
            );
        }

         
        if frame.enable_snow {
             self.snow_renderer.record_commands(
                &self.instance,
                &self.device,
                command_buffer,
                self.swapchain_extent,
                frame.view,
                frame.proj,
                frame.camera_pos,
                frame.rain_time,
                &frame.snow_config,
                frame.wind_dir,
                frame.wind_speed,
                frame.wind_turbulence,
            );
        }
        if let Some((hud, health, show_inv, active_slot)) = hud_renderer {
            hud.record_commands(&self.device, command_buffer, self.swapchain_extent, health, show_inv, active_slot);
        }

         
        if !self.skinned_actors.is_empty() {
            for actor in &self.skinned_actors {
                let model = glam::Mat4::from_translation(actor.position)
                    * glam::Mat4::from_rotation_y(actor.rotation_y)
                    * glam::Mat4::from_scale(glam::Vec3::splat(actor.scale));
                let skinned_mvp = frame.proj * frame.view * model;
                let model_rot = glam::Quat::from_rotation_y(actor.rotation_y);
                let skinned_pc = SkinnedPushConstants {
                    mvp: skinned_mvp,
                    sun_dir: [
                        frame.sun_dir.x,
                        frame.sun_dir.y,
                        frame.sun_dir.z,
                        frame.sun_intensity,
                    ],
                    sun_color: [
                        frame.sun_color.x,
                        frame.sun_color.y,
                        frame.sun_color.z,
                        actor.meshlet_count as f32,
                    ],
                    model_pos_scale: [
                        actor.position.x,
                        actor.position.y,
                        actor.position.z,
                        actor.scale,
                    ],
                    model_rot: [model_rot.x, model_rot.y, model_rot.z, model_rot.w],
                };
            self.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.skinned_pipeline,
            );
            self.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.skinned_pipeline_layout,
                0,
                std::slice::from_ref(&actor.descriptor_set),
                &[],
            );
            self.device.cmd_push_constants(
                command_buffer,
                self.skinned_pipeline_layout,
                vk::ShaderStageFlags::MESH_EXT | vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&skinned_pc),
            );
            if actor.meshlet_count > 0 {
                mesh_shader_fn.cmd_draw_mesh_tasks(
                    command_buffer,
                    actor.meshlet_count,
                    1,
                    1,
                );
            }
            }
        }


        if let Some(menu_renderer) = menu {
            menu_renderer.record_commands(&self.device, command_buffer, self.swapchain_extent);
        }
        if self.crosshair_enabled {
            self.crosshair_renderer.record_commands(
                &self.device,
                command_buffer,
                self.swapchain_extent,
                self.crosshair_style,
            );
        }
        if let Some((text_renderer, indices)) = text {
            match indices {
                Some(list) => text_renderer.record_commands_subset(
                    &self.device,
                    command_buffer,
                    self.swapchain_extent,
                    list,
                ),
                None => text_renderer.record_commands(&self.device, command_buffer, self.swapchain_extent),
            }
        }
        self.device.cmd_end_render_pass(command_buffer);
        
         
        if self.bloom_enabled {
            self.bloom_renderer.dispatch(&self.device, command_buffer, self.hdr_image);
            
             
            let barrier = vk::ImageMemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .old_layout(vk::ImageLayout::GENERAL)
                .new_layout(vk::ImageLayout::GENERAL)  
                .image(self.bloom_renderer.bloom_image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,  
                    base_array_layer: 0,
                    layer_count: 1,
                });
             self.device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::empty(), &[], &[], &[barrier]);
        }
        
         
        let clear_none = [];
        let composite_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.composite_render_pass)
            .framebuffer(self.composite_framebuffers[image_index])  
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_extent,
            })
            .clear_values(&clear_none);  
            
        self.device.cmd_begin_render_pass(command_buffer, &composite_info, vk::SubpassContents::INLINE);
        
        let descriptor_set = self.composite_descriptor_sets[self.current_frame];
        self.composite_renderer.record_commands(
            &self.device,
            command_buffer,
            descriptor_set,
            self.hdr_view,
            self.bloom_renderer.bloom_view,  
            0.04,  
            self.bloom_enabled
        );
        self.device.cmd_end_render_pass(command_buffer);
        
        
         
        {
             
            let depth_barrier = vk::ImageMemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)  
                .old_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .new_layout(vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(self.depth.image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

             
            let hiz_barrier_pre = vk::ImageMemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::SHADER_READ)  
                .dst_access_mask(vk::AccessFlags::SHADER_WRITE)
                .old_layout(vk::ImageLayout::UNDEFINED)  
                .new_layout(vk::ImageLayout::GENERAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(self.hiz_image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: self.hiz_mips,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            
            self.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::LATE_FRAGMENT_TESTS | vk::PipelineStageFlags::TASK_SHADER_EXT,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[depth_barrier, hiz_barrier_pre],
            );

            self.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.hiz_pipeline,
            );
            
            let mut width = (self.swapchain_extent.width / 2).max(1);
            let mut height = (self.swapchain_extent.height / 2).max(1);

            for i in 0..self.hiz_mips {
                 
                let dispatch_x = (width + 31) / 32;
                let dispatch_y = (height + 31) / 32;
                
                 
                 
                 
                 
                
                self.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::COMPUTE,
                    self.hiz_pipeline_layout,
                    0,
                    &[self.hiz_descriptor_sets[i as usize]],
                    &[],
                );
                
                self.device.cmd_dispatch(command_buffer, dispatch_x, dispatch_y, 1);

                 
                 
                 
                 
                let barrier = vk::ImageMemoryBarrier::default()
                    .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                    .dst_access_mask(vk::AccessFlags::SHADER_READ)
                    .old_layout(vk::ImageLayout::GENERAL)
                    .new_layout(vk::ImageLayout::GENERAL)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(self.hiz_image)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: i,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                     
                self.device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::COMPUTE_SHADER,
                    vk::PipelineStageFlags::COMPUTE_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier],
                );
                
                width = (width / 2).max(1);
                height = (height / 2).max(1);
            }
            
             
            let hiz_barrier_final = vk::ImageMemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .old_layout(vk::ImageLayout::GENERAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(self.hiz_image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: self.hiz_mips,
                    base_array_layer: 0,
                    layer_count: 1,
                });
                
            self.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::VERTEX_SHADER | vk::PipelineStageFlags::TASK_SHADER_EXT,  
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[hiz_barrier_final],
            );
        }
        self.device.end_command_buffer(command_buffer).unwrap();
    }
}
