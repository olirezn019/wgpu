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