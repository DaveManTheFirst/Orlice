extern crate wasm_bindgen;

use crate::game_types::Province;
use crate::game_types::Nation;
use crate::image_generator::ImageOptions;
use crate::eu4_save_parser::SaveValue;

use wasm_bindgen::prelude::*;
use std::error::Error;

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

#[wasm_bindgen]
pub async fn generate_image(save_string: &[u8]) -> Result<Vec<u8>, JsValue> {
    let def_path = String::from("/data/definition.csv");
    let wb_path = String::from("/data/water_bodies.csv");
    let map_path = String::from("/data/coord_id_map.csv");
    let save_path = String::from("/mnt/MassenData/Programmieren/Rust/Orlice/resources/savegame/gamestate");
    let dest_path = String::from("/mnt/MassenData/Programmieren/Rust/Orlice/misc/test.png");

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

    let mut country_tags = vec![String::from("EGY")]; /*, String::from("ITA"), String::from("EGY"), String::from("SCA"), String::from("GBR")]; */

    let opts = ImageOptions {
        show_subjects: false,
        blend_subjects: false,
        show_allies: true,
        blend_allies: true,
        blend_factor: 0.25,
        dest_path: dest_path.clone(),
    };

    //crate::map_converter::create_coord_to_id_csv(bmp_path.clone(), def_path.clone(), wb_path.clone(), out_path.clone());
    let i = match image_generator::make_image(&all_provinces, country_tags, &nations, opts, map_path.clone()).await {
        Ok(pv) => pv,
        Err(error) => return Err(JsValue::from_str(&format!("Problem creating the image: {error:?}"))),
    };
    Ok(i)
}
