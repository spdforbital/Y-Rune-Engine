use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use gltf::animation::util::ReadOutputs;

#[derive(Clone)]
struct ChannelSampler {
    input: Vec<f32>,
    outputs: Vec<f32>,
    stride: usize,
    path: gltf::animation::Property,
    interpolation: gltf::animation::Interpolation,
}

#[derive(Clone)]
struct AnimationChannel {
    target_node: usize,
    sampler: ChannelSampler,
}

#[derive(Clone)]
struct AnimationClip {
    name: String,
    channels: Vec<AnimationChannel>,
    max_time: f32,
}

#[derive(Clone)]
pub struct SkinnedMesh {
    pub indices: Vec<u32>,
    pub joints: Vec<[u16; 4]>,
    pub weights: Vec<[f32; 4]>,
    pub base_positions: Vec<Vec3>,
    pub base_normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub joint_nodes: Vec<usize>,
    pub inverse_bind_matrices: Vec<Mat4>,
    pub node_parents: Vec<Option<usize>>,
    pub node_bind: Vec<NodeBind>,
    pub animations: Vec<AnimationClip>,
}

#[derive(Clone)]
pub struct NodeBind {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

pub struct SkinnedInstance {
    pub mesh: SkinnedMesh,
    pub current_clip: usize,
    pub time: f32,
}

pub struct SkinnedFrame {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
}

impl SkinnedInstance {
    pub fn new(mesh: SkinnedMesh, clip_name: Option<&str>) -> Self {
        let current_clip = clip_name
            .and_then(|name| {
                mesh.animations
                    .iter()
                    .position(|a| a.name == name)
            })
            .unwrap_or(0);
        Self {
            mesh,
            current_clip,
            time: 0.0,
        }
    }

    pub fn advance(&mut self, dt: f32) {
        let clip = &self.mesh.animations[self.current_clip];
        self.time = (self.time + dt) % clip.max_time.max(0.001);
    }

    fn compute_skin_matrices(&self) -> Vec<Mat4> {
        let clip = &self.mesh.animations[self.current_clip];

         
        let mut local_trs: Vec<(Vec3, Quat, Vec3)> = self
            .mesh
            .node_bind
            .iter()
            .map(|b| (b.translation, b.rotation, b.scale))
            .collect();

         
        for channel in &clip.channels {
            let value = sample_channel(&channel.sampler, self.time);
            match channel.sampler.path {
                gltf::animation::Property::Translation => {
                    local_trs[channel.target_node].0 = Vec3::from_slice(&value[0..3]);
                }
                gltf::animation::Property::Rotation => {
                    local_trs[channel.target_node].1 = Quat::from_xyzw(
                        value[0], value[1], value[2], value[3],
                    )
                    .normalize();
                }
                gltf::animation::Property::Scale => {
                    local_trs[channel.target_node].2 = Vec3::from_slice(&value[0..3]);
                }
                gltf::animation::Property::MorphTargetWeights => {}
            }
        }

         
        let mut world: Vec<Mat4> = vec![Mat4::IDENTITY; local_trs.len()];
        for (idx, parent) in self.mesh.node_parents.iter().enumerate() {
            let (t, r, s) = &local_trs[idx];
            let local = Mat4::from_scale_rotation_translation(*s, *r, *t);
            world[idx] = if let Some(p) = parent {
                world[*p] * local
            } else {
                local
            };
        }

         
        let mut skin_mats = Vec::with_capacity(self.mesh.joint_nodes.len());
        for (j_idx, joint_node) in self.mesh.joint_nodes.iter().enumerate() {
            let m = world[*joint_node] * self.mesh.inverse_bind_matrices[j_idx];
            skin_mats.push(m);
        }
        skin_mats
    }

    pub fn sample_matrices(&self) -> Vec<Mat4> {
        self.compute_skin_matrices()
    }

    pub fn sample(&self) -> SkinnedFrame {
        let skin_mats = self.compute_skin_matrices();
         
        let mut positions = vec![Vec3::ZERO; self.mesh.base_positions.len()];
        let mut normals = vec![Vec3::ZERO; self.mesh.base_normals.len()];

        for i in 0..self.mesh.base_positions.len() {
            let pos = self.mesh.base_positions[i];
            let nrm = self.mesh.base_normals[i];
            let joints = self.mesh.joints[i];
            let weights = self.mesh.weights[i];

            let mut skinned_pos = Vec4::ZERO;
            let mut skinned_nrm = Vec3::ZERO;
            for k in 0..4 {
                let w = weights[k];
                if w <= 0.0 {
                    continue;
                }
                let j = joints[k] as usize;
                if j >= skin_mats.len() {
                    continue;
                }
                let m = skin_mats[j];
                skinned_pos += m * pos.extend(1.0) * w;
                let n = (m * nrm.extend(0.0)).truncate();
                skinned_nrm += n * w;
            }

            positions[i] = skinned_pos.truncate();
            normals[i] = skinned_nrm.normalize_or_zero();
        }

        SkinnedFrame { positions, normals }
    }
}

fn sample_channel(sampler: &ChannelSampler, time: f32) -> Vec<f32> {
    let count = sampler.input.len();
    if count == 0 {
        return vec![0.0; sampler.stride];
    }
    let t_max = sampler.input[count - 1];
    let t = if t_max > 0.0 { time % t_max } else { 0.0 };

     
    let mut idx = 0;
    while idx + 1 < count && t > sampler.input[idx + 1] {
        idx += 1;
    }
    if idx + 1 >= count {
        return sampler.outputs[idx * sampler.stride..(idx + 1) * sampler.stride].to_vec();
    }
    let t0 = sampler.input[idx];
    let t1 = sampler.input[idx + 1];
    let local_t = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0);
    let a = &sampler.outputs[idx * sampler.stride..(idx + 1) * sampler.stride];
    let b = &sampler.outputs[(idx + 1) * sampler.stride..(idx + 2) * sampler.stride];

