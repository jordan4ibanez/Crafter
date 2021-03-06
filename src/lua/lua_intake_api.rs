use std::path::Path;

use image::{
    DynamicImage
};
use mlua::{
    Lua,
    Table,
    prelude,
    Integer, Error
};
use texture_packer::{
    importer::ImageImporter,
    TexturePackerConfig,
    TexturePacker,
    exporter::ImageExporter
};

use crate::{
    blocks::block_component_system::{
        BlockComponentSystem,
        DrawType,
        BlockBox, AtlasTextureMap
    },
    graphics::mesh_component_system::MeshComponentSystem,
    helper::helper_functions::with_path,
    lua::lua_texture_atlas_calculation::{
        calculate_atlas_location_normal
    }, biomes::generation_component_system::{LayerDepth, NoiseParams, GenerationComponentSystem, BiomeOres}
};


fn get_texture_size(path_string: String) -> (u32, u32) {
    let path = Path::new(&path_string);
    let texture_option = ImageImporter::import_from_file(path);

    let texture: DynamicImage;

    match texture_option {
        Ok(texture_wrapped) =>  texture = texture_wrapped,
        Err(error) => {
            panic!("COULD NOT LOAD TEXTURE: {}! Error String: {}", path_string, error)
        }
    }
    (texture.width(), texture.height())
}

fn configure_texture_atlas(module_name: &str, texture_name: &str, number_of_textures: &mut u32, biggest_width: &mut u32, biggest_height: &mut u32) {
    let (width, height) = get_texture_size(with_path( &("/mods/".to_owned() + module_name + "/textures/" + texture_name) ));

    if width > *biggest_width {
        *biggest_width = width;
    }
    if height > *biggest_height {
        *biggest_height = height;
    }

    *number_of_textures += 1;
}

fn create_texture(module_name: &str, texture_name: &str) -> DynamicImage {
    let string_path: String = with_path( &("/mods/".to_owned() + module_name + "/textures/" + texture_name) );
    let path: &Path = Path::new(&string_path);
    ImageImporter::import_from_file(path).expect("UNABLE TO LOAD TEXTURE")
}


