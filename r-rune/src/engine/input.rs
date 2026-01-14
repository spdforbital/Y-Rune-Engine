 
 
 

use crate::engine::gui::hud::{INV_COLS, INV_ROWS, INV_SLOT_SIZE};
use crate::engine::components::{Position, PhysicsBody, Velocity, StaticMesh};
use crate::vulkan::vk_meshlets::{Aabb, ColliderType};
use super::{Engine, DragSource};



impl Engine {
     
    pub fn handle_click(&mut self, x: f64, y: f64) {
        if self.inventory_open {
            self.handle_inventory_click(x, y);
            return;
        }

        if !self.menu_open {
            self.handle_game_click();
            return;
        }

        let width = self.renderer.swapchain_extent.width as f32;
        let height = self.renderer.swapchain_extent.height as f32;

         
        let ndc_x = (x as f32 / width) * 2.0 - 1.0;
        let ndc_y = (y as f32 / height) * 2.0 - 1.0;

         
        let mut action_to_run = None;

        for instance in &self.text_renderer.instances {
            let half_w_ndc = (instance.width as f32 * instance.scale) / width;
            let half_h_ndc = (instance.height as f32 * instance.scale) / height;

            let left = instance.position[0] - half_w_ndc;
            let right = instance.position[0] + half_w_ndc;
            let top = instance.position[1] - half_h_ndc;
            let bottom = instance.position[1] + half_h_ndc;
            
            println!("Click NDC: ({:.2}, {:.2}) | Btn '{}' Bounds: X[{:.2}, {:.2}] Y[{:.2}, {:.2}]", 
                ndc_x, ndc_y, instance.text, left, right, top, bottom);

            if ndc_x >= left && ndc_x <= right && ndc_y >= top && ndc_y <= bottom {
                if let Some(act) = &instance.action {
                    action_to_run = Some(act.clone());
                    println!("Matched action: {}", act);
                    break;
                }
            }
        }
        
        if let Some(action) = action_to_run {
            if let Some(scene_name) = action.strip_prefix("load_scene:") {
                self.load_scene(scene_name);
            } else {
                println!("Unknown action: {}", action);
            }
        }
    }

     
    pub fn handle_mouse_down(&mut self, x: f64, y: f64) {
        if self.menu_open {
            self.handle_click(x, y);
            return;
        }
         
        for i in 0..5 {
            if let Some(center) = Self::hotbar_slot_center(i) {
                if self.is_click_in_slot(x, y, center, 0.12) {
                    if let Some(item) = &self.player_state.hotbar[i] {
                        self.drag_source = Some(DragSource::Hotbar(i));
                        self.dragged_item = Some(item.clone());
                        println!("Dragging hotbar item from {}", i);
                        return;
                    }
                }
            }
        }
        
         
        if self.inventory_open {
            let icon_items: Vec<_> = self.player_state.inventory.iter().enumerate().filter(|(_, item)| item.icon.is_some()).collect();
            let max_icons = INV_ROWS * INV_COLS;
            let icon_count = icon_items.len().min(max_icons);
            
            for (visual_idx, (real_idx, item)) in icon_items.into_iter().take(icon_count).enumerate() {
                if let Some(center) = Self::inventory_slot_center(visual_idx) {
                    if self.is_click_in_slot(x, y, center, INV_SLOT_SIZE) {
                        self.drag_source = Some(DragSource::Inventory(real_idx));
                        self.dragged_item = Some(item.clone());
                        println!("Dragging inventory item idx {}", real_idx);
                        return;
                    }
                }
            }
        }
        
        if !self.inventory_open {
            self.handle_game_click();
        }
    }
    
     
    pub fn handle_mouse_up(&mut self, x: f64, y: f64) {
        if let Some(source) = self.drag_source {
            if let Some(dragged) = self.dragged_item.clone() {
                let mut dropped = false;
                
                 
                for i in 0..5 {
                    if let Some(center) = Self::hotbar_slot_center(i) {
                        if self.is_click_in_slot(x, y, center, 0.12) {
                            let target_item = self.player_state.hotbar[i].clone();
                            self.player_state.hotbar[i] = Some(dragged.clone());
                            
                            match source {
                                DragSource::Inventory(inv_idx) => {
                                    if inv_idx < self.player_state.inventory.len() && self.player_state.inventory[inv_idx].id == dragged.id {
                                        self.player_state.inventory.remove(inv_idx);
                                    }
                                    if let Some(old) = target_item {
                                        self.player_state.add_item(old);
                                    }
                                }
                                DragSource::Hotbar(src_slot) => {
                                    self.player_state.hotbar[src_slot] = target_item;
                                }
                            }
                            dropped = true;
                            println!("Dropped on Hotbar {}", i);
                        }
                    }
                }
                
                 
                if !dropped && self.inventory_open {
                    if let DragSource::Hotbar(src_slot) = source {
                        self.player_state.hotbar[src_slot] = None;
                        self.player_state.add_item(dragged.clone());
                        dropped = true;
                        println!("Unequipped hotbar item -> Inventory");
                    }
                }
                
                let _ = dropped;  
                self.inventory_ui_dirty = true;
                self.select_hotbar_slot(self.player_state.active_hotbar_slot);
            }
        }
        self.drag_source = None;
        self.dragged_item = None;
    }

     
    pub(crate) fn is_click_in_slot(&self, x: f64, y: f64, center_ndc: [f32; 2], size_ndc: f32) -> bool {
        let width = self.renderer.swapchain_extent.width as f32;
        let height = self.renderer.swapchain_extent.height as f32;
        let ndc_x = (x as f32 / width) * 2.0 - 1.0;
        let ndc_y = (y as f32 / height) * 2.0 - 1.0;
        
        let half_size = size_ndc * 0.5;
        let dx = (ndc_x - center_ndc[0]).abs();
        let dy = (ndc_y - center_ndc[1]).abs();
        
        dx < half_size && dy < half_size
    }

     
    pub(crate) fn handle_inventory_click(&mut self, _x: f64, _y: f64) {
         
    }

     
    pub(crate) fn handle_game_click(&mut self) {
         
         
        if self.firearm_system.equipped_is_firearm(&self.player_state, &self.model_paths, &self.dragged_item) {
            let (forward, right, up) = self.player.view_axes();
            let eye = self.player.eye_position();
            self.firearm_system.fire_bullet(
                &mut self.renderer,
                &self.player_state,
                &self.model_base_positions,
                &self.world_colliders,
                &self.collider_to_model,
                &mut self.world,
                eye,
                forward,
                right, 
                up
            );
            return;
        }
        if let Some(item_id) = self.player_state.equipped_item.clone() {
            let (forward, _, _) = self.player.view_axes();
            let eye = self.player.eye_position();
            let ray_dir = forward.normalize();
            let max_dist = 5.0;
            
            let mut best_dist = max_dist;
            let mut hit_point = None;

            for aabb in &self.world_colliders {
                if let Some(dist) = aabb.ray_intersect(eye, ray_dir) {
                    if dist < best_dist {
                        best_dist = dist;
                        hit_point = Some(eye + ray_dir * dist);
                    }
                }
            }
            
            for (_id, (pos, body)) in self.world.query_mut::<(&Position, &PhysicsBody)>() {
                if !body.active { continue; }
                let extents = glam::Vec3::splat(body.radius);
                let aabb = Aabb::new(pos.0 - extents, pos.0 + extents, ColliderType::Solid);
                if let Some(dist) = aabb.ray_intersect(eye, ray_dir) {
                    if dist < best_dist {
                        best_dist = dist;
                        hit_point = Some(eye + ray_dir * dist);
                    }
                }
            }

            if let Some(pos_val) = hit_point {
                if let Some(model_idx_str) = item_id.strip_prefix("item_") {
                    if let Ok(idx) = model_idx_str.parse::<u32>() {
                        for (_id, (pos, body, vel, mesh)) in self.world.query_mut::<(&mut Position, &mut PhysicsBody, &mut Velocity, &StaticMesh)>() {
                            if mesh.model_id == idx {
                                println!("Placing item {} at {:?}", item_id, pos_val);
                                body.active = true;
                                pos.0 = pos_val + glam::Vec3::Y * body.radius;
                                vel.0 = glam::Vec3::ZERO;
                                
                                self.player_state.remove_item(&item_id, 1);
                                for i in 0..5 {
                                    if let Some(hotbar_item) = &self.player_state.hotbar[i] {
                                        if hotbar_item.id == item_id {
                                            self.player_state.hotbar[i] = None;
                                            break;
                                        }
                                    }
                                }
                                self.player_state.equipped_item = None;
                                self.inventory_ui_dirty = true;
                                self.update_equipped_text();
                                break;
                            }
                        }
                    }
                }
            } else {
                println!("Too far to place!");
            }
        }
    }

     
    pub fn handle_right_click(&mut self) {
        if self.inventory_open || self.menu_open {
            return;
        }

        if !self.firearm_system.equipped_is_firearm(&self.player_state, &self.model_paths, &self.dragged_item) {
            self.handle_game_click();
        }
    }


}

