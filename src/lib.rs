extern crate wasm_bindgen;

use crate::game_types::Nation;
use crate::image_generator::ImageOptions;
use crate::image_generator::ContinentCoord;

use wasm_bindgen::prelude::*;

pub mod game_types;
pub mod map_converter;
pub mod eu4_save_parser;
pub mod image_generator;
pub mod eu4_province_reader;

// to see panics in the console
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

// TODO: Return Only those nations that have land
#[wasm_bindgen]
pub fn get_nation_tags(save_string: &[u8]) -> Result<Vec<String>, JsValue> {
    let sv = match crate::eu4_save_parser::parse_savegame(save_string) {
        Ok(r) => r,
        Err(error) => return Err(JsValue::from_str(&format!("Problem parsing save game: {error:?}"))),
    };

    let tags = match crate::game_types::get_tags_from_savevalues(&sv) {
        Ok(t) => t,
        Err(error) => return Err(JsValue::from_str(&format!("Problem retrieving country tags: {error:?}"))),
    };

    Ok(tags)
}

#[wasm_bindgen]
pub async fn generate_image(save_string: &[u8], country_tags: Vec<String>, image_options: Vec<u8>) -> Result<Vec<u8>, JsValue> {
    let def_path = String::from("/data/definition.csv");
    let wb_path = String::from("/data/water_bodies.csv");
    let map_path = String::from("/data/coord_id_map.csv");

    let sv = match crate::eu4_save_parser::parse_savegame(save_string) {
            Ok(r) => r,
            Err(error) => return Err(JsValue::from_str(&format!("Problem parsing save game: {error:?}"))),
    };

    let mut all_provinces = match crate::eu4_province_reader::read_provinces(def_path.clone(), wb_path.clone()).await {
        Ok(pv) => pv,
        Err(error) => return Err(JsValue::from_str(&format!("Problem opening the definition or wb file: {error:?}"))),
    };

    let nations : Vec<Nation> = match crate::game_types::from_savevalues(&sv, &mut all_provinces) {
        Ok(n) => n,
        Err(error) => return Err(JsValue::from_str(&format!("Problem getting game objects from save values: {error:?}"))),
    };

    let continent: ContinentCoord = match image_options[6] {
        0 => ContinentCoord::ALL,
        1 => ContinentCoord::NORTH_AMERICA,
        2 => ContinentCoord::SOUTH_AMERICA,
        3 => ContinentCoord::EUROPE,
        4 => ContinentCoord::AFRICA,
        5 => ContinentCoord::ASIA,
        6 => ContinentCoord::SE_ASIA_OCEANIA,
        7 => ContinentCoord::INDIA_PERSIA,
        8 => ContinentCoord::SE_ASIA,
        _ => ContinentCoord::ALL,
    };

    // we can recieve this in order
    let opts = ImageOptions {
        show_subjects: image_options[2] > 0,
        blend_subjects: image_options[3] > 0,
        show_allies: image_options[0] > 0,
        blend_allies: image_options[1] > 0,
        blend_factor: 0.25,
        sub_overlord_col: image_options[4] > 0,
        show_all: image_options[5] > 0,
        continent: continent,
    };

    //crate::map_converter::create_coord_to_id_csv(bmp_path.clone(), def_path.clone(), wb_path.clone(), out_path.clone());
    let i = match image_generator::make_image(&all_provinces, country_tags, &nations, opts, map_path.clone()).await {
        Ok(m) => m,
        Err(error) => return Err(JsValue::from_str(&format!("Problem creating the image: {error:?}"))),
    };
    Ok(i)
}