pub fn intake_api_values(lua: &Lua, gcs: &mut GenerationComponentSystem, mcs: &mut MeshComponentSystem, bcs: &mut BlockComponentSystem) {

    // this follows the same pattern as lua
    let crafter: Table = lua.globals().get("crafter").unwrap();
    let texture_cache: Table = crafter.get("texture_cache").unwrap();


    println!("-------BEGINNING TEST OF API TRANSLATION ------------");

    // this is done imperatively because it's easier to understand the program flow

    // first we must configure the texture atlas using the modules defined in lua
    let mut cached_table_values: Vec<(String, String)> = Vec::new();

    // iterate the base of the texture cache table - crafter.texture_cache
    for module_name in texture_cache.pairs::<String, Table>() {

        // the module_name_unwrapped.0 is the module name
        let module_name_unwrapped: (String, Table) = module_name.unwrap();

        // iterate through textures in module table - crafter.texture_cache[texture]
        for texture_name in module_name_unwrapped.1.pairs::<u32, String>() {

            // 0 - index | 1 - texture name (.png)
            let texture_name_unwrapped: (u32, String) = texture_name.unwrap();

            // insert the values into the vector tuple
            cached_table_values.push((module_name_unwrapped.0.clone(), texture_name_unwrapped.1));

        }
    }

    // println!("{:#?}", cached_table_values);

    // find the biggest size, and number of textures

    let mut biggest_width = 0;
    let mut biggest_height = 0;
    let mut number_of_textures = 0;

    for (module_name, texture_name) in cached_table_values.iter() {
        configure_texture_atlas(
            &module_name, 
            &texture_name,
            &mut number_of_textures,
            &mut biggest_width,
            &mut biggest_height
        )
    }

    // automatically configure the texture atlas with the supplied information

    // println!("width: {} | height: {}, number of textures: {}", biggest_width, biggest_height, number_of_textures);

    // configged width is the number of textures it can fit on that axis
    let configged_width: u32 = (number_of_textures + 2) / 2;
    let configged_height: u32 = ((number_of_textures + 2) / 2) + 1;

    // println!("{configged_width}");
    // println!("{configged_height}");

    let config = TexturePackerConfig {
        max_width: biggest_width * configged_width,
        max_height: biggest_height * configged_height,
        allow_rotation: false,
        texture_outlines: false,
        border_padding: 0,
        texture_padding: 0,
        texture_extrusion: 0,
        trim: false,
    };
    
    // this is the actual texture packer
    let mut packer: TexturePacker<DynamicImage, String> = TexturePacker::new_skyline(config);

    for (module_name, texture_name) in cached_table_values.iter() {
        let created_texture: DynamicImage = create_texture(
            &module_name, 
            &texture_name
        );

        packer.pack_own(texture_name.to_string(), created_texture).expect("Unable to pack texture!");
    }

    let atlas: DynamicImage = ImageExporter::export(&packer).unwrap();

    /*
    for value in packer.get_frames() {
        println!("{:#?}", value);
    }
    */

    // println!("atlas width: {} | atlas height: {}", atlas.width(), atlas.height());

    // iterate blocks to be put into Block Component System

    // iterating crafter.blocks
    let blocks: Table = crafter.get("blocks").unwrap();

    // intake all data from lua
    for blocks in blocks.pairs::<String, Table>() {

        let (_, lua_table) = blocks.unwrap();

        // these are required
        let block_name: String = lua_table.get("name").unwrap();

        // completely ignore air
        if block_name.eq("air") {
            continue;
        }

        let block_mod: String = lua_table.get("mod").unwrap();

        // println!("{}, {}", block_name, block_mod);

        // pull lua texture table into Rust String vector
        let lua_block_textures: Table = lua_table.get("textures").unwrap();
        let mut block_textures: Vec<String> = Vec::new();

        for value in lua_block_textures.pairs::<String, String>() {
            block_textures.push(value.unwrap().1);
        }

        // pull lua texture rotation table into Rust u8 vector
        let lua_block_rotations: Table = lua_table.get("rotations").unwrap();
        let mut block_rotations: Vec<u8> = Vec::new();

        for value in lua_block_rotations.pairs::<Integer, Integer>() {
            block_rotations.push(value.unwrap().1 as u8);
        }

        // pull lua texture flip table into Rust u8 vector
        let lua_block_flips: Table = lua_table.get("flips").unwrap();
        let mut block_flips: Vec<u8> = Vec::new();

        for value in lua_block_flips.pairs::<Integer, Integer>() {
            block_flips.push(value.unwrap().1 as u8);
        }
        

        // begin the optional values
        let draw_type_option: Result<String, prelude::LuaError> = lua_table.get("draw_type");

        let draw_type: DrawType;

        // block boxes will need an advanced precalculation per box
        match draw_type_option {
            Ok(draw_type_string) => {
                match draw_type_string.as_str() {
                    "normal" => draw_type = DrawType::Normal,
                    "airlike" => draw_type = DrawType::None,
                    "block_box" => draw_type = DrawType::BlockBox,
                    _ => draw_type = DrawType::Normal
                }
            },
            Err(_) => todo!(),
        }

        /*
        precalculate mapping on texture atlas - but only if it's a block box

        this also will throw an error if there is no shape defined
        */

        let mut block_box_option: Option<BlockBox> = None;

        if matches!(draw_type, DrawType::BlockBox) {

            let lua_block_box: Result<Table, mlua::Error> = lua_table.get("block_box");

            // assign from lua
            let block_box_table: Table;

            match lua_block_box {
                Ok(lua_table) => block_box_table = lua_table,
                Err(error) => {
                    // if this gets hit something truly unspeakable has happened
                    panic!("NO BLOCK BOX WAS DEFINED FOR {}! ERROR: {}", block_name, error);
                },
            }

            // shove all the lua floats into a vector
            let mut block_box: Vec<f32> = Vec::new();

            for value in block_box_table.pairs::<Integer, f32>() {
                let (_, float_component) = value.unwrap();
                block_box.push(float_component);
            }

            block_box_option = Some(BlockBox::new(block_box));
            

            /*
            for i in block_textures.iter() {
                // println!("{}", i);
                let test = packer.get_frame(i).unwrap();

                println!("{:#?}", test.frame);
            }
            */
        }


        // calculate texture coordinates

        /*
        match block_box_option {
            Some(block_box) => {
                println!("this needs to do calculations on this thing{:?}", block_box.get());
            },
            None => (),
        }
        */

        /*
        it may seem like it's not good practice to precalculate textures per face

        but when you are working with maybe thousands of different blocks

        you would have to individually calculate this while generating a chunk regardless

        this trades extreme code complexity during runtime for up front slight memory cost

        */

        let mut mapping: Vec<AtlasTextureMap> = Vec::new();

        match draw_type {
            // nothing needs to be done
            DrawType::None => (),
            // a full block - nothing special is needed
            DrawType::Normal => {
                // println!("---- debugging {} ------", block_name.clone());
                // this will return an AtlasTextureMap per face
                let mut index = 0;
                for i in block_textures.iter() {
                    let current_mapping = calculate_atlas_location_normal(
                        atlas.width(), 
                        atlas.height(),
                        /*
                        the frame cannot be null or nullptr
                        this would have caused a crash earlier on
                        */
                        packer.get_frame(i).unwrap(),
                        block_rotations[index],
                        block_flips[index]
                    );

                    mapping.push(current_mapping);

                    index += 1;
                }
            },
            // very complex calculation - intakes block box and does conversions
            DrawType::BlockBox => {
                // println!("THIS BLOCK IS A BLOCK_BOX")
            },
        }


        bcs.register_block(
            block_mod,
            block_name,
            draw_type,
            block_textures,
            block_box_option,
            mapping
        )
    } 

    // texture atlas will always be id 1
    let value_test = mcs.new_texture_from_memory(atlas.as_rgba8().unwrap().to_owned());

    println!("TEXTURE ATLAS IS VALUE: {}", value_test);


    // begin iterating biome data

    // iterating crafter.biomes
    let biomes: Table = crafter.get("biomes").unwrap();

    for biome_option in biomes.pairs::<String, Table>() {

        let (biome_name, biome_table) = biome_option.unwrap();

        let game_mod: String = biome_table.get("mod").unwrap();

        let top_layer: String = biome_table.get("top_layer").unwrap();

        let top_layer_depth_table: Table = biome_table.get("top_layer_depth").unwrap();

        let top_layer_depth: LayerDepth = LayerDepth::new(
            top_layer_depth_table.get::<u8, u8>(1).unwrap(),
            top_layer_depth_table.get::<u8, u8>(2).unwrap()
        );


        let bottom_layer: String = biome_table.get("bottom_layer").unwrap();

        let bottom_layer_depth_table: Table = biome_table.get("bottom_layer_depth").unwrap();

        let bottom_layer_depth: LayerDepth = LayerDepth::new(
            bottom_layer_depth_table.get::<u8, u8>(1).unwrap(),
            bottom_layer_depth_table.get::<u8, u8>(2).unwrap()
        );


        let stone_layer: String = biome_table.get("stone_layer").unwrap();

        let bedrock_layer: String = biome_table.get("bedrock_layer").unwrap();


        let terrain_height_flux: u8 = biome_table.get("terrain_height_flux").unwrap();

        let caves: bool = biome_table.get("caves").unwrap();

        let cave_heat_table: Table = biome_table.get("cave_noise_params").unwrap();

        let cave_noise_params: NoiseParams = NoiseParams::new(
            cave_heat_table.get("heat_min").unwrap(),
            cave_heat_table.get("heat_max").unwrap(),
            cave_heat_table.get("scale").unwrap(),
            cave_heat_table.get("frequency").unwrap()
        );

        let rain: bool = biome_table.get("rain").unwrap();

        let snow: bool = biome_table.get("snow").unwrap();

        // process biome ores

        let lua_biome_ores_option: Result<Table, Error> = biome_table.get("ores");

        let biome_ores_option: Option<BiomeOres>;

        // lua checked everything so we can freely work with it
        match lua_biome_ores_option {
            Ok(biome_lua_table) => {
                let mut finished_biome_ore_definition = BiomeOres::new();

                for defined_ore_result in biome_lua_table.pairs::<String, Table>() {
                    let (ore_name, ore_lua_table) = defined_ore_result.unwrap();

                    let depth_table: Table = ore_lua_table.get("depth").unwrap();

                    let depth: LayerDepth = LayerDepth::new(
                        depth_table.get(1).unwrap(),
                        depth_table.get(2).unwrap()
                    );

                    let heat_table: Table = ore_lua_table.get("heat").unwrap();

                    let heat: NoiseParams = NoiseParams::new(
                        heat_table.get(1).unwrap(),
                        heat_table.get(2).unwrap(),
                        0.0,
                        0.0
                    );

                    let frequency: f32 = ore_lua_table.get("frequency").unwrap();

                    let scale: f32 = ore_lua_table.get("scale").unwrap();

                    finished_biome_ore_definition.register_ore(
                        bcs.get_id_of(ore_name),
                        depth,
                        heat,
                        frequency,
                        scale
                    );
                }
                biome_ores_option = Some(finished_biome_ore_definition);
            },
            Err(_) => biome_ores_option = None,
        }

        // getting biome noise parameters
        let lua_biome_heat_params: Table = biome_table.get("biome_noise_params").unwrap();

        let biome_noise_params: NoiseParams = NoiseParams::new(
            lua_biome_heat_params.get("heat_min").unwrap(),
            lua_biome_heat_params.get("heat_max").unwrap(),
            lua_biome_heat_params.get("scale").unwrap(),
            lua_biome_heat_params.get("frequency").unwrap()
        );



        gcs.register_biome(
            biome_name,
            biome_noise_params,
            terrain_height_flux,
            game_mod,
            bcs.get_id_of(top_layer),
            top_layer_depth,
            bcs.get_id_of(bottom_layer),
            bottom_layer_depth,
            bcs.get_id_of(stone_layer),
            bcs.get_id_of(bedrock_layer),
            biome_ores_option,
            caves,
            cave_noise_params,
            rain,
            snow
        );
    }


    println!("-------------- done -----------------");
}