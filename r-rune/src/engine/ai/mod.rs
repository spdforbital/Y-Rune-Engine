

use glam::Vec3;

 
#[derive(Clone, Debug)]
pub struct AIState {
    pub position: Vec3,
    pub forward: Vec3,
}

impl AIState {
    pub fn new(position: Vec3, forward: Vec3) -> Self {
        Self {
            position,
            forward: forward.normalize_or_zero(),
        }
    }
}

use std::fmt;

pub struct WalkTask {
    pub target: Vec3,
    pub speed: f32,
    pub reach_radius: f32,
    pub started: bool,
    pub on_start: Option<Box<dyn FnMut(&mut AIState) + Send + Sync>>,
    pub on_reach: Option<Box<dyn FnMut(&mut AIState) + Send + Sync>>,
}

impl fmt::Debug for WalkTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WalkTask")
         .field("target", &self.target)
         .field("speed", &self.speed)
         .field("reach_radius", &self.reach_radius)
         .field("started", &self.started)
         .finish()
    }
}

impl WalkTask {
    pub fn new(
        target: Vec3,
        speed: f32,
        reach_radius: f32,
        on_start: Option<Box<dyn FnMut(&mut AIState) + Send + Sync>>,
        on_reach: Option<Box<dyn FnMut(&mut AIState) + Send + Sync>>,
    ) -> Self {
        Self {
            target,
            speed,
            reach_radius,
            started: false,
            on_start,
            on_reach,
        }
    }

    /// Step the walk task toward its target; returns true when completed.
    pub fn step(&mut self, ai: &mut AIState, dt: f32) -> bool {
        if !self.started {
            if let Some(cb) = self.on_start.as_mut() {
                cb(ai);
            }
            self.started = true;
        }

        let to_target = self.target - ai.position;
        let dist = to_target.length();
        if dist <= self.reach_radius {
            if let Some(cb) = self.on_reach.as_mut() {
                cb(ai);
            }
            return true;
        }

        let dir = to_target / dist.max(1e-5);
        ai.position += dir * self.speed * dt;
        ai.forward = dir;
        false
    }
}

/// Check if a target is inside the AI's field of view and range; invoke callback when seen.
pub fn check_fov_and_notify(
    ai_pos: Vec3,
    ai_forward: Vec3,
    target_pos: Vec3,
    fov_degrees: f32,
    max_distance: f32,
    on_see: &mut dyn FnMut(),
) -> bool {
    let to_target = target_pos - ai_pos;
    let dist = to_target.length();
    if dist > max_distance {
        return false;
    }

    let dir = ai_forward.normalize_or_zero();
    let dir_to_target = to_target / dist.max(1e-5);
    let cos_angle = dir.dot(dir_to_target);
    let limit = (fov_degrees.to_radians() * 0.5).cos();
    if cos_angle >= limit {
        on_see();
        return true;
    }
    false
}