fn ray_hit_aabb(
    origin: glam::Vec3,
    direction: glam::Vec3,
    aabb: &Aabb,
    max_dist: f32,
) -> Option<(f32, glam::Vec3)> {
    let inv_dir = glam::Vec3::new(
        1.0 / direction.x,
        1.0 / direction.y,
        1.0 / direction.z,
    );
    let t1 = (aabb.min - origin) * inv_dir;
    let t2 = (aabb.max - origin) * inv_dir;
    let tmin = t1.min(t2);
    let tmax = t1.max(t2);

    let t_enter = tmin.max_element();
    let t_exit = tmax.min_element();
    if t_exit < t_enter || t_exit < 0.0 {
        return None;
    }

    let dist = t_enter.max(0.0);
    if dist > max_dist {
        return None;
    }
    let hit = origin + direction * dist;
    let mut normal = glam::Vec3::ZERO;
    let mut best = f32::MAX;

    let dx_min = (hit.x - aabb.min.x).abs();
    if dx_min < best {
        best = dx_min;
        normal = glam::Vec3::new(-1.0, 0.0, 0.0);
    }
    let dx_max = (hit.x - aabb.max.x).abs();
    if dx_max < best {
        best = dx_max;
        normal = glam::Vec3::new(1.0, 0.0, 0.0);
    }
    let dy_min = (hit.y - aabb.min.y).abs();
    if dy_min < best {
        best = dy_min;
        normal = glam::Vec3::new(0.0, -1.0, 0.0);
    }
    let dy_max = (hit.y - aabb.max.y).abs();
    if dy_max < best {
        best = dy_max;
        normal = glam::Vec3::new(0.0, 1.0, 0.0);
    }
    let dz_min = (hit.z - aabb.min.z).abs();
    if dz_min < best {
        best = dz_min;
        normal = glam::Vec3::new(0.0, 0.0, -1.0);
    }
    let dz_max = (hit.z - aabb.max.z).abs();
    if dz_max < best {
        normal = glam::Vec3::new(0.0, 0.0, 1.0);
    }

    Some((dist, normal))
}
