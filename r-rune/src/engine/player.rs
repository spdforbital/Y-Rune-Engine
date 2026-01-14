use glam::Vec3;

pub struct Player {
    pub position: Vec3,  
    pub velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
    pub eye_height: f32,
    pub noclip: bool,
}

impl Player {
    pub fn new(feet_position: Vec3) -> Self {
        Self {
            position: feet_position,
            velocity: Vec3::ZERO,
            yaw: 0.0,    
            pitch: 0.0,  
            on_ground: true,
            eye_height: 1.6,
            noclip: false,
        }
    }

    pub fn eye_position(&self) -> Vec3 {
        self.position + Vec3::Y * self.eye_height
    }

    pub fn add_look(&mut self, dx: f32, dy: f32, sensitivity: f32) {
        self.yaw += dx * sensitivity;
        self.pitch += dy * sensitivity;
        let limit = 1.57f32;  
        if self.pitch > limit {
            self.pitch = limit;
        }
        if self.pitch < -limit {
            self.pitch = -limit;
        }
    }

    pub fn view_axes(&self) -> (Vec3, Vec3, Vec3) {
        let forward = Vec3::new(
            self.pitch.cos() * self.yaw.cos(),
            (-self.pitch).sin(),
            self.pitch.cos() * self.yaw.sin(),
        )
        .normalize();
        let right = forward.cross(Vec3::Y).normalize_or_zero();
        let up = right.cross(forward).normalize_or_zero();
        (forward, right, up)
    }
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: String,
    pub name: String,
    pub count: u32,
    pub icon: Option<String>,
    #[serde(default)]
    pub firearm: bool,
}

#[derive(Clone, Debug)]
pub struct PlayerState {
    pub health: f32,
    pub max_health: f32,
    pub inventory: Vec<InventoryItem>,
    pub equipped_item: Option<String>,
    pub hotbar: [Option<InventoryItem>; 5],
    pub active_hotbar_slot: usize,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            health: 100.0,
            max_health: 100.0,
            inventory: Vec::new(),
            equipped_item: None,
            hotbar: [const { None }; 5],
            active_hotbar_slot: 0,
        }
    }
}

impl PlayerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_item(&mut self, item: InventoryItem) {
        // 1. Try to stack in hotbar
        for slot in self.hotbar.iter_mut() {
            if let Some(existing) = slot {
                if existing.id == item.id {
                    existing.count += item.count;
                    existing.firearm |= item.firearm;
                    return;
                }
            }
        }

        // 2. Try to stack in inventory
        if let Some(existing) = self.inventory.iter_mut().find(|i| i.id == item.id) {
            existing.count += item.count;
            existing.firearm |= item.firearm;
            return;
        }

        // 3. Try into first free hotbar slot
        for (i, slot) in self.hotbar.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(item.clone());
                if i == self.active_hotbar_slot {
                    self.equipped_item = Some(item.id.clone());
                }
                return;
            }
        }

        // 4. Fallback to inventory
        self.inventory.push(item);
    }

    pub fn remove_item(&mut self, id: &str, count: u32) -> bool {
        if let Some(pos) = self.inventory.iter().position(|i| i.id == id) {
            if self.inventory[pos].count >= count {
                self.inventory[pos].count -= count;
                if self.inventory[pos].count == 0 {
                    self.inventory.remove(pos);
                }
                return true;
            }
        }
        false
    }
}
