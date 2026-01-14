use ash::vk;
use std::ffi::CStr;

use crate::vulkan::vk_meshlets::PushConstants;

pub fn compile_shader(path: &str, kind: shaderc::ShaderKind) -> Vec<u32> {
    let source = std::fs::read_to_string(path)
        .or_else(|_| std::fs::read_to_string(format!("r-rune/{}", path)))
        .or_else(|_| std::fs::read_to_string(format!("../{}", path)))
        .expect("Failed to read shader");
    let compiler = shaderc::Compiler::new().unwrap();
    let mut options = shaderc::CompileOptions::new().unwrap();
    options.set_target_env(
        shaderc::TargetEnv::Vulkan,
        shaderc::EnvVersion::Vulkan1_3 as u32,
    );
    options.set_target_spirv(shaderc::SpirvVersion::V1_5);
    options.set_forced_version_profile(460, shaderc::GlslProfile::None);

    let binary = compiler
        .compile_into_spirv(&source, kind, path, "main", Some(&options))
        .expect("Shader compilation failed");

    binary.as_binary().to_vec()
}

pub unsafe fn create_descriptor_set_layout(
    device: &ash::Device,
) -> (vk::PipelineLayout, vk::DescriptorSetLayout) {
    let bindings = [
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::TASK_EXT | vk::ShaderStageFlags::MESH_EXT),
        vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::MESH_EXT),
        vk::DescriptorSetLayoutBinding::default()
            .binding(2)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::MESH_EXT),
        vk::DescriptorSetLayoutBinding::default()
            .binding(3)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::MESH_EXT),
        vk::DescriptorSetLayoutBinding::default()
            .binding(4)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(4)  
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
         
        vk::DescriptorSetLayoutBinding::default()
            .binding(7)
            .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        vk::DescriptorSetLayoutBinding::default()
            .binding(8)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::TASK_EXT),
         
        vk::DescriptorSetLayoutBinding::default()
            .binding(10)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::MESH_EXT | vk::ShaderStageFlags::TASK_EXT),
         
        vk::DescriptorSetLayoutBinding::default()
            .binding(11)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::TASK_EXT | vk::ShaderStageFlags::MESH_EXT),
         
        vk::DescriptorSetLayoutBinding::default()
            .binding(12)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
         
        vk::DescriptorSetLayoutBinding::default()
            .binding(13)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
         
        vk::DescriptorSetLayoutBinding::default()
            .binding(14)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::TASK_EXT | vk::ShaderStageFlags::MESH_EXT),
    ];

    let binding_flags = [
        vk::DescriptorBindingFlags::empty(),
        vk::DescriptorBindingFlags::empty(),
        vk::DescriptorBindingFlags::empty(),
        vk::DescriptorBindingFlags::empty(),
        vk::DescriptorBindingFlags::PARTIALLY_BOUND,
        vk::DescriptorBindingFlags::empty(),  
        vk::DescriptorBindingFlags::empty(),  
        vk::DescriptorBindingFlags::empty(),  
        vk::DescriptorBindingFlags::empty(),  
        vk::DescriptorBindingFlags::empty(),  
        vk::DescriptorBindingFlags::empty(),  
        vk::DescriptorBindingFlags::empty(),  
    ];
    let mut binding_flags_create_info = vk::DescriptorSetLayoutBindingFlagsCreateInfo::default()
        .binding_flags(&binding_flags);

    let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
        .bindings(&bindings);
         
    
    let descriptor_set_layout = device
        .create_descriptor_set_layout(&layout_info.push_next(&mut binding_flags_create_info), None)
        .unwrap();

    let push_constant_range = vk::PushConstantRange::default()
        .stage_flags(
            vk::ShaderStageFlags::MESH_EXT
                | vk::ShaderStageFlags::TASK_EXT
                | vk::ShaderStageFlags::FRAGMENT,
        )
        .offset(0)
        .size(std::mem::size_of::<PushConstants>() as u32);

    let layout_create_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(std::slice::from_ref(&descriptor_set_layout))
        .push_constant_ranges(std::slice::from_ref(&push_constant_range));

    let pipeline_layout = device
        .create_pipeline_layout(&layout_create_info, None)
        .unwrap();

    (pipeline_layout, descriptor_set_layout)
}

