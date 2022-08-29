mod sphere;
mod cube;
mod cylinder;

use bytemuck::{Pod, Zeroable};

// Constants
const PI: f32 = 3.14159265359;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Vertex {
    _pos: [f32; 4],
    _color: [f32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MeshType {
    Cube,
    Cylinder,
    Sphere
}

#[derive(Debug)]
pub enum TextureType {
    Water,
    Grass,
    Rainbow
}

pub struct Mesh {
    pub m_type: MeshType,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>
}

impl Mesh {
    pub fn generate_vertices(&mut self) {
        let result;
        match self.m_type {
            MeshType::Cube => result = cube::create_vertices(),
            MeshType::Cylinder => result = cylinder::generate_vertices(),
            MeshType::Sphere => result = sphere::generate_vertices()
        }
        self.vertices = result.0;
        self.indices = result.1;
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Object {
    pub m_type: MeshType,
    pub texture: TextureType,
    pub transform_m: glam::Mat4
}

fn vertex(pos: [i8; 3], c: [u8; 4]) -> Vertex {
    Vertex {
        _pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
        _color: crate::create_color(c),
    }
}

pub fn merge_index_vertex_data(meshes: &Vec<&Mesh>) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices: Vec<Vertex> = Vec::<Vertex>::new();
    let mut indices: Vec<u16> = Vec::<u16>::new();

    // Vertex count of meshes before
    // We need to add this number to indices of the next mesh
    let mut vertex_count = 0;

    for m in meshes {
        vertices.extend(&m.vertices);

        for i in 0..m.indices.len() {
            indices.push(m.indices[i] + vertex_count as u16);
        }
        vertex_count += m.vertices.len();
    }

    (vertices, indices)
}

pub fn merge_matrices(objects: &Vec<&Object>) -> Vec<f32> {
    let mut matrices: Vec<f32> = Vec::<f32>::new();

    for o in objects {
        matrices.extend(
            &o.transform_m.to_cols_array_2d().concat()
        );
    }

    matrices
}