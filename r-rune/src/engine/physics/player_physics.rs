use glam::Vec3;

use crate::engine::{config::PhysicsConfig, player::Player, InputState};
use crate::engine::physics::capsule::{Capsule, capsule_aabb_penetration};
use crate::vulkan::vk_meshlets::{Aabb, ColliderType, FLOOR_Y_OFFSET};

/// Perform one physics step for the player using capsule collision.
pub fn step(player: &mut Player, input: &InputState, cfg: &PhysicsConfig, colliders: &[Aabb], dt: f32) {
    // Noclip mode: free movement, no collision
    if player.noclip {
        let cam_forward = Vec3::new(
            player.pitch.cos() * player.yaw.cos(),
            (-player.pitch).sin(),
            player.pitch.cos() * player.yaw.sin(),
        ).normalize();
        
        let cam_right = cam_forward.cross(Vec3::Y).normalize_or_zero();
        
        let mut wish_dir = Vec3::ZERO;
        if input.forward { wish_dir += cam_forward; }
        if input.backward { wish_dir -= cam_forward; }
        if input.left { wish_dir -= cam_right; }
        if input.right { wish_dir += cam_right; }
        if input.jump { wish_dir += Vec3::Y; }
        if input.sprint { wish_dir -= Vec3::Y; }
        
        if wish_dir.length_squared() > 0.0 {
            wish_dir = wish_dir.normalize();
        }
        
        let speed = cfg.base_speed * 2.0 * if input.sprint { 2.0 } else { 1.0 };
        player.position += wish_dir * speed * dt;
        player.velocity = Vec3::ZERO;
        player.on_ground = false;
        return;
    }

    // Calculate movement direction from input
    let (forward, right, _) = player.view_axes();
    let mut wish_dir = Vec3::ZERO;
    if input.forward { wish_dir += forward; }
    if input.backward { wish_dir -= forward; }
    if input.left { wish_dir -= right; }
    if input.right { wish_dir += right; }
    wish_dir.y = 0.0;
    if wish_dir.length_squared() > 0.0 {
        wish_dir = wish_dir.normalize();
    }

    // Calculate target speed and acceleration
    let target_speed = cfg.base_speed * if input.sprint { cfg.sprint_multiplier } else { 1.0 };
    let accel = if player.on_ground { cfg.ground_accel } else { cfg.air_control };

    // Apply friction when grounded and not moving
    if player.on_ground && wish_dir.length_squared() < f32::EPSILON {
        let friction_scale = (1.0 - cfg.friction * dt).clamp(0.0, 1.0);
        player.velocity.x *= friction_scale;
        player.velocity.z *= friction_scale;
    }

    // Accelerate toward desired velocity
    let desired = wish_dir * target_speed;
    let flat_velocity = Vec3::new(player.velocity.x, 0.0, player.velocity.z);
    let delta = desired - flat_velocity;
    let max_delta = accel * dt;
    let delta_clamped = if delta.length() > max_delta {
        delta.normalize() * max_delta
    } else {
        delta
    };
    player.velocity.x += delta_clamped.x;
    player.velocity.z += delta_clamped.z;

    // Jump
    if input.jump && player.on_ground {
        player.velocity.y = cfg.jump_velocity;
        player.on_ground = false;
    }
    
    // Apply gravity
    player.velocity.y -= cfg.gravity * dt;

    // Calculate next position
    let mut next_position = player.position + player.velocity * dt;
    
    // Build the player capsule at the next position
    let capsule = Capsule::from_feet(next_position, cfg.capsule_height, cfg.capsule_radius);
    
    let mut grounded_on_object = false;
    let max_step_height = 0.5;
    
    // Collision resolution with multiple iterations for stability
    const MAX_ITERATIONS: usize = 4;
    for _ in 0..MAX_ITERATIONS {
        let current_capsule = Capsule::from_feet(next_position, cfg.capsule_height, cfg.capsule_radius);
        let mut total_push = Vec3::ZERO;
        let mut collision_count = 0;
        
        for aabb in colliders {
            match aabb.collider_type {
                ColliderType::Ramp => {
                    // Handle ramp collision
                    let in_xz = next_position.x > aabb.min.x - cfg.capsule_radius 
                             && next_position.x < aabb.max.x + cfg.capsule_radius
                             && next_position.z > aabb.min.z - cfg.capsule_radius 
                             && next_position.z < aabb.max.z + cfg.capsule_radius;
                    
                    if in_xz && next_position.y < aabb.max.y && next_position.y + cfg.capsule_height > aabb.min.y {
                        let ramp_len = aabb.max.z - aabb.min.z;
                        if ramp_len > 0.001 {
                            let relative_z = (next_position.z - aabb.min.z) / ramp_len;
                            let target_y = aabb.min.y + (aabb.max.y - aabb.min.y) * relative_z.clamp(0.0, 1.0);
                            
                            if next_position.y >= aabb.min.y - 0.1 && next_position.y <= target_y + 0.5 {
                                next_position.y = target_y;
                                grounded_on_object = true;
                                player.velocity.y = 0.0;
                            }
                        }
                    }
                }
                _ => {
                    // Standard capsule-AABB collision
                    if let Some((push_dir, depth)) = capsule_aabb_penetration(&current_capsule, aabb) {
                        // Check if this is a step-up candidate
                        let obstacle_top = aabb.max.y;
                        let player_bottom = next_position.y;
                        
                        if obstacle_top > player_bottom 
                           && obstacle_top - player_bottom <= max_step_height 
                           && push_dir.y.abs() < 0.5  // Not a ceiling collision
                        {
                            // Step up
                            next_position.y = obstacle_top;
                            grounded_on_object = true;
                            player.velocity.y = 0.0;
                        } else {
                            // Normal collision resolution
                            total_push += push_dir * (depth + 0.001);
                            collision_count += 1;
                            
                            // If pushed mostly upward, we're grounded
                            if push_dir.y > 0.7 {
                                grounded_on_object = true;
                                player.velocity.y = 0.0;
                            }
                            
                            // Cancel velocity in push direction
                            let vel_into_wall = player.velocity.dot(push_dir);
                            if vel_into_wall < 0.0 {
                                player.velocity -= push_dir * vel_into_wall;
                            }
                        }
                    }
                }
            }
        }
        
        if collision_count == 0 {
            break;
        }
        
        // Apply the total push
        next_position += total_push;
    }
    
    player.position = next_position;

    // Floor collision
    let floor_min_y = FLOOR_Y_OFFSET + 0.1;
    if player.position.y < floor_min_y {
        player.position.y = floor_min_y;
        player.velocity.y = 0.0;
        player.on_ground = true;
    } else {
        player.on_ground = grounded_on_object;
    }
}
