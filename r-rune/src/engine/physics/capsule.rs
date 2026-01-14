use glam::Vec3;
use crate::vulkan::vk_meshlets::Aabb;

/// A capsule collider represented as a vertical line segment with a radius.
/// The capsule extends from `bottom` to `top` with hemispheres at each end.
#[derive(Clone, Copy, Debug)]
pub struct Capsule {
    pub bottom: Vec3,
    pub top: Vec3,
    pub radius: f32,
}

impl Capsule {
    /// Create a capsule from the player's feet position.
    /// The capsule extends from feet + radius (bottom sphere center) to feet + height - radius (top sphere center).
    pub fn from_feet(feet: Vec3, height: f32, radius: f32) -> Self {
        Self {
            bottom: feet + Vec3::Y * radius,
            top: feet + Vec3::Y * (height - radius),
            radius,
        }
    }

    /// Get the total height of the capsule (including hemispheres)
    pub fn height(&self) -> f32 {
        (self.top.y - self.bottom.y) + 2.0 * self.radius
    }
}

/// Find the closest point on line segment AB to point P.
pub fn closest_point_on_segment(p: Vec3, a: Vec3, b: Vec3) -> Vec3 {
    let ab = b - a;
    let len_sq = ab.length_squared();
    
    if len_sq < f32::EPSILON {
        return a; // Degenerate segment
    }
    
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    a + ab * t
}

/// Compute the shortest distance from a point to the capsule surface.
/// Returns negative if the point is inside the capsule.
pub fn point_capsule_distance(point: Vec3, capsule: &Capsule) -> f32 {
    let closest = closest_point_on_segment(point, capsule.bottom, capsule.top);
    (point - closest).length() - capsule.radius
}

/// Check if a capsule intersects an AABB.
/// Returns the penetration vector (direction to push capsule out) and depth if intersecting.
pub fn capsule_aabb_penetration(capsule: &Capsule, aabb: &Aabb) -> Option<(Vec3, f32)> {
    // Find the closest point on the capsule's spine to the AABB
    // Then check if that point (expanded by radius) is inside the AABB
    
    // First, clamp the capsule spine to the AABB's Y range to find the relevant segment
    let aabb_center = (aabb.min + aabb.max) * 0.5;
    let aabb_half = (aabb.max - aabb.min) * 0.5;
    
    // Find closest point on capsule spine to AABB center
    let spine_closest = closest_point_on_segment(aabb_center, capsule.bottom, capsule.top);
    
    // Find closest point on AABB to this spine point
    let clamped = Vec3::new(
        spine_closest.x.clamp(aabb.min.x, aabb.max.x),
        spine_closest.y.clamp(aabb.min.y, aabb.max.y),
        spine_closest.z.clamp(aabb.min.z, aabb.max.z),
    );
    
    // Now find the closest point on the spine to this clamped point
    let spine_point = closest_point_on_segment(clamped, capsule.bottom, capsule.top);
    
    // Re-clamp to get the true closest point on AABB
    let aabb_point = Vec3::new(
        spine_point.x.clamp(aabb.min.x, aabb.max.x),
        spine_point.y.clamp(aabb.min.y, aabb.max.y),
        spine_point.z.clamp(aabb.min.z, aabb.max.z),
    );
    
    // Vector from AABB point to capsule spine
    let delta = spine_point - aabb_point;
    let dist = delta.length();
    
    if dist < capsule.radius {
        // Penetrating
        let depth = capsule.radius - dist;
        
        let push_dir = if dist > f32::EPSILON {
            delta / dist
        } else {
            // Capsule center is inside AABB, push out along shortest axis
            let to_min = spine_point - aabb.min;
            let to_max = aabb.max - spine_point;
            
            let min_x = to_min.x.min(to_max.x);
            let min_y = to_min.y.min(to_max.y);
            let min_z = to_min.z.min(to_max.z);
            
            if min_x <= min_y && min_x <= min_z {
                if to_min.x < to_max.x { -Vec3::X } else { Vec3::X }
            } else if min_y <= min_z {
                if to_min.y < to_max.y { -Vec3::Y } else { Vec3::Y }
            } else {
                if to_min.z < to_max.z { -Vec3::Z } else { Vec3::Z }
            }
        };
        
        Some((push_dir, depth))
    } else {
        None
    }
}

/// Sweep-test: check if moving a capsule by `velocity * dt` would hit an AABB.
/// Returns the time of impact (0.0 to 1.0) and the hit normal if there's a collision.
pub fn capsule_aabb_sweep(
    capsule: &Capsule,
    velocity: Vec3,
    aabb: &Aabb,
    dt: f32,
) -> Option<(f32, Vec3)> {
    // Expand the AABB by the capsule radius and test as a line sweep
    let expanded_aabb = Aabb::new(
        aabb.min - Vec3::splat(capsule.radius),
        aabb.max + Vec3::splat(capsule.radius),
        aabb.collider_type,
    );
    
    // For simplicity, we'll do a discrete check at the end position
    // A full swept capsule test is more complex
    let end_capsule = Capsule {
        bottom: capsule.bottom + velocity * dt,
        top: capsule.top + velocity * dt,
        radius: capsule.radius,
    };
    
    if let Some((push_dir, depth)) = capsule_aabb_penetration(&end_capsule, aabb) {
        // Estimate time of impact based on how much we've penetrated
        let travel_dist = (velocity * dt).length();
        if travel_dist > f32::EPSILON {
            let t = ((travel_dist - depth) / travel_dist).clamp(0.0, 1.0);
            Some((t, push_dir))
        } else {
            Some((0.0, push_dir))
        }
    } else {
        None
    }
}
