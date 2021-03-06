

// this is laid out in this manor because it is easier to debug


/*

function ideas:

intake rotation and pass back texture coordinates for up and down

intake size and generate positions with a function


information

the function stripe() is an interlacing function. In openGL this is called interlacing vertex data.
it is called stripe because it is easier to type, simpler to read, and easier to understand that it's striping data.

this is extremely similar to RAID-0 with hard drive/ssd technology


*/


// generic functions to reduce boilerplate

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::blocks::block_component_system::{AtlasTextureMap};

// pushes the adjusted xyz into the vertex data
fn set_pos(pos: &mut [f32], x: f32, y: f32, z: f32) {
    let mut xyz_index: i8 = 0;
    // iterate and modify for xyz values
    pos.iter_mut().for_each( | value: &mut f32 | {
        match xyz_index {
            0 => *value += x,
            1 => *value += y,
            2 => *value += z,
            _ => ()
        }

        xyz_index += 1;

        if xyz_index == 3 {
            xyz_index = 0;
        }
    });
}

// adjusts the indices to the correct value from base
fn adjust_indices(index: &mut [u32], face_count: &mut usize) {
    index.iter_mut().for_each( | value: &mut u32 | {
        *value += *face_count as u32;
    });
    *face_count += 4;
}

// pushes the array slice into vector
fn assign_indices(vector: &mut Vec<u32>, array: &[u32], current_count: &mut usize) {
    array.iter().for_each( | value: &u32 | {
        vector[*current_count] = *value;
        *current_count += 1;
    });
}


// a precalculator for capacity information
pub fn dry_run(float_count: &AtomicUsize, indices_count: &AtomicUsize) {
    /*
    pos     12
    color   12
    texture 8
    */
    float_count.fetch_add(32, Ordering::Relaxed);

    // indices 6
    indices_count.fetch_add(6, Ordering::Relaxed);    
}

// this interlaces the mesh data for the gpu
fn stripe(float_data: &mut Vec<f32>, pos: &[f32], color: &[f32], texture: &[f32], float_count: &mut usize) {

    for index in 0..4 {

        // pos
        for i in 0..3 {
            float_data[*float_count] = pos[(index * 3) + i];
            *float_count += 1;
        }

        // color
        for i in 0..3 {
            float_data[*float_count] = color[(index * 3) + i];
            *float_count += 1;
        }

        // texture
        for i in 0..2 {
            float_data[*float_count] = texture[(index * 2) + i];
            *float_count += 1;
        }
    }
}

// a simple wrap around function
fn overflow(value: &mut usize) -> usize {
    if *value > 7 {
        *value -= 8;
    }
    *value
}

fn calculate_face_rotation(texture_map_table: &mut [f32; 8], face_rotation: u8) {
    // don't bother if no rotation
    if face_rotation == 0 {
        return;
    }
    // since x and y are linear in rows of 2 we must shift them by double the rotation
    let multiplier = (face_rotation * 2) as usize;

    let mut cloning_table: [f32; 8] = [0.0; 8];

    for i in 0..8 {
        cloning_table[i] = texture_map_table[overflow(&mut (i + multiplier))]
    }

    // shadow the table
    *texture_map_table = cloning_table;
}

/*
let mut texture: [f32; 8] = [
    min_x, min_y, // 0
    min_x, max_y, // 1
    max_x, max_y, // 2
    max_x, min_y, // 3
];
*/

// micro function for this specific case to check if floats are equal
fn float_eq(value_1: f32, value_2: f32) -> bool {
    let check_1 = (value_1 * 1_000_000.0).floor() as i32;
    let check_2 = (value_2 * 1_000_000.0).floor() as i32;

    check_1 == check_2
}

