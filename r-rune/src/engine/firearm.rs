use std::collections::HashSet;
use glam::{Vec3, Quat};
use crate::engine::renderer::Renderer;
use crate::engine::components::{Position, PhysicsBody, StaticMesh};
use crate::vulkan::vk_meshlets::{Aabb, ColliderType};
use crate::engine::player::PlayerState;


const BULLET_SPEED: f32 = 45.0;
const BULLET_LIFETIME: f32 = 2.0;

#[derive(Clone, Copy, Debug)]
pub struct Bullet {
    pub model_idx: usize,
    pub position: Vec3,
    pub velocity: Vec3,
    pub life: f32,
    pub active: bool,
}

pub struct FirearmSystem {
    pub firearm_models: HashSet<usize>,
    pub bullet_hole_models: Vec<usize>,
    pub next_bullet_hole: usize,
    pub bullet_models: Vec<usize>,
    pub bullets: Vec<Bullet>,
    pub next_bullet: usize,
    pub weapon_sway: glam::Vec2,
}

impl FirearmSystem {
    pub fn new() -> Self {
        Self {
            firearm_models: HashSet::new(),
            bullet_hole_models: Vec::new(),
            next_bullet_hole: 0,
            bullet_models: Vec::new(),
            bullets: Vec::new(),
            next_bullet: 0,
            weapon_sway: glam::Vec2::ZERO,
        }
    }
    
    /// Calculate hand offset with weapon sway applied.
    /// Returns the offset from eye position for the weapon.
    pub fn calculate_hand_offset(
        &mut self,
        forward: Vec3,
        right: Vec3,
        up: Vec3,
        player_velocity: Vec3,
        mouse_delta: glam::Vec2,
        camera_sway_scale: f32,
        movement_sway_scale: f32,
        sway_speed: f32,
        sway_limit: f32,
        dt: f32,
    ) -> Vec3 {
        // Sway from camera rotation
        let camera_sway_target = glam::Vec2::new(
            -mouse_delta.x * camera_sway_scale,
            -mouse_delta.y * camera_sway_scale,
        );
        
        // Sway from physical movement (half intensity as requested)
        let movement_sway_target = glam::Vec2::new(
            -player_velocity.dot(right) * movement_sway_scale,
            0.0,
        ) * 0.5;
        
        let target_sway = camera_sway_target + movement_sway_target;
        
        // Smooth interpolation
        self.weapon_sway = self.weapon_sway.lerp(target_sway, (sway_speed * dt).min(1.0));
        
        // Clamp sway to limits
        self.weapon_sway = self.weapon_sway.clamp(
            glam::Vec2::splat(-sway_limit),
            glam::Vec2::splat(sway_limit),
        );
        
        // Base hand offset + sway
        forward * 0.5 + right * (0.2 + self.weapon_sway.x) + up * (-0.2 + self.weapon_sway.y)
    }

    pub fn clear(&mut self) {
        self.firearm_models.clear();
        self.bullet_hole_models.clear();
        self.next_bullet_hole = 0;
        self.bullet_models.clear();
        self.bullets.clear();
        self.next_bullet = 0;
    }

    pub fn register_model(&mut self, idx: usize, is_firearm: bool, is_bullet: bool, path: &str) {
        if is_firearm {
            self.firearm_models.insert(idx);
        }
        if is_bullet {
            self.bullet_models.push(idx);
        }
        if path.contains("bullet_hole") {
            self.bullet_hole_models.push(idx);
        }
    }

    pub fn init_bullets(&mut self) {
         self.bullets = self.bullet_models
            .iter()
            .copied()
            .map(|model_idx| Bullet {
                model_idx,
                position: Vec3::ZERO,
                velocity: Vec3::ZERO,
                life: 0.0,
                active: false,
            })
            .collect::<Vec<_>>();
    }

    pub fn equipped_is_firearm(
        &self,
        player_state: &PlayerState,
        model_paths: &std::collections::HashMap<usize, String>,
        dragged_item: &Option<crate::engine::player::InventoryItem>
    ) -> bool {
        let Some(item_id) = player_state.equipped_item.as_deref() else {
            return false;
        };
        
        if let Some(is_firearm) = self.item_firearm(item_id, player_state, dragged_item) {
            if is_firearm {
                return true;
            }
        }
        
        if let Some(idx) = item_id
            .strip_prefix("item_")
            .and_then(|idx| idx.parse::<usize>().ok())
        {
            if self.firearm_models.contains(&idx) {
                return true;
            }
            if let Some(path) = model_paths.get(&idx) {
                if path.contains("ak47") {
                    return true;
                }
            }
        }
        false
    }

    fn item_firearm(
        &self, 
        item_id: &str, 
        player_state: &PlayerState,
        dragged_item: &Option<crate::engine::player::InventoryItem>
    ) -> Option<bool> {
        if let Some(item) = player_state.inventory.iter().find(|i| i.id == item_id) {
            return Some(item.firearm);
        }
        for item_opt in &player_state.hotbar {
            if let Some(item) = item_opt {
                if item.id == item_id {
                    return Some(item.firearm);
                }
            }
        }
        if let Some(item) = dragged_item {
            if item.id == item_id {
                return Some(item.firearm);
            }
        }
        None
    }

