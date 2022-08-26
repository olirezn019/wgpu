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

pub enum MeshType {
    Cube,
    Cylinder,
    Sphere
}

pub enum TextureType {
    Water,
    Grass
}

pub struct Object {
    pub id: u32,
    pub mesh: MeshType,
    pub texture: TextureType
}

impl Object {
    pub fn generate_vertices(&self) -> (Vec<Vertex>, Vec<u16>) {
        let result;
        match self.mesh {
            MeshType::Cube => result = cube::create_vertices(),
            MeshType::Cylinder => result = cylinder::generate_vertices(),
            MeshType::Sphere => result = sphere::generate_vertices()
        }
        result
    }
}

fn vertex(pos: [i8; 3], c: [u8; 4]) -> Vertex {
    Vertex {
        _pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
        _color: crate::create_color(c),
    }
}

pub fn merge_index_data(index_data1: &Vec<u16>, index_data2: &mut Vec<u16>, vertex_count1: u16) -> Vec<u16> {
    for i in 0..index_data2.len() {
        index_data2[i] += vertex_count1;
    }

    [&index_data1[..], &index_data2[..]].concat()
}

pub fn translate_vertices(vertex_arr: &mut Vec<Vertex>, matrix: &[f32; 4]) {
    for i in 0..vertex_arr.len() {
        for j in 0..4 {
            vertex_arr[i]._pos[j] += matrix[j];
        }
    }
}