// this is hardcoded because there is only 1 way to flip
fn flip_axis(texture_map_table: &mut [f32; 8], flip: u8, min_x: f32, min_y: f32, max_x: f32, max_y: f32) {
    // don't bother if no flip
    if flip == 0 {
        return;
    }
    // flip X
    else if flip == 1 {        
        for i in 0..4 {
            let index = i * 2;
            // if min_x
            if float_eq(texture_map_table[index], min_x) {
                texture_map_table[index] = max_x;
            }
            // if max_x
            else {
                texture_map_table[index] = min_x;
            }
        }
    }
    // flip y
    else if flip == 2 {
        for i in 0..4 {
            let index = (i * 2) + 1;
            // if min_y
            if float_eq(texture_map_table[index], min_y) {
                texture_map_table[index] = max_y;
            }
            // if max_y
            else {
                texture_map_table[index] = min_y;
            }
        }
    }

}


pub fn face_up(

    atlas_map: &AtlasTextureMap,

    float_data: &mut Vec<f32>,
    indices_data: &mut Vec<u32>,

    float_count: &mut usize,
    indices_count: &mut usize,
    face_count: &mut usize,

    x: f32,
    y: f32,
    z: f32,
    light: f32
) {

    // first assign all float data

    // vertex data

    let mut pos: [f32; 12] = [
        0., 1., 0., // 0
        0., 1., 1., // 1
        1., 1., 1., // 2
        1., 1., 0., // 3
    ];
    set_pos(&mut pos, x, y, z);

    // light/color data
    let color: [f32; 12] = [
        light, light, light, // 0
        light, light, light, // 1
        light, light, light, // 2
        light, light, light, // 3
    ];

    // texture coordinates

    let (min_x, min_y, max_x, max_y, face_rotation, flip) = atlas_map.get_as_tuple();

    let mut texture: [f32; 8] = [
        min_x, min_y, // 0
        min_x, max_y, // 1
        max_x, max_y, // 2
        max_x, min_y, // 3
    ];

    // flip then rotate
    flip_axis(&mut texture, flip, min_x, min_y, max_x, max_y);

    calculate_face_rotation(&mut texture, face_rotation);

    stripe(float_data, &pos, &color, &texture, float_count);


    // finally assign vertices data

    // index (face/indices) data

    let mut index: [u32; 6] = [
        // tri 1
        0,1,2,

        // tri 2
        2,3,0
    ];
    adjust_indices(&mut index, face_count);
    
    assign_indices(indices_data, &index, indices_count);
}


pub fn face_down(

    atlas_map: &AtlasTextureMap,

    float_data: &mut Vec<f32>,
    indices_data: &mut Vec<u32>,

    float_count: &mut usize,
    indices_count: &mut usize,
    face_count: &mut usize,

    x: f32,
    y: f32,
    z: f32,
    light: f32
) {

        // vertex data

        let mut pos: [f32; 12] = [
            0., 0., 1., // 0
            0., 0., 0., // 1
            1., 0., 0., // 2
            1., 0., 1., // 3
        ];
        set_pos(&mut pos, x, y, z);        

        // light/color data
        let color: [f32; 12] = [
            light, light, light, // 0
            light, light, light, // 1
            light, light, light, // 2
            light, light, light, // 3
        ];

        // texture coordinates
        let (min_x, min_y, max_x, max_y, face_rotation, flip) = atlas_map.get_as_tuple();
        
        let mut texture: [f32; 8] = [
            max_x, max_y, // 0
            max_x, min_y, // 1
            min_x, min_y, // 2
            min_x, max_y, // 3
        ];

        // flip then rotate
        flip_axis(&mut texture, flip, min_x, min_y, max_x, max_y);

        calculate_face_rotation(&mut texture, face_rotation);
    
        stripe(float_data, &pos, &color, &texture, float_count);


        // index (face/indices) data
    
        let mut index: [u32; 6] = [
            // tri 1
            0,1,2,    
            // tri 2
            2,3,0
        ];
        adjust_indices(&mut index, face_count);

        assign_indices(indices_data, &index, indices_count);
}



