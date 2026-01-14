 
 
 
 

use super::Engine;
use super::gui::text::TextRenderer;
use super::gui::menu::MenuRenderer;
use super::environment::sun_sphere::SunSphereRenderer;
use super::environment::clouds::CloudRenderer;
use super::environment::rain::RainRenderer;
use super::environment::stars::StarsRenderer;
use super::environment::fire::FireRenderer;

impl Engine {
     
    pub(crate) fn update_equipped_text(&mut self) {
        let text = if let Some(item_id) = &self.player_state.equipped_item {
            if let Some(item) = self.player_state.inventory.iter().find(|i| i.id == *item_id) {
                format!("Held: {}", item.name)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        if text.is_empty() {
            if let Some(idx) = self.equipped_text_idx.take() {
                unsafe {
                    self.text_renderer.remove_text(idx, &self.renderer.device, self.renderer.ui_descriptor_pool);
                }
                self.handle_text_removed(idx);
            }
        } else {
            let idx = unsafe {
                self.text_renderer.replace_text(
                    self.equipped_text_idx,
                    &self.renderer.instance,
                    &self.renderer.device,
                    self.renderer.physical_device,
                    self.renderer.command_pool,
                    self.renderer.graphics_queue,
                    self.renderer.ui_descriptor_pool,
                    self.renderer.swapchain_extent,
                    &text,
                    [0.8, -0.9],
                    0.4,
                    None,
                )
            };
            self.equipped_text_idx = Some(idx);
        }
    }

     
    pub(crate) fn shift_optional_index(idx: &mut Option<usize>, removed: usize) {
        if let Some(value) = *idx {
            if value > removed {
                *idx = Some(value - 1);
            } else if value == removed {
                *idx = None;
            }
        }
    }

     
    pub(crate) fn handle_text_removed(&mut self, removed: usize) {
        Self::shift_optional_index(&mut self.debug_text_idx, removed);
        Self::shift_optional_index(&mut self.interaction_text_idx, removed);
        Self::shift_optional_index(&mut self.inventory_title_idx, removed);
        Self::shift_optional_index(&mut self.equipped_text_idx, removed);

        if let Some(pos) = self
            .inventory_icon_indices
            .iter()
            .position(|&idx| idx == removed)
        {
            self.inventory_icon_indices.remove(pos);
            if self.inventory_visible_icons > pos {
                self.inventory_visible_icons -= 1;
            }
        }

        for idx in &mut self.inventory_icon_indices {
            if *idx > removed {
                *idx -= 1;
            }
        }
        
        for i in 0..5 {
            Self::shift_optional_index(&mut self.hotbar_icon_indices[i], removed);
            if self.hotbar_icon_indices[i].is_none() {
                self.hotbar_icon_paths[i] = None;
            }
        }
        Self::shift_optional_index(&mut self.drag_icon_idx, removed);
    }

     
    pub(crate) unsafe fn update_debug_overlay(&mut self) {
        if !self.debug_mode {
            return;
        }

        let pos = self.player.position;
        let eye = self.player.eye_position();
        let vel = self.player.velocity;
        let speed = vel.length();
        let yaw = self.player.yaw.to_degrees();
        let pitch = self.player.pitch.to_degrees();

        let text = format!(
            "DEBUG MODE\n\
pos (feet): {:.2}, {:.2}, {:.2}\n\
eye (cam): {:.2}, {:.2}, {:.2}\n\
vel (m/s): {:.2}, {:.2}, {:.2} | speed: {:.2}\n\
yaw/pitch (deg): {:.1} / {:.1}",
            pos.x, pos.y, pos.z,
            eye.x, eye.y, eye.z,
            vel.x, vel.y, vel.z, speed,
            yaw, pitch
        );

        let idx = self.text_renderer.replace_text(
            self.debug_text_idx,
            &self.renderer.instance,
            &self.renderer.device,
            self.renderer.physical_device,
            self.renderer.command_pool,
            self.renderer.graphics_queue,
            self.renderer.ui_descriptor_pool,
            self.renderer.swapchain_extent,
            &text,
            [-0.7, -0.9],
            0.35,
            None,
        );
        self.debug_text_idx = Some(idx);
    }

     
    pub(crate) unsafe fn rebuild_overlays(&mut self) {
        self.equipped_text_idx = None;
        let pool = self.renderer.ui_descriptor_pool;
        self.text_renderer.destroy(&self.renderer.device, pool);
        self.text_renderer = TextRenderer::new(
            &self.renderer.device,
            self.renderer.render_pass,
            "assets/GothicA1-Regular.ttf",
            self.renderer.msaa_samples,
        );
        self.debug_text_idx = None;
        self.interaction_text_idx = None;
        self.inventory_title_idx = None;
        self.inventory_icon_indices.clear();
        self.inventory_visible_icons = 0;
        self.inventory_ui_dirty = true;
        
        let pool = self.renderer.ui_descriptor_pool;
        for elem in &self.current_gui {
            self.text_renderer.add_text(
                &self.renderer.instance,
                &self.renderer.device,
                self.renderer.physical_device,
                self.renderer.command_pool,
                self.renderer.graphics_queue,
                pool,
                self.renderer.swapchain_extent,
                &elem.text,
                elem.position,
                elem.scale,
                elem.action.clone(),
            );
        }

        self.menu_renderer.destroy(&self.renderer.device);
        self.menu_renderer = MenuRenderer::new(
            &self.renderer.instance,
            &self.renderer.device,
            self.renderer.physical_device,
            self.renderer.command_pool,
            self.renderer.graphics_queue,
            self.renderer.render_pass,
            self.renderer.swapchain_extent,
            self.renderer.msaa_samples,
        );
        self.sun_sphere_renderer.destroy(&self.renderer.device);
        self.sun_sphere_renderer =
            SunSphereRenderer::new(&self.renderer.device, self.renderer.render_pass, self.renderer.msaa_samples);
        self.cloud_renderer.destroy(&self.renderer.device);
        self.cloud_renderer = CloudRenderer::new(&self.renderer.device, self.renderer.render_pass, self.renderer.msaa_samples);

        self.rain_renderer.destroy(&self.renderer.device);
        self.rain_renderer = RainRenderer::new(&self.renderer.device, self.renderer.render_pass, self.renderer.msaa_samples);

        self.stars_renderer.destroy(&self.renderer.device);
        self.stars_renderer = StarsRenderer::new(&self.renderer.device, self.renderer.render_pass, self.renderer.msaa_samples);
        
        self.fire_renderer.destroy(&self.renderer.device);
        self.fire_renderer = FireRenderer::new(
            &self.renderer.instance,
            &self.renderer.device,
            self.renderer.physical_device,
            self.renderer.render_pass,
            self.renderer.msaa_samples,
        );
    }
}

