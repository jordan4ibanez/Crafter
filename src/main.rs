extern crate glfw;

mod helper;
mod graphics;
mod controls;
mod time;
mod chunk_mesh_procedure;
mod world;
mod blocks;
mod lua;
mod biomes;

use glfw::*;

use graphics::window_controls::toggle_full_screen;
use mlua::Lua;
use opensimplex_noise_rs::OpenSimplexNoise;

use std::{
    sync::mpsc::Receiver
};

use crate::{
    graphics::{
        shader_program::{
            ShaderProgram
        },
        mesh_component_system::{
            *
        },
        render::Renderer,
        window_controls::WindowVariables
    },

    time::time_object::{
            Time
        },
    chunk_mesh_procedure::{
        chunk_mesh_creation,
        chunk_mesh_generator_queue::{
            ChunkMeshGeneratorQueue,
            MeshUpdate
        }
    },
    world::{
        world::{
            *,
        },        
    }, 
    controls::{
        keyboard::Keyboard, 
        mouse::Mouse
    }, blocks::block_component_system::{BlockComponentSystem},
    lua::{
        lua_initialize::initialize_lua,
        lua_intake_api::intake_api_values
    },
        helper::helper_functions::get_path_string, biomes::{biome_generator::gen_biome, generation_component_system::GenerationComponentSystem},

};

pub const SEED: u64 = 123213123;

fn main() {

    // glfw initialization and configuration

    // initalize glfw
    let mut glfw: Glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // borrow and mutate glfw
    // return created glfw window
    let (mut window, events) = graphics::set_up::set_up_glfw(&mut glfw);


    // testing of 3D camera
    window.set_cursor_mode(glfw::CursorMode::Disabled);

    // let mut thread_rng: ThreadRng = rand::thread_rng();

    // fps counter object
    let mut time_object: Time = Time::new(&glfw);

    // window title - reused pointer
    let mut window_title: String = String::new();

    println!("Current Working Path: {}", get_path_string());

    let mut keyboard: Keyboard = Keyboard::new();
    let mut mouse: Mouse = Mouse::new(&window);

    // noise structure
    let noise = OpenSimplexNoise::new(Some(SEED as i64));

    const RENDER_DISTANCE: i32 = 20;

    // construct the renderer
    let mut renderer: Renderer = Renderer::new();
    renderer.set_render_distance(RENDER_DISTANCE as f32 * 16.0);
    let mut default_shader: ShaderProgram = ShaderProgram::new(
         "/shader_code/vertex_shader.vs",
        "/shader_code/fragment_shader.fs"
    );
    default_shader.create_uniform("projection_matrix");
    default_shader.create_uniform("model_matrix");
    // default_shader.create_uniform("game_render_distance");
    default_shader.test();
    renderer.add_shader_program("default", default_shader);

    let mut mcs: MeshComponentSystem = MeshComponentSystem::init();

    let mut bcs: BlockComponentSystem = BlockComponentSystem::new();

    let mut gcs: GenerationComponentSystem = GenerationComponentSystem::new();

    let mut window_variables: WindowVariables = WindowVariables::new();

    let mut world: World = World::initialize();

    let mut debug_x = -RENDER_DISTANCE;
    let mut debug_z = -RENDER_DISTANCE;

    let mut continue_debug = true;

    let mut chunk_mesh_generator_queue: ChunkMeshGeneratorQueue = ChunkMeshGeneratorQueue::new();
    
    let mut poll = true;
    
    // let debug_texture: u32 = mcs.new_texture("/mods/default/textures/dirt.png");
    // bcs.register_block("dirt", vec![String::from("test.png")], None, DrawType::Normal);

    let lua: Lua = initialize_lua();

    intake_api_values(&lua, &mut gcs, &mut mcs, &mut bcs);


    // main program loop
    while !window.should_close() {

        // here is testing for the logic of the chunk mesh generator queue
        if poll {

            let mesh_update_option: Option<MeshUpdate> = chunk_mesh_generator_queue.pop_front();

            // does this update exist?
            match mesh_update_option {
                Some(mesh_update) => {
                    // add neighbors to queue if told to do so
                    if mesh_update.update_neighbors() {
                        chunk_mesh_generator_queue.batch_neighbor_update(mesh_update.get_x(), mesh_update.get_z());
                    }

                    let mesh: Option<u32> = chunk_mesh_creation::create_chunk_mesh(&bcs, &mut mcs, &world, mesh_update.get_x(), mesh_update.get_z(), 1);//debug_texture);
                    match mesh {
                        Some(unwrapped_mesh) => {
                            world.set_chunk_mesh(&mut mcs, mesh_update.get_x(), mesh_update.get_z(), unwrapped_mesh);
                            world.sort_map(renderer.get_camera().get_pos());                            
                        },
                        None => (),
                    }
                },
                None => {
                    if !continue_debug {
                        poll = false;
                        println!("DONE GENERATING MESHES!");
                    }
                },
            } 
            // println!("RUNNING")     
        }


        // this is chunk generation debug
        // this needs to be turned into an async queue
        if continue_debug {
            
            // println!(" CREATING {} {}", debug_x, debug_y);
            world.add_chunk(debug_x, debug_z);

            gen_biome(
                &gcs,
                world.get_chunk_blocks_mut(debug_x, debug_z).unwrap(),
                debug_x,
                debug_z,
                &noise
            );

            // world.add(generated_chunk);

            chunk_mesh_generator_queue.push_back(debug_x, debug_z, true);


            // this part just ticks up the generation value
            debug_x += 1;

            if debug_x > RENDER_DISTANCE {
                debug_x = -RENDER_DISTANCE;

                debug_z += 1;

                if debug_z > RENDER_DISTANCE {
                    continue_debug = false;
                    println!("DONE GENERATING CHUNKS!");
                }
            }
        }

        let delta: f64 = time_object.calculate_delta(&glfw);

        glfw.poll_events();

        mouse.reset();

        // this is where all events are processed
        process_events(&mut glfw, &mut window, &events, &mut mouse, &mut keyboard, &mut window_variables);


        let update_chunk_ordering: bool = renderer.get_camera_mut().on_tick(&mouse, &keyboard, delta as f32);

        if update_chunk_ordering {
            world.sort_map(renderer.get_camera().get_pos());
        }

        renderer.render(&mut mcs, &window, &mut world);


        let returned_value = time_object.count_fps(&glfw);

        if returned_value {
            window_title.push_str("FPS: ");
            window_title.push_str(&time_object.get_fps().to_string());
            window.set_title(&window_title);
            window_title.clear();
        }

        
        window.swap_buffers();
     
    }

    world.clean_up(&mut mcs);
    renderer.clean_up();

    mcs.final_clean_up();

    println!("Program exited successfully!");

}


// event processing, keys, mouse, etc
fn process_events(
    glfw: &mut Glfw,
    window: &mut glfw::Window,
    events: &Receiver<(f64, glfw::WindowEvent)>,

    mouse: &mut Mouse,
    keyboard: &mut Keyboard,

    window_variables: &mut WindowVariables
) {
    // iterate events
    for (_, event) in glfw::flush_messages(events) {


        keyboard.process_events(&event);
        mouse.process_events(&event);

        // match event enums
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                // update the gl viewport to match the new window size
                unsafe {
                    gl::Viewport(0,0, width, height)
                }
            }

            // close the window on escape
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
            glfw::WindowEvent::Key(Key::F11, _, Action::Press, _) => toggle_full_screen(glfw, window, window_variables),

            _ => ()
        }
    }
}