pub fn face_south(

    atlas_map: &AtlasTextureMap,

    float_data: &mut Vec<f32>,
    indices_data: &mut Vec<u32>,

    float_count: &mut usize,
    indices_count: &mut usize,
    face_count: &mut usize,

    x: f32,
    y: f32,
    z: f32,
    light: f32
) {

    // vertex data

    let mut pos: [f32; 12] = [
        0., 1., 1., // 0
        0., 0., 1., // 1
        1., 0., 1., // 2
        1., 1., 1., // 3
    ];
    set_pos(&mut pos, x, y, z);

    // light/color data
    let color: [f32; 12] = [
        light, light, light, // 0
        light, light, light, // 1
        light, light, light, // 2
        light, light, light, // 3
    ];

    // texture coordinates
    let (min_x, min_y, max_x, max_y, face_rotation, flip) = atlas_map.get_as_tuple();

    let mut texture: [f32; 8] = [
        min_x, min_y, // 0
        min_x, max_y, // 1
        max_x, max_y, // 2
        max_x, min_y, // 3    
    ];

    // flip then rotate
    flip_axis(&mut texture, flip, min_x, min_y, max_x, max_y);

    calculate_face_rotation(&mut texture, face_rotation);

    stripe(float_data, &pos, &color, &texture, float_count);


    // index (face/indices) data

    let mut index: [u32; 6] = [
        // tri 1
        0,1,2,
        // tri 2
        2,3,0
    ];

    adjust_indices(&mut index, face_count);
    
    assign_indices(indices_data, &index, indices_count);
}

pub fn face_north(

    atlas_map: &AtlasTextureMap,

    float_data: &mut Vec<f32>,
    indices_data: &mut Vec<u32>,

    float_count: &mut usize,
    indices_count: &mut usize,
    face_count: &mut usize,

    x: f32,
    y: f32,
    z: f32,
    light: f32
) {
    
    // vertex data

    let mut pos: [f32; 12] = [
        0., 0., 0., // 0
        0., 1., 0., // 1
        1., 1., 0., // 2
        1., 0., 0., // 3
    ];
    set_pos(&mut pos, x, y, z);

    // light/color data
    let color: [f32; 12] = [
        light, light, light, // 0
        light, light, light, // 1
        light, light, light, // 2
        light, light, light, // 3
    ];

    // texture coordinates
    let (min_x, min_y, max_x, max_y, face_rotation, flip) = atlas_map.get_as_tuple();

    let mut texture: [f32; 8] = [
        max_x, max_y, // 0
        max_x, min_y, // 1
        min_x, min_y, // 2
        min_x, max_y, // 3
    ];

    // flip then rotate
    flip_axis(&mut texture, flip, min_x, min_y, max_x, max_y);

    calculate_face_rotation(&mut texture, face_rotation);

    stripe(float_data, &pos, &color, &texture, float_count);


    // index (face/indices) data

    let mut index: [u32; 6] = [
        // tri 1
        0,1,2,
        // tri 2
        2,3,0
    ];
    adjust_indices(&mut index, face_count);
    
    assign_indices(indices_data, &index, indices_count);    
}


pub fn face_west(

    atlas_map: &AtlasTextureMap,

    float_data: &mut Vec<f32>,
    indices_data: &mut Vec<u32>,

    float_count: &mut usize,
    indices_count: &mut usize,
    face_count: &mut usize,

    x: f32,
    y: f32,
    z: f32,
    light: f32
) {
    
    // vertex data

    let mut pos: [f32; 12] = [
        1., 0., 1., // 0
        1., 0., 0., // 1
        1., 1., 0., // 2
        1., 1., 1., // 3
    ];
    set_pos(&mut pos, x, y, z);

    // light/color data
    let color: [f32; 12] = [
        light, light, light, // 0
        light, light, light, // 1
        light, light, light, // 2
        light, light, light, // 3   
    ];

    // texture coordinates
    let (min_x, min_y, max_x, max_y, face_rotation, flip) = atlas_map.get_as_tuple();

    let mut texture: [f32; 8] = [
        min_x, max_y, // 0
        max_x, max_y, // 1
        max_x, min_y, // 2
        min_x, min_y, // 3
    ];

    // flip then rotate
    flip_axis(&mut texture, flip, min_x, min_y, max_x, max_y);

    calculate_face_rotation(&mut texture, face_rotation);

    stripe(float_data, &pos, &color, &texture, float_count);


    // index (face/indices) data

    let mut index: [u32; 6] = [
        // tri 1
        0,1,2,
        // tri 2
        2,3,0
    ];

    adjust_indices(&mut index, face_count);

    assign_indices(indices_data, &index, indices_count);
}



