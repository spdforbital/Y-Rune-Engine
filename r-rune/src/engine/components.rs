use glam::Vec3;
use crate::engine::ai::{AIState, WalkTask};
use crate::engine::player::Player;
use crate::vulkan::vk_meshlets::{Aabb, ColliderType, FLOOR_Y_OFFSET};

const PLAYER_RADIUS: f32 = 0.3;
const PLAYER_HEIGHT: f32 = 1.7;
const PUSH_STRENGTH: f32 = 2.2;
const ROLL_FRICTION: f32 = 2.5;
const AIR_DAMPING: f32 = 0.25;
const STOP_SPEED: f32 = 0.05;
const MAX_SPEED: f32 = 12.0;
const MAX_STEP: f32 = 1.0 / 60.0;
const PUSH_MARGIN: f32 = 0.08;

#[derive(Debug, Clone, Copy)]
pub struct Position(pub Vec3);

#[derive(Debug, Clone, Copy)]
pub struct Rotation(pub Vec3);  

#[derive(Debug, Clone, Copy)]
pub struct Scale(pub f32);

#[derive(Debug, Clone, Copy)]
pub struct Velocity(pub Vec3);

 
#[derive(Debug, Clone, Copy)]
pub struct StaticMesh {
    pub model_id: u32,
    pub offset: Vec3,  
}

 
#[derive(Debug, Clone)]
pub struct SkinnedMesh {
    pub resource_id: String,
}

 
#[derive(Debug, Clone, Copy)]
pub struct PlayerComponent;

 
#[derive(Debug, Clone)]
pub struct PhysicsBody {
    pub radius: f32,
    pub on_ground: bool,
    pub active: bool,
    pub base_position: Vec3,  
    pub pushable: bool,
}

impl Default for PhysicsBody {
    fn default() -> Self {
        Self {
            radius: 0.5,
            on_ground: false,
            active: true,
            base_position: Vec3::ZERO,
            pushable: false,
        }
    }
}

impl PhysicsBody {
    pub fn step(
        &mut self,
        pos: &mut Position,
        vel: &mut Velocity,
        player: &Player,
        colliders: &[Aabb],
        gravity: f32,
        push_dir: Vec3,
        dt: f32,
    ) {
        if dt <= 0.0 || !self.active {
            return;
        }
        let mut remaining = dt;
        while remaining > 0.0 {
            let step_dt = remaining.min(MAX_STEP);
            self.step_once(pos, vel, player, colliders, gravity, push_dir, step_dt);
            remaining -= step_dt;
        }
    }

    fn step_once(
        &mut self,
        pos: &mut Position,
        vel: &mut Velocity,
        player: &Player,
        colliders: &[Aabb],
        gravity: f32,
        push_dir: Vec3,
        dt: f32,
    ) {
        if dt <= 0.0 {
            return;
        }

        vel.0.y -= gravity * dt;
        pos.0 += vel.0 * dt;

        let mut on_ground = false;
        for aabb in colliders {
            if let Some(normal) = resolve_sphere_aabb(pos, vel, self.radius, aabb) {
                if normal.y > 0.5 {
                    on_ground = true;
                }
            }
        }

        if self.pushable {
            resolve_player_push(pos, vel, self.radius, player, push_dir);
        }

        let mut horizontal = Vec3::new(vel.0.x, 0.0, vel.0.z);
        if on_ground {
            let friction = (1.0 - ROLL_FRICTION * dt).clamp(0.0, 1.0);
            horizontal *= friction;
        } else {
            let damping = (1.0 - AIR_DAMPING * dt).clamp(0.0, 1.0);
            horizontal *= damping;
        }

        if horizontal.length() < STOP_SPEED {
            horizontal = Vec3::ZERO;
        }

        let speed = horizontal.length();
        if speed > MAX_SPEED {
            horizontal = horizontal / speed * MAX_SPEED;
        }

        vel.0.x = horizontal.x;
        vel.0.z = horizontal.z;

        let floor_y = FLOOR_Y_OFFSET + self.radius;
        if pos.0.y < floor_y {
            pos.0.y = floor_y;
            if vel.0.y < 0.0 {
                vel.0.y = 0.0;
            }
            on_ground = true;
        }

        self.on_ground = on_ground;
    }
}

fn resolve_sphere_aabb(pos: &mut Position, vel: &mut Velocity, radius: f32, aabb: &Aabb) -> Option<Vec3> {
    let closest = pos.0.clamp(aabb.min, aabb.max);
    let delta = pos.0 - closest;
    let dist_sq = delta.length_squared();

    if dist_sq >= radius * radius {
        return None;
    }

    let (normal, penetration) = if dist_sq > 1e-6 {
        let dist = dist_sq.sqrt();
        (delta / dist, radius - dist)
    } else {
        let to_min = pos.0 - aabb.min;
        let to_max = aabb.max - pos.0;
        let mut min_dist = to_min.x;
        let mut normal = Vec3::new(-1.0, 0.0, 0.0);

        if to_max.x < min_dist {
            min_dist = to_max.x;
            normal = Vec3::new(1.0, 0.0, 0.0);
        }
        if to_min.y < min_dist {
            min_dist = to_min.y;
            normal = Vec3::new(0.0, -1.0, 0.0);
        }
        if to_max.y < min_dist {
            min_dist = to_max.y;
            normal = Vec3::new(0.0, 1.0, 0.0);
        }
        if to_min.z < min_dist {
            min_dist = to_min.z;
            normal = Vec3::new(0.0, 0.0, -1.0);
        }
        if to_max.z < min_dist {
            min_dist = to_max.z;
            normal = Vec3::new(0.0, 0.0, 1.0);
        }

        (normal, radius + min_dist)
    };

    pos.0 += normal * penetration;
    let vn = vel.0.dot(normal);
    if vn < 0.0 {
        vel.0 -= normal * vn;
    }

    Some(normal)
}

fn resolve_player_push(pos: &mut Position, vel: &mut Velocity, radius: f32, player: &Player, push_dir: Vec3) {
    if player.noclip {
        return;
    }
    if push_dir.length_squared() <= f32::EPSILON {
        return;
    }

    let player_bottom = player.position.y;
    let player_top = player.position.y + PLAYER_HEIGHT;
    if pos.0.y + radius < player_bottom || pos.0.y - radius > player_top {
        return;
    }

    let offset = Vec3::new(
        pos.0.x - player.position.x,
        0.0,
        pos.0.z - player.position.z,
    );
    let dist_sq = offset.length_squared();
    let min_dist = radius + PLAYER_RADIUS;
    let contact_dist = min_dist + PUSH_MARGIN;

    if dist_sq > contact_dist * contact_dist {
        return;
    }

    let dist = dist_sq.sqrt().max(1e-4);
    let normal = if dist > 1e-4 {
        offset / dist
    } else if push_dir.length_squared() > 0.0 {
        push_dir
    } else {
        Vec3::Z
    };

    if dist < min_dist {
        let penetration = min_dist - dist;
        pos.0 += normal * penetration;
    }

    let toward = push_dir.dot(normal);
    if toward > 0.0 {
        vel.0 += normal * (toward * PUSH_STRENGTH);
    }
}

 
#[derive(Debug)]
pub struct AiAgent {
    pub state: AIState,
    pub current_task: Option<WalkTask>,
    pub wander_radius: f32,
    pub speed: f32,
}

 
#[derive(Debug, Clone)]
pub struct Interactable {
    pub action_name: String,
}