    pub fn fire_bullet(
        &mut self,
        renderer: &mut Renderer,
        player_state: &PlayerState,
        model_base_positions: &[Vec3],
        world_colliders: &[Aabb],
        collider_to_model: &std::collections::HashMap<usize, usize>,
        world: &mut hecs::World,
        eye: Vec3,
        forward: Vec3,
        right: Vec3,
        up: Vec3,
    ) {
        let ray_dir = forward.normalize_or_zero();

        if self.spawn_bullet(renderer, model_base_positions, eye, forward, right, up) {
            return;
        }

         
         
         
         
         
         
         
         
         
         

        let max_dist = 50.0;
        let equipped_model_idx = player_state
            .equipped_item
            .as_deref()
            .and_then(|item_id| item_id.strip_prefix("item_"))
            .and_then(|idx| idx.parse::<usize>().ok());

        let mut best_dist = max_dist;
        let mut hit_point = None;
        let mut hit_normal = None;

        for (i, aabb) in world_colliders.iter().enumerate() {
            if let Some((dist, normal)) = ray_hit_aabb(eye, ray_dir, aabb, max_dist) {
                if dist < best_dist {
                    if let Some(model_idx) = collider_to_model.get(&i) {
                        if Some(*model_idx) == equipped_model_idx {
                            continue;
                        }
                    }
                    best_dist = dist;
                    hit_point = Some(eye + ray_dir * dist);
                    hit_normal = Some(normal);
                }
            }
        }

       for (_id, (pos, body, mesh)) in world.query::<(&Position, &PhysicsBody, &StaticMesh)>().iter() {
           if !body.active {
                continue;
            }
            if Some(mesh.model_id as usize) == equipped_model_idx {
                continue;
            }
            let extents = Vec3::splat(body.radius);
            let aabb = Aabb::new(pos.0 - extents, pos.0 + extents, ColliderType::Solid);
            if let Some((dist, normal)) = ray_hit_aabb(eye, ray_dir, &aabb, max_dist) {
                if dist < best_dist {
                    best_dist = dist;
                    hit_point = Some(eye + ray_dir * dist);
                    hit_normal = Some(normal);
                }
            }
        }

        if let (Some(point), Some(normal)) = (hit_point, hit_normal) {
            self.spawn_bullet_hole(renderer, model_base_positions, point, normal, ray_dir);
        }
    }

    fn spawn_bullet(
        &mut self,
        renderer: &mut Renderer,
        model_base_positions: &[Vec3],
        eye: Vec3,
        forward: Vec3,
        right: Vec3,
        up: Vec3,
    ) -> bool {
        if self.bullets.is_empty() {
            return false;
        }

        let idx = self.next_bullet;
        self.next_bullet = (self.next_bullet + 1) % self.bullets.len();
        let bullet = &mut self.bullets[idx];

        let muzzle = eye + forward * 0.6 + right * 0.2 + up * -0.2;
        bullet.position = muzzle;
        bullet.velocity = forward.normalize_or_zero() * BULLET_SPEED;
        bullet.life = BULLET_LIFETIME;
        bullet.active = true;

        let base_pos = model_base_positions
            .get(bullet.model_idx)
            .copied()
            .unwrap_or(Vec3::ZERO);
        let offset = bullet.position - base_pos;
        unsafe {
            renderer.update_model_offsets(&[(bullet.model_idx, offset)]);
        }
        true
    }

    pub fn update_bullets(
        &mut self,
        dt: f32,
        renderer: &mut Renderer,
        world_colliders: &[Aabb],
        world: &mut hecs::World,
        model_base_positions: &[Vec3],
    ) {
        if dt <= 0.0 || self.bullets.is_empty() {
            return;
        }

        let mut updates: Vec<(usize, Vec3)> = Vec::new();
        let mut hits: Vec<(Vec3, Vec3, Vec3)> = Vec::new();

        for bullet in &mut self.bullets {
            if !bullet.active {
                continue;
            }

            let step = bullet.velocity * dt;
            let step_len = step.length();
            let dir = bullet.velocity.normalize_or_zero();
            let mut hit_point = None;
            let mut hit_normal = None;
            let mut best_dist = step_len;

            if step_len > f32::EPSILON {
                for aabb in world_colliders {
                    if let Some((dist, normal)) = ray_hit_aabb(bullet.position, dir, aabb, step_len)
                    {
                        if dist < best_dist {
                            best_dist = dist;
                            hit_point = Some(bullet.position + dir * dist);
                            hit_normal = Some(normal);
                        }
                    }
                }

                for (_id, (pos, body, _mesh)) in world.query::<(&Position, &PhysicsBody, &StaticMesh)>().iter() {
                    if !body.active {
                        continue;
                    }
                    let extents = Vec3::splat(body.radius);
                    let aabb = Aabb::new(pos.0 - extents, pos.0 + extents, ColliderType::Solid);
                    if let Some((dist, normal)) =
                        ray_hit_aabb(bullet.position, dir, &aabb, step_len)
                    {
                        if dist < best_dist {
                            best_dist = dist;
                            hit_point = Some(bullet.position + dir * dist);
                            hit_normal = Some(normal);
                        }
                    }
                }
            }

            if let (Some(point), Some(normal)) = (hit_point, hit_normal) {
                bullet.active = false;
                hits.push((point, normal, dir));
                updates.push((bullet.model_idx, Vec3::ZERO));
                continue;
            }

            bullet.position += step;
            bullet.life -= dt;
            if bullet.life <= 0.0 {
                bullet.active = false;
                updates.push((bullet.model_idx, Vec3::ZERO));
                continue;
            }

            let base_pos = model_base_positions
                .get(bullet.model_idx)
                .copied()
                .unwrap_or(Vec3::ZERO);
            updates.push((bullet.model_idx, bullet.position - base_pos));
        }

        if !updates.is_empty() {
            unsafe {
                renderer.update_model_offsets(&updates);
            }
        }

        for (point, normal, dir) in hits {
            self.spawn_bullet_hole(renderer, model_base_positions, point, normal, dir);
        }
    }

