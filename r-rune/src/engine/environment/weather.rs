use glam::Vec3;

#[derive(Clone, Debug)]
pub struct Sun {
    pub position: Vec3,
    pub direction: Vec3,
    pub color: [f32; 3],
    pub intensity: f32,
}

#[derive(Clone, Debug)]
pub struct Weather {
    pub sun: Sun,
    pub sky_color: [f32; 4],
}

pub fn load_weather_scene() -> Weather {
    let sun = default_sun();
    let sky_color = derive_sky_color(&sun);
    Weather { sun, sky_color }
}

fn default_sun() -> Sun {
     
    let position = Vec3::new(-30.0, 90.0, 0.0);
    let target = Vec3::ZERO;
    let direction = (target - position).normalize_or_zero();

    Sun {
        position,
        direction,
        color: [1.0, 0.97, 0.9],
        intensity: 40.0,
    }
}

fn derive_sky_color(sun: &Sun) -> [f32; 4] {
    let base_sky = Vec3::new(0.34, 0.54, 0.92);
     
    let height_factor = (sun.position.y / 100.0).clamp(0.0, 1.0);
    
    let night_sky = Vec3::new(0.05, 0.05, 0.1);
    let day_sky = base_sky;
    
    let current_base = night_sky.lerp(day_sky, height_factor);

    let tint = Vec3::from_array(sun.color) * (sun.intensity * 0.01);
    let blended = (current_base + tint).min(Vec3::splat(1.0));
    [blended.x, blended.y, blended.z, 1.0]
}

impl Weather {
    pub fn update(&mut self, time: f32) {
         
         
         
        
        let normalized_time = time % 24.0;
        let angle;
        
        if normalized_time >= 6.0 && normalized_time < 20.0 {
            let t = (normalized_time - 6.0) / 14.0;
            angle = t * std::f32::consts::PI;
        } else {
            let t = if normalized_time >= 20.0 {
                (normalized_time - 20.0) / 10.0
            } else {
                (normalized_time + 4.0) / 10.0
            };
            angle = std::f32::consts::PI + t * std::f32::consts::PI;
        }

        let radius = 100.0;
         
         
        
        let x = -angle.cos() * radius;
        let y = angle.sin() * radius;
        let z = 0.0;  

        self.sun.position = Vec3::new(x, y, z);
        self.sun.direction = (Vec3::ZERO - self.sun.position).normalize_or_zero();
        
         
        if y > 0.0 {
            self.sun.intensity = 40.0 * (y / radius).clamp(0.1, 1.0);
            
             
            if y < 30.0 {
                self.sun.color = [1.0, 0.5, 0.2];  
            } else {
                self.sun.color = [1.0, 0.97, 0.9];
            }
        } else {
            self.sun.intensity = 0.0;
        }

        self.sky_color = derive_sky_color(&self.sun);
    }
}