pub fn face_east(

    atlas_map: &AtlasTextureMap,

    float_data: &mut Vec<f32>,
    indices_data: &mut Vec<u32>,

    float_count: &mut usize,
    indices_count: &mut usize,
    face_count: &mut usize,

    x: f32,
    y: f32,
    z: f32,
    light: f32
) {
    
    // vertex data

    let mut pos: [f32; 12] = [
        0., 0., 0., // 0
        0., 0., 1., // 1
        0., 1., 1., // 2
        0., 1., 0., // 3
    ];
    set_pos(&mut pos, x, y, z);

    // light/color data
    
    let color: [f32; 12] = [
        light, light, light, // 0
        light, light, light, // 1
        light, light, light, // 2
        light, light, light, // 3
    ];   

    // texture coordinates
    let (min_x, min_y, max_x, max_y, face_rotation, flip) = atlas_map.get_as_tuple();

    let mut texture: [f32; 8] = [
        min_x, max_y, // 0
        max_x, max_y, // 1
        max_x, min_y, // 2
        min_x, min_y, // 3
    ];

    // flip then rotate
    flip_axis(&mut texture, flip, min_x, min_y, max_x, max_y);

    calculate_face_rotation(&mut texture, face_rotation);

    stripe(float_data, &pos, &color, &texture, float_count);

    // index (face/indices) data

    let mut index: [u32; 6] = [
        // tri 1
        0,1,2,
        // tri 2
        2,3,0
    ];

    adjust_indices(&mut index, face_count);
    
    assign_indices(indices_data, &index, indices_count);
}



// the packed boilerplate to allow a single function call
pub fn add_block(
    
    block_atlas_map: &Vec<AtlasTextureMap>,

    float_data: &mut Vec<f32>,
    indices_data: &mut Vec<u32>,
    
    float_count: &mut usize,
    face_count: &mut usize,
    indices_count: &mut usize,

    x_plus: bool,
    x_minus: bool,
    y_plus: bool,
    y_minus: bool,
    z_plus: bool,
    z_minus: bool,

    x: f32,
    y: f32,
    z: f32,
    light: f32
) {

    let side_face_light_subtraction =  0.75 / 16.0;

    if y_plus {
        face_up(
            &block_atlas_map[0],

            float_data,
            indices_data,

            float_count,
            indices_count,
            face_count,

            x,
            y,
            z,
            light
        );
    }
    
    if y_minus {
        face_down(
            &block_atlas_map[1],

            float_data,
            indices_data,

            float_count,
            indices_count,
            face_count,

            x,
            y,
            z,
            light
        );
    }
    
    
    if z_plus {
        face_south(
            &block_atlas_map[2],

            float_data,
            indices_data,

            float_count,
            indices_count,
            face_count,

            x,
            y,
            z,
            light - side_face_light_subtraction
        );
    }

    
    if z_minus {
        face_north(
            &block_atlas_map[3],

            float_data,
            indices_data,

            float_count,
            indices_count,
            face_count,

            x,
            y,
            z,
            light - side_face_light_subtraction
        );
    }

    
    /*

    +y = up
    -y = down
    
    +z = south
    -z = north

    +x = west
    -x = east

    */

    if x_plus {
        face_west(
            &block_atlas_map[4],

            float_data,
            indices_data,

            float_count,
            indices_count,
            face_count,

            x,
            y,
            z,
            light - side_face_light_subtraction
        );
    }
    
    
    if x_minus {
        face_east(
            &block_atlas_map[5],

            float_data,
            indices_data,

            float_count,
            indices_count,
            face_count,

            x,
            y,
            z,
            light - side_face_light_subtraction
        );
    }    
}