    fn spawn_bullet_hole(
        &mut self, 
        renderer: &mut Renderer,
        model_base_positions: &[Vec3],
        point: Vec3, 
        normal: Vec3, 
        ray_dir: Vec3
    ) {
        if self.bullet_hole_models.is_empty() {
            return;
        }

        let model_idx = self.bullet_hole_models[self.next_bullet_hole];
        self.next_bullet_hole = (self.next_bullet_hole + 1) % self.bullet_hole_models.len();

        let base_pos = model_base_positions
            .get(model_idx)
            .copied()
            .unwrap_or(Vec3::ZERO);
        let resolved_normal = if normal.length_squared() > 1e-6 {
            normal.normalize_or_zero()
        } else {
            (-ray_dir).normalize_or_zero()
        };
        let decal_offset = resolved_normal * 0.02;
        let world_pos = point + decal_offset;
        let offset = world_pos - base_pos;
        let rotation = align_local_axes_to_view(
            resolved_normal,
            Vec3::Y,
            Vec3::Z,
            Vec3::Y,
        );

        unsafe {
            renderer.update_model_offsets(&[(model_idx, offset)]);
            renderer.update_model_rotations(&[(model_idx, rotation)]);
        }
    }
}

 

fn ray_hit_aabb(
    origin: Vec3,
    direction: Vec3,
    aabb: &Aabb,
    max_dist: f32,
) -> Option<(f32, Vec3)> {
    let inv_dir = Vec3::new(
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
    
     
    let mut best = f32::MAX;
    let mut normal = Vec3::ZERO;
    
    let dx_min = (hit.x - aabb.min.x).abs();
    if dx_min < best { best = dx_min; normal = Vec3::new(-1.0, 0.0, 0.0); }
    let dx_max = (hit.x - aabb.max.x).abs();
    if dx_max < best { best = dx_max; normal = Vec3::new(1.0, 0.0, 0.0); }
    let dy_min = (hit.y - aabb.min.y).abs();
    if dy_min < best { best = dy_min; normal = Vec3::new(0.0, -1.0, 0.0); }
    let dy_max = (hit.y - aabb.max.y).abs();
    if dy_max < best { best = dy_max; normal = Vec3::new(0.0, 1.0, 0.0); }
    let dz_min = (hit.z - aabb.min.z).abs();
    if dz_min < best { best = dz_min; normal = Vec3::new(0.0, 0.0, -1.0); }
    let dz_max = (hit.z - aabb.max.z).abs();
    if dz_max < best { normal = Vec3::new(0.0, 0.0, 1.0); }

    Some((dist, normal))
}


fn align_local_axes_to_view(
    world_forward: Vec3,
    world_up: Vec3,
    local_forward: Vec3,
    local_up: Vec3,
) -> Quat {
    let world_forward = world_forward.normalize_or_zero();
    if world_forward.length_squared() < 1e-6 {
        return Quat::IDENTITY;
    }

    let mut world_up = world_up.normalize_or_zero();
    if world_up.length_squared() < 1e-6 {
        world_up = Vec3::Y;
    }
    let mut world_right = world_forward.cross(world_up);
    if world_right.length_squared() < 1e-6 {
        world_up = Vec3::Z;
        world_right = world_forward.cross(world_up);
    }
    world_right = world_right.normalize_or_zero();
    world_up = world_right.cross(world_forward).normalize_or_zero();

    let local_forward = local_forward.normalize_or_zero();
    let mut local_up = local_up.normalize_or_zero();
    let mut local_right = local_forward.cross(local_up);
    if local_right.length_squared() < 1e-6 {
        return Quat::IDENTITY;
    }
    local_right = local_right.normalize_or_zero();
    local_up = local_right.cross(local_forward).normalize_or_zero();

    let local_basis = glam::Mat3::from_cols(local_right, local_up, local_forward);
    let world_basis = glam::Mat3::from_cols(world_right, world_up, world_forward);
    Quat::from_mat3(&(world_basis * local_basis.transpose()))
}