    match sampler.path {
        gltf::animation::Property::Rotation => {
            let qa = Quat::from_xyzw(a[0], a[1], a[2], a[3]).normalize();
            let qb = Quat::from_xyzw(b[0], b[1], b[2], b[3]).normalize();
            let q = qa.slerp(qb, local_t);
            vec![q.x, q.y, q.z, q.w]
        }
        _ => (0..sampler.stride)
            .map(|i| a[i] + (b[i] - a[i]) * local_t)
            .collect(),
    }
}

pub fn load_skinned_gltf(path: &str) -> SkinnedMesh {
    let (doc, buffers, _) = gltf::import(path).expect("Failed to load skinned glTF");

    let mut node_parents = vec![None; doc.nodes().len()];
    for node in doc.nodes() {
        for child in node.children() {
            node_parents[child.index()] = Some(node.index());
        }
    }

    let mut node_bind = Vec::new();
    for node in doc.nodes() {
        let (t, r, s) = node.transform().decomposed();
        node_bind.push(NodeBind {
            translation: Vec3::from(t),
            rotation: Quat::from_xyzw(r[0], r[1], r[2], r[3]),
            scale: Vec3::from(s),
        });
    }

     
    let skin = doc.skins().next().expect("No skin in skinned glTF");
    let joint_nodes: Vec<usize> = skin.joints().map(|j| j.index()).collect();
    let inverse_bind_matrices: Vec<Mat4> = {
        let reader = skin.reader(|b| Some(&buffers[b.index()]));
        reader
            .read_inverse_bind_matrices()
            .expect("read matrices")
            .map(|m| Mat4::from_cols_array_2d(&m))
            .collect()
    };

    let mesh = doc.meshes().next().expect("No mesh");
    let primitive = mesh.primitives().next().expect("No primitive");

    let reader = primitive.reader(|b| Some(&buffers[b.index()]));
    let positions: Vec<Vec3> = reader
        .read_positions()
        .expect("positions")
        .map(Vec3::from)
        .collect();
    let normals: Vec<Vec3> = reader
        .read_normals()
        .map(|n| n.map(Vec3::from).collect())
        .unwrap_or_else(|| vec![Vec3::Y; positions.len()]);
    let uvs: Vec<Vec2> = reader
        .read_tex_coords(0)
        .map(|tc| tc.into_f32().map(Vec2::from).collect())
        .unwrap_or_else(|| vec![Vec2::ZERO; positions.len()]);
    let joints: Vec<[u16; 4]> = reader
        .read_joints(0)
        .expect("joints")
        .into_u16()
        .collect();
    let weights: Vec<[f32; 4]> = reader
        .read_weights(0)
        .expect("weights")
        .into_f32()
        .collect();
    let indices: Vec<u32> = reader
        .read_indices()
        .map(|i| i.into_u32().collect())
        .unwrap_or_else(|| (0..positions.len() as u32).collect());

    let mut animations = Vec::new();
    for anim in doc.animations() {
        let mut channels = Vec::new();
        let mut max_time: f32 = 0.0;
        for ch in anim.channels() {
            let target = ch.target();
            let reader = ch.reader(|b| Some(&buffers[b.index()]));
            let sampler = ch.sampler();
            let input: Vec<f32> = reader.read_inputs().unwrap().collect();
            max_time = max_time.max(input.last().cloned().unwrap_or(0.0));
            let outputs: Vec<f32>;
            let stride;
            if let Some(out) = reader.read_outputs() {
                match out {
                    ReadOutputs::Translations(v) => {
                        stride = 3;
                        outputs = v.flat_map(|v| v.to_vec()).collect();
                    }
                    ReadOutputs::Rotations(v) => {
                        stride = 4;
                        outputs = v
                            .into_f32()
                            .flat_map(|v| v.to_vec())
                            .collect();
                    }
                    ReadOutputs::Scales(v) => {
                        stride = 3;
                        outputs = v.flat_map(|v| v.to_vec()).collect();
                    }
                    ReadOutputs::MorphTargetWeights(_) => {
                        continue;
                    }
                }
            } else {
                continue;
            }

            let sampler = ChannelSampler {
                input,
                outputs,
                stride,
                path: target.property(),
                interpolation: sampler.interpolation(),
            };
            channels.push(AnimationChannel {
                target_node: target.node().index(),
                sampler,
            });
        }
        if channels.is_empty() {
            continue;
        }
        animations.push(AnimationClip {
            name: anim.name().unwrap_or("clip").to_string(),
            channels,
            max_time,
        });
    }

    SkinnedMesh {
        indices,
        joints,
        weights,
        base_positions: positions,
        base_normals: normals,
        uvs,
        joint_nodes,
        inverse_bind_matrices,
        node_parents,
        node_bind,
        animations,
    }
}