pub unsafe fn create_pipeline(
    device: &ash::Device,
    layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
    msaa_samples: vk::SampleCountFlags,
) -> vk::Pipeline {
    let task_code = compile_shader("shader/meshlet.task", shaderc::ShaderKind::Task);
    let mesh_code = compile_shader("shader/meshlet.mesh", shaderc::ShaderKind::Mesh);
    let frag_code = compile_shader("shader/meshlet.frag", shaderc::ShaderKind::Fragment);

    let task_module = device
        .create_shader_module(
            &vk::ShaderModuleCreateInfo::default().code(&task_code),
            None,
        )
        .unwrap();
    let mesh_module = device
        .create_shader_module(
            &vk::ShaderModuleCreateInfo::default().code(&mesh_code),
            None,
        )
        .unwrap();
    let frag_module = device
        .create_shader_module(
            &vk::ShaderModuleCreateInfo::default().code(&frag_code),
            None,
        )
        .unwrap();

    let main_name = CStr::from_bytes_with_nul(b"main\0").unwrap();

    let shader_stages = [
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::TASK_EXT)
            .module(task_module)
            .name(main_name),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::MESH_EXT)
            .module(mesh_module)
            .name(main_name),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_module)
            .name(main_name),
    ];

    let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
         
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE);

    let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(false);

    let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
        .attachments(std::slice::from_ref(&color_blend_attachment));

    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS);

    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);

    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state =
        vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

    let multisample = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(msaa_samples);

    let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&shader_stages)
        .rasterization_state(&rasterizer)
        .color_blend_state(&color_blending)
        .depth_stencil_state(&depth_stencil)
        .viewport_state(&viewport_state)
        .dynamic_state(&dynamic_state)
        .layout(layout)
        .render_pass(render_pass)
        .multisample_state(&multisample);

    let pipeline = device
        .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
        .unwrap()[0];

    device.destroy_shader_module(task_module, None);
    device.destroy_shader_module(mesh_module, None);
    device.destroy_shader_module(frag_module, None);

    pipeline
}

pub unsafe fn create_outline_pipeline(
    device: &ash::Device,
    layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
    msaa_samples: vk::SampleCountFlags,
) -> vk::Pipeline {
    let task_code = compile_shader("shader/meshlet.task", shaderc::ShaderKind::Task);
    let mesh_code = compile_shader("shader/meshlet.mesh", shaderc::ShaderKind::Mesh);
    let frag_code = compile_shader("shader/meshlet.frag", shaderc::ShaderKind::Fragment);

    let task_module = device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&task_code), None).unwrap();
    let mesh_module = device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&mesh_code), None).unwrap();
    let frag_module = device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&frag_code), None).unwrap();

    let main_name = CStr::from_bytes_with_nul(b"main\0").unwrap();

    let shader_stages = [
        vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::TASK_EXT).module(task_module).name(main_name),
        vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::MESH_EXT).module(mesh_module).name(main_name),
        vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::FRAGMENT).module(frag_module).name(main_name),
    ];

    let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::FRONT)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE);

    let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default().color_write_mask(vk::ColorComponentFlags::RGBA).blend_enable(false);
    let color_blending = vk::PipelineColorBlendStateCreateInfo::default().attachments(std::slice::from_ref(&color_blend_attachment));

    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL);

    let viewport_state = vk::PipelineViewportStateCreateInfo::default().viewport_count(1).scissor_count(1);
    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
    let multisample = vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(msaa_samples);

    let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&shader_stages)
        .rasterization_state(&rasterizer)
        .color_blend_state(&color_blending)
        .depth_stencil_state(&depth_stencil)
        .viewport_state(&viewport_state)
        .dynamic_state(&dynamic_state)
        .layout(layout)
        .render_pass(render_pass)
        .multisample_state(&multisample);

    let pipeline = device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None).unwrap()[0];

    device.destroy_shader_module(task_module, None);
    device.destroy_shader_module(mesh_module, None);
    device.destroy_shader_module(frag_module, None);

    pipeline
}

pub unsafe fn create_skinned_descriptor_set_layout(
    device: &ash::Device,
) -> (vk::PipelineLayout, vk::DescriptorSetLayout) {
    let bindings = [
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::MESH_EXT),
        vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::MESH_EXT),
        vk::DescriptorSetLayoutBinding::default()
            .binding(2)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::MESH_EXT),
        vk::DescriptorSetLayoutBinding::default()
            .binding(3)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::MESH_EXT),
        vk::DescriptorSetLayoutBinding::default()
            .binding(4)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::MESH_EXT),
        vk::DescriptorSetLayoutBinding::default()
            .binding(5)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        vk::DescriptorSetLayoutBinding::default()
            .binding(6)
            .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
    ];
    let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    let descriptor_set_layout = device
        .create_descriptor_set_layout(&layout_info, None)
        .unwrap();

    let push_constant_range = vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::MESH_EXT | vk::ShaderStageFlags::FRAGMENT)
        .offset(0)
        .size(128);  

    let layout_create_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(std::slice::from_ref(&descriptor_set_layout))
        .push_constant_ranges(std::slice::from_ref(&push_constant_range));

    let pipeline_layout = device
        .create_pipeline_layout(&layout_create_info, None)
        .unwrap();

    (pipeline_layout, descriptor_set_layout)
}

