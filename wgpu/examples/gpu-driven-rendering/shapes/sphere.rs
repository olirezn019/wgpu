use crate::shapes::{Vertex, vertex, PI};
use std::vec::Vec;

pub fn map(value: f32, range1: [f32; 2], range2: [f32; 2]) -> f32 {
    // Check values passed to the function
    if range1[1] <= range1[0] || range2[1] <= range2[0] {
        println!("Wrong values were passed in the function.");
        return 0.0;
    }
    else if value > range1[1] || value < range1[0] {
        println!("Value passed to the function is not inside the given range.");
        return 0.0;
    }

    // Calculate how big are those ranges
    let size1 = range1[1] - range1[0];
    let size2 = range2[1] - range2[0];

    // How far is value from the start of its range
    let distance = value - range1[0];

    // Calculate percent and the resulting value
    let percentage = distance / size1;
    let result_value = (size2 * percentage) + range2[0];

    result_value
}

// Get position of two dimensional value in one dimensional array
fn pos_in_one_dim_vec(x: u16, y: u16, columns: u16) -> u16 {
    y*columns+x
}

pub fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
    let r: f32 = 1.0;
    let resolution: u16 = 20;

    let mut vertices: Vec<Vertex> = Vec::<Vertex>::new();

    for i in 0..resolution {
        let lon = map(i as f32, [0.0, resolution as f32], [-PI, PI]);
        for j in 0..resolution {
            let lat = map(j as f32, [0.0, resolution as f32], [-PI/2.0, PI/2.0]);
            let x = r * lon.sin() * lat.cos();
            let y = r * lon.sin() * lat.sin();
            let z = r * lon.cos();

            vertices.push(Vertex {
                _pos: [x, y, z, 1.0],
                _tex_coord: [0.0, 0.0],
            });
        }
    }

    let mut indexes: Vec<u16> = Vec::<u16>::new();

    for y in 0..resolution {
        for x in 0..resolution {
            //let position: [f32; 4] = vertices[(y*resolution+x) as usize]._pos;
            // first triangle
            indexes.push(pos_in_one_dim_vec(x, y+1, resolution));
            indexes.push(pos_in_one_dim_vec(x+1, y+1, resolution));
            indexes.push(pos_in_one_dim_vec(x+1, y, resolution));

            // second triangle
            indexes.push(pos_in_one_dim_vec(x+1, y, resolution));
            indexes.push(pos_in_one_dim_vec(x, y, resolution));
            indexes.push(pos_in_one_dim_vec(x, y+1, resolution));
        }
    }
    //println!("{:?}", indexes);

    (vertices, indexes)
}