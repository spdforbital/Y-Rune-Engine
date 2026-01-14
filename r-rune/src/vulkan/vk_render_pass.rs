use ash::vk;

pub unsafe fn create_render_pass(
    device: &ash::Device,
    color_format: vk::Format,
    depth_format: vk::Format,
    msaa_samples: vk::SampleCountFlags,
    final_layout: vk::ImageLayout,
) -> vk::RenderPass {
    let color_attachment = vk::AttachmentDescription::default()
        .format(color_format)
        .samples(msaa_samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(if msaa_samples == vk::SampleCountFlags::TYPE_1 {
            vk::AttachmentStoreOp::STORE
        } else {
            vk::AttachmentStoreOp::DONT_CARE
        })
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(if msaa_samples == vk::SampleCountFlags::TYPE_1 {
            final_layout
        } else {
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
        });

    let depth_attachment = vk::AttachmentDescription::default()
        .format(depth_format)
        .samples(msaa_samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

    let resolve_attachment = vk::AttachmentDescription::default()
        .format(color_format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::DONT_CARE)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(final_layout);

    let color_attachment_ref = vk::AttachmentReference::default()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let depth_attachment_ref = vk::AttachmentReference::default()
        .attachment(1)
        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

    let resolve_attachment_ref = vk::AttachmentReference::default()
        .attachment(2)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let mut attachments = vec![color_attachment, depth_attachment];
    if msaa_samples != vk::SampleCountFlags::TYPE_1 {
        attachments.push(resolve_attachment);
    }

    let mut subpass = vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(std::slice::from_ref(&color_attachment_ref))
        .depth_stencil_attachment(&depth_attachment_ref);

    if msaa_samples != vk::SampleCountFlags::TYPE_1 {
        subpass = subpass.resolve_attachments(std::slice::from_ref(&resolve_attachment_ref));
    }

    let dependency = vk::SubpassDependency::default()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
        .dst_access_mask(
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        );

    let create_info = vk::RenderPassCreateInfo::default()
        .attachments(&attachments)
        .subpasses(std::slice::from_ref(&subpass))
        .dependencies(std::slice::from_ref(&dependency));

    device.create_render_pass(&create_info, None).unwrap()
}

pub unsafe fn create_framebuffers(
    device: &ash::Device,
    views: &[vk::ImageView],
    depth_view: vk::ImageView,
    color_view_msaa: Option<vk::ImageView>,
    rp: vk::RenderPass,
    extent: vk::Extent2D,
) -> Vec<vk::Framebuffer> {
    views
        .iter()
        .map(|&view| {
            let mut attachments = Vec::new();
            if let Some(msaa_view) = color_view_msaa {
                attachments.push(msaa_view);
                attachments.push(depth_view);
                attachments.push(view);
            } else {
                attachments.push(view);
                attachments.push(depth_view);
            }
            
            let create_info = vk::FramebufferCreateInfo::default()
                .render_pass(rp)
                .attachments(&attachments)
                .width(extent.width)
                .height(extent.height)
                .layers(1);
            device.create_framebuffer(&create_info, None).unwrap()
        })
        .collect()
}
