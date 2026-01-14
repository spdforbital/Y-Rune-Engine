 
 
 
 

use crate::engine::gui::hud::{INV_COLS, INV_ROWS, INV_SLOT_GAP, INV_SLOT_SIZE};
use super::Engine;

impl Engine {
     
    pub(crate) fn inventory_slot_center(slot_index: usize) -> Option<[f32; 2]> {
        let max_slots = INV_ROWS * INV_COLS;
        if slot_index >= max_slots {
            return None;
        }

        let total_w = INV_COLS as f32 * INV_SLOT_SIZE + (INV_COLS as f32 - 1.0) * INV_SLOT_GAP;
        let total_h = INV_ROWS as f32 * INV_SLOT_SIZE + (INV_ROWS as f32 - 1.0) * INV_SLOT_GAP;
        let start_x = -total_w * 0.5;
        let start_y = -total_h * 0.5;

        let row = slot_index / INV_COLS;
        let col = slot_index % INV_COLS;
        let x0 = start_x + col as f32 * (INV_SLOT_SIZE + INV_SLOT_GAP);
        let y0 = start_y + row as f32 * (INV_SLOT_SIZE + INV_SLOT_GAP);

        Some([x0 + INV_SLOT_SIZE * 0.5, y0 + INV_SLOT_SIZE * 0.5])
    }
    
     
    pub(crate) fn hotbar_slot_center(slot: usize) -> Option<[f32; 2]> {
        if slot >= 5 { return None; }
        let slots = 5;
        let slot_size = 0.12; 
        let gap = 0.02;
        let total_w = slots as f32 * slot_size + (slots as f32 - 1.0) * gap;
        let bottom_y = 0.95; 
        let start_x = -total_w * 0.5;
        let start_y = bottom_y - slot_size;
        
        let x0 = start_x + slot as f32 * (slot_size + gap);
        let y0 = start_y;
        
        Some([x0 + slot_size * 0.5, y0 + slot_size * 0.5])
    }

     
    pub fn select_hotbar_slot(&mut self, slot: usize) {
        if slot < 5 {
            self.player_state.active_hotbar_slot = slot;
            self.player_state.equipped_item = self.player_state.hotbar[slot].as_ref().map(|i| i.id.clone());
            self.update_equipped_text();
        }
    }

     
    pub fn toggle_inventory(&mut self) {
        self.inventory_open = !self.inventory_open;
        if self.inventory_open {
            self.inventory_ui_dirty = true;
            self.renderer.window.set_cursor_grab(winit::window::CursorGrabMode::None).ok();
            self.renderer.window.set_cursor_visible(true);
        } else if !self.menu_open {
            self.renderer.window.set_cursor_grab(winit::window::CursorGrabMode::Confined).ok();
            self.renderer.window.set_cursor_visible(false);
        }
    }

     
    pub(crate) unsafe fn sync_inventory_ui(&mut self) {
        // Inventory Grid Updates
        if self.inventory_open {
            if self.inventory_ui_dirty || self.inventory_title_idx.is_none() {
                 let title_idx = self.text_renderer.replace_text(
                    self.inventory_title_idx,
                    &self.renderer.instance,
                    &self.renderer.device,
                    self.renderer.physical_device,
                    self.renderer.command_pool,
                    self.renderer.graphics_queue,
                    self.renderer.ui_descriptor_pool,
                    self.renderer.swapchain_extent,
                    "Inventory.",
                    [0.0, -0.45],
                    0.6,
                    None,
                );
                self.inventory_title_idx = Some(title_idx);

                let icon_items: Vec<_> = self
                    .player_state
                    .inventory
                    .iter()
                    .filter(|item| item.icon.is_some())
                    .collect();
                let max_icons = INV_ROWS * INV_COLS;
                let icon_count = icon_items.len().min(max_icons);

                let extent = self.renderer.swapchain_extent;
                let icon_ndc = INV_SLOT_SIZE * 0.8;
                let icon_size_px = [
                    icon_ndc * extent.width as f32 * 0.5,
                    icon_ndc * extent.height as f32 * 0.5,
                ];

                for (slot, item) in icon_items.into_iter().take(icon_count).enumerate() {
                    let Some(icon_path) = item.icon.as_ref() else {
                        continue;
                    };
                    let Some(position) = Self::inventory_slot_center(slot) else {
                        continue;
                    };

                    if slot < self.inventory_icon_indices.len() {
                        let idx = self.inventory_icon_indices[slot];
                        let new_idx = self.text_renderer.replace_image(
                            Some(idx),
                            &self.renderer.instance,
                            &self.renderer.device,
                            self.renderer.physical_device,
                            self.renderer.command_pool,
                            self.renderer.graphics_queue,
                            self.renderer.ui_descriptor_pool,
                            self.renderer.swapchain_extent,
                            icon_path,
                            position,
                            icon_size_px,
                        );
                        self.inventory_icon_indices[slot] = new_idx;
                    } else {
                        let idx = self.text_renderer.add_image(
                            &self.renderer.instance,
                            &self.renderer.device,
                            self.renderer.physical_device,
                            self.renderer.command_pool,
                            self.renderer.graphics_queue,
                            self.renderer.ui_descriptor_pool,
                            self.renderer.swapchain_extent,
                            icon_path,
                            position,
                            icon_size_px,
                        );
                        self.inventory_icon_indices.push(idx);
                    }
                }

                self.inventory_visible_icons = icon_count;
                self.inventory_ui_dirty = false;
            }
        }
        
         
        for i in 0..5 {
            let item_opt = &self.player_state.hotbar[i];
            let current_icon = item_opt.as_ref().and_then(|item| item.icon.clone());
            
             
            if self.hotbar_icon_paths[i] != current_icon {
                 
                self.hotbar_icon_paths[i] = current_icon.clone();
                
                if let Some(path) = current_icon {
                        let pos = Self::hotbar_slot_center(i).unwrap();
                        let icon_ndc = 0.12 * 0.8; 
                        let icon_size = [
                            icon_ndc * self.renderer.swapchain_extent.width as f32 * 0.5,
                            icon_ndc * self.renderer.swapchain_extent.height as f32 * 0.5,
                        ];

                        if let Some(idx) = self.hotbar_icon_indices[i] {
                            let new_idx = self.text_renderer.replace_image(
                                Some(idx),
                                &self.renderer.instance,
                                &self.renderer.device,
                                self.renderer.physical_device,
                                self.renderer.command_pool,
                                self.renderer.graphics_queue,
                                self.renderer.ui_descriptor_pool,
                                self.renderer.swapchain_extent,
                                &path,
                                pos,
                                icon_size,
                            );
                            self.hotbar_icon_indices[i] = Some(new_idx);
                        } else {
                            let idx = self.text_renderer.add_image(
                                &self.renderer.instance,
                                &self.renderer.device,
                                self.renderer.physical_device,
                                self.renderer.command_pool,
                                self.renderer.graphics_queue,
                                self.renderer.ui_descriptor_pool,
                                self.renderer.swapchain_extent,
                                &path,
                                pos,
                                icon_size,
                            );
                            self.hotbar_icon_indices[i] = Some(idx);
                        }
                } else {
                     
                    if let Some(idx) = self.hotbar_icon_indices[i].take() {
                        self.text_renderer.remove_text(idx, &self.renderer.device, self.renderer.ui_descriptor_pool);
                        self.handle_text_removed(idx);
                    }
                }
            }
        }
    }
}