pub unsafe fn create_skinned_pipeline(
    device: &ash::Device,
    layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
    msaa_samples: vk::SampleCountFlags,
) -> vk::Pipeline {
    let mesh_code = compile_shader("shader/fox.mesh", shaderc::ShaderKind::Mesh);
    let frag_code = compile_shader("shader/fox.frag", shaderc::ShaderKind::Fragment);

    let mesh_module = device
        .create_shader_module(
            &vk::ShaderModuleCreateInfo::default().code(&mesh_code),
            None,
        )
        .unwrap();
    let frag_module = device
        .create_shader_module(
            &vk::ShaderModuleCreateInfo::default().code(&frag_code),
            None,
        )
        .unwrap();

    let main_name = CStr::from_bytes_with_nul(b"main\0").unwrap();
    let shader_stages = [
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::MESH_EXT)
            .module(mesh_module)
            .name(main_name),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_module)
            .name(main_name),
    ];

    let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE);

    let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(false);
    let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
        .attachments(std::slice::from_ref(&color_blend_attachment));

    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS);

    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);
    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state =
        vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

    let multisample = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(msaa_samples);

    let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&shader_stages)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterizer)
        .multisample_state(&multisample)
        .depth_stencil_state(&depth_stencil)
        .color_blend_state(&color_blending)
        .dynamic_state(&dynamic_state)
        .layout(layout)
        .render_pass(render_pass);

    let pipeline = device
        .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
        .unwrap()[0];

    device.destroy_shader_module(mesh_module, None);
    device.destroy_shader_module(frag_module, None);

    pipeline
}

pub unsafe fn create_hiz_descriptor_set_layout(
    device: &ash::Device,
) -> (vk::PipelineLayout, vk::DescriptorSetLayout) {
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

    let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    let descriptor_set_layout = device
        .create_descriptor_set_layout(&layout_info, None)
        .unwrap();

    let layout_create_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(std::slice::from_ref(&descriptor_set_layout));

    let pipeline_layout = device
        .create_pipeline_layout(&layout_create_info, None)
        .unwrap();

    (pipeline_layout, descriptor_set_layout)
}

pub unsafe fn create_hiz_pipeline(
    device: &ash::Device,
    layout: vk::PipelineLayout,
) -> vk::Pipeline {
    let comp_code = compile_shader("shader/hiz.comp", shaderc::ShaderKind::Compute);
    
    let comp_module = device
        .create_shader_module(
            &vk::ShaderModuleCreateInfo::default().code(&comp_code),
            None,
        )
        .unwrap();

    let main_name = CStr::from_bytes_with_nul(b"main\0").unwrap();
    let stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::COMPUTE)
        .module(comp_module)
        .name(main_name);

    let pipeline_info = vk::ComputePipelineCreateInfo::default()
        .stage(stage)
        .layout(layout);

    let pipeline = device
        .create_compute_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
        .unwrap()[0];

    device.destroy_shader_module(comp_module, None);

    pipeline
}

pub unsafe fn create_compute_pipeline(
    device: &ash::Device,
    layout: vk::PipelineLayout,
    shader_path: &str,
) -> vk::Pipeline {
    let comp_code = compile_shader(shader_path, shaderc::ShaderKind::Compute);
    
    let comp_module = device
        .create_shader_module(
            &vk::ShaderModuleCreateInfo::default().code(&comp_code),
            None,
        )
        .unwrap();

    let main_name = CStr::from_bytes_with_nul(b"main\0").unwrap();
    let stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::COMPUTE)
        .module(comp_module)
        .name(main_name);

    let pipeline_info = vk::ComputePipelineCreateInfo::default()
        .stage(stage)
        .layout(layout);

    let pipeline = device
        .create_compute_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
        .unwrap()[0];

    device.destroy_shader_module(comp_module, None);

    pipeline
}
