pub mod sphere;
pub mod cube;

use bytemuck::{Pod, Zeroable};

// Constants
const PI: f32 = 3.14159265359;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Vertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2], // _color: [f32; 4],
}

fn vertex(pos: [i8; 3], tc: [i8; 2]) -> Vertex {
    Vertex {
        _pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
        _tex_coord: [tc[0] as f32, tc[1] as f32],
    }
}

pub fn merge_index_data(index_data1: &Vec<u16>, index_data2: &mut Vec<u16>, vertex_count1: u16) -> Vec<u16> {
    for i in 0..index_data2.len() {
        index_data2[i] += vertex_count1;
    }

    [&index_data1[..], &index_data2[..]].concat()
}

pub fn translate_vertex_data(vertex_data: &mut Vec<Vertex>, matrix: &[f32; 4]) {
    for i in 0..vertex_data.len() {
        for j in 0..4 {
            vertex_data[i]._pos[j] += matrix[j];
        }
    }
}