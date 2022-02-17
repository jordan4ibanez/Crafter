

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
fn adjust_indices(index: &mut [u32], face_count: &mut u32) {
    index.iter_mut().for_each( | value: &mut u32 | {
        *value += *face_count;
    });
    *face_count += 4;
}

// pushes the array slice into vector
fn assign_indices(vector: &mut Vec<u32>, array: &[u32], current_count: &mut u32) {
    array.iter().for_each( | value: &u32 | {
        vector[*current_count as usize] = *value;
        *current_count += 1;
    });
}


// a precalculator for capacity information
pub fn dry_run(float_count: &mut u32, indices_count: &mut u32) {
    *float_count += 12; // pos
    *float_count += 12; // color
    *float_count += 8; // texture

    *indices_count += 6;
}

// this interlaces the mesh data for the gpu
fn stripe(float_data: &mut Vec<f32>, pos: &[f32], color: &[f32], texture: &[f32], float_count: &mut u32) {

    for index in 0..4 {

        // pos
        for i in 0..3 {
            float_data[*float_count as usize] = pos[(index * 3) + i];
            *float_count += 1;
        }

        // color
        for i in 0..3 {
            float_data[*float_count as usize] = color[(index * 3) + i];
            *float_count += 1;
        }

        // texture
        for i in 0..2 {
            float_data[*float_count as usize] = texture[(index * 2) + i];
            *float_count += 1;
        }
    }
}

pub fn face_up(
    float_data: &mut Vec<f32>,
    indices_data: &mut Vec<u32>,

    float_count: &mut u32,
    indices_count: &mut u32,
    face_count: &mut u32,

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

    let texture: [f32; 8] = [
        0., 0., // 0
        0., 1., // 1
        1., 1., // 2
        1., 0., // 3
    ];

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
    float_data: &mut Vec<f32>,
    indices_data: &mut Vec<u32>,

    float_count: &mut u32,
    indices_count: &mut u32,
    face_count: &mut u32,

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
        let texture: [f32; 8] = [
            0., 1., // 0
            0., 0., // 1
            1., 0., // 2
            1., 1., // 3
        ];
    
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
    float_data: &mut Vec<f32>,
    indices_data: &mut Vec<u32>,

    float_count: &mut u32,
    indices_count: &mut u32,
    face_count: &mut u32,

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
    let texture: [f32; 8] = [
        0., 1., // 0
        0., 0., // 1
        1., 0., // 2
        1., 1., // 3   
    ];

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
    float_data: &mut Vec<f32>,
    indices_data: &mut Vec<u32>,

    float_count: &mut u32,
    indices_count: &mut u32,
    face_count: &mut u32,

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

    let texture: [f32; 8] = [
        0., 0., // 0
        0., 1., // 1
        1., 1., // 2
        1., 0., // 3
    ];

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
    float_data: &mut Vec<f32>,
    indices_data: &mut Vec<u32>,

    float_count: &mut u32,
    indices_count: &mut u32,
    face_count: &mut u32,

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

    let texture: [f32; 8] = [
        0., 0., // 0
        0., 1., // 1
        1., 1., // 2
        1., 0., // 3
    ];

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
    float_data: &mut Vec<f32>,
    indices_data: &mut Vec<u32>,

    float_count: &mut u32,
    indices_count: &mut u32,
    face_count: &mut u32,

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

    let texture: [f32; 8] = [        
        0., 1., // 0
        0., 0., // 1
        1., 0., // 2
        1., 1., // 3
    ];

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
    float_data: &mut Vec<f32>,
    indices_data: &mut Vec<u32>,
    
    float_count: &mut u32,
    face_count: &mut u32,
    indices_count: &mut u32,

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

    if y_plus {
        face_up(
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

    if z_minus {
        face_north(
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

    /*
    
    +z = south
    -z = north

    +y = up
    -y = down

    +x = west
    -x = east

    */

    if x_plus {
        face_west(
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
    
    if x_minus {
        face_east(
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
}