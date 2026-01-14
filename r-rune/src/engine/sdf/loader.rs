use glam::{Vec3, Mat4};

#[derive(Debug, Clone)]
pub enum SdfOp {
     
     
    Sphere { radius: f32, center: Vec3, color: Vec3 },
     
    Box { half_extents: Vec3, center: Vec3, color: Vec3 },
     
    Cylinder { height: f32, radius: f32, center: Vec3, color: Vec3 },
     
    Torus { thickness: f32, radius: f32, center: Vec3, color: Vec3 },  

     
    Union,       
    Subtract,    
    Intersect,   
    SmoothUnion { k: f32 },
    SmoothSubtract { k: f32 },
    SmoothIntersect { k: f32 },
}

 
 
 
 
 
 
 
 

pub const OP_SPHERE: f32 = 1.0;
pub const OP_BOX: f32 = 2.0;
pub const OP_CYLINDER: f32 = 3.0;
pub const OP_TORUS: f32 = 4.0;
pub const OP_UNION: f32 = 50.0;
pub const OP_SUB: f32 = 51.0;
pub const OP_INTERSECT: f32 = 52.0;
pub const OP_SMOOTH_UNION: f32 = 53.0;
pub const OP_SMOOTH_SUB: f32 = 54.0;
pub const OP_SMOOTH_INTERSECT: f32 = 55.0;

#[derive(Default)]
pub struct SdfModel {
    pub data: Vec<f32>,
}

impl SdfModel {
    pub fn new() -> Self {
        Self::default()
    }

    fn push_color(&mut self, color: Vec3) {
        self.data.push(color.x);
        self.data.push(color.y);
        self.data.push(color.z);
    }
    
    fn push_vec3(&mut self, v: Vec3) {
        self.data.push(v.x);
        self.data.push(v.y);
        self.data.push(v.z);
    }

    pub fn sphere(mut self, center: Vec3, radius: f32, color: Vec3) -> Self {
        self.data.push(OP_SPHERE);
        self.push_vec3(center);
        self.data.push(radius);
        self.push_color(color);
        self  
    }

    pub fn box_shape(mut self, center: Vec3, half_extents: Vec3, color: Vec3) -> Self {
        self.data.push(OP_BOX);
        self.push_vec3(center);
        self.push_vec3(half_extents);
        self.push_color(color);
        self  
    }

    pub fn cylinder(mut self, center: Vec3, height: f32, radius: f32, color: Vec3) -> Self {
        self.data.push(OP_CYLINDER);
        self.push_vec3(center);
        self.data.push(height);
        self.data.push(radius);
        self.push_color(color);
        self  
    }

    pub fn torus(mut self, center: Vec3, radius: f32, thickness: f32, color: Vec3) -> Self {
        self.data.push(OP_TORUS);
        self.push_vec3(center);
        self.data.push(radius);
        self.data.push(thickness);
        self.push_color(color);
        self  
    }

    pub fn union(mut self) -> Self {
        self.data.push(OP_UNION);
        self
    }

    pub fn subtract(mut self) -> Self {
         
         
        self.data.push(OP_SUB);
        self
    }
    
    pub fn intersect(mut self) -> Self {
        self.data.push(OP_INTERSECT);
        self
    }

    pub fn smooth_union(mut self, k: f32) -> Self {
        self.data.push(OP_SMOOTH_UNION);
        self.data.push(k);
        self
    }
    
    pub fn smooth_subtract(mut self, k: f32) -> Self {
        self.data.push(OP_SMOOTH_SUB);
        self.data.push(k);
        self
    }
}
