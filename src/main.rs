use crate::game_types::Province;
use crate::eu4_save_parser::SaveValue;

use std::time::Instant;
use std::error::Error;

use image::{GenericImageView, ImageBuffer, Rgb, DynamicImage};

pub mod game_types;
pub mod savegame_reader;
pub mod map_converter;
pub mod eu4_save_parser;

fn main() {
    let mut st = Instant::now();
    let def_path = String::from("/mnt/MassenData/Programmieren/Rust/Orlice/resources/map/definition.csv");
    let wb_path = String::from("/mnt/MassenData/Programmieren/Rust/Orlice/resources/map/water_bodies.csv");
    let out_path = String::from("/mnt/MassenData/Programmieren/Rust/Orlice/resources/generation");
    let bmp_path = String::from("/mnt/MassenData/Programmieren/Rust/Orlice/resources/map/provinces.bmp");
    let save_path = String::from("/mnt/MassenData/Programmieren/Rust/Orlice/resources/savegame/gamestate");
    let map_path = String::from("/mnt/MassenData/Programmieren/Rust/Orlice/resources/generation/coord_id_map.csv");

    let sv = match crate::eu4_save_parser::parse_savegame(save_path) {
            Ok(r) => r,
            Err(error) => panic!("Problem parsing save game: {error:?}"),
    };
    println!("Parse Save: {}", st.elapsed().as_micros());
    st = Instant::now();

    let mut all_provinces = match crate::game_types::read_provinces(def_path.clone(), wb_path.clone()) {
        Ok(pv) => pv,
        Err(error) => panic!("Problem opening the definition or wb file: {error:?}"),
    };
    println!("Read Provinces: {}", st.elapsed().as_micros());
    st = Instant::now();

    let nations = match crate::game_types::from_savevalues(&sv, &mut all_provinces) {
        Ok(n) => n,
        Err(error) => panic!("Problem getting game objects from save values: {error:?}"),
    };
    println!("Read & Assign Countries: {}", st.elapsed().as_micros());
    st = Instant::now();

    let mut country_tags = vec![String::from("BOH"), String::from("ITA"), String::from("EGY"), String::from("SCA"), String::from("GBR")];

    println!("Assign Countries: {}", st.elapsed().as_micros());
    st = Instant::now();

    //crate::map_converter::create_coord_to_id_csv(bmp_path.clone(), def_path.clone(), wb_path.clone(), out_path.clone());
    let _ = make_image(all_provinces, country_tags, map_path.clone(), bmp_path.clone());
    println!("Make Image: {}", st.elapsed().as_micros());
}

fn print_sv(sv: &SaveValue, tabs: u32) {
    match sv {
        SaveValue::SaveNull => { print_with_tab("Empty", tabs); }
        SaveValue::SaveBool(b) => { print_with_tab(&format!("Bool: {b:?}!"), tabs); }
        SaveValue::SaveNumber(i) => { print_with_tab(&format!("Number: {i:?}!"), tabs); }
        SaveValue::SaveFloat(f) => { print_with_tab(&format!("Float: {f:?}!"), tabs); }
        SaveValue::SaveDate(d) => { print_with_tab(&format!("Date: {d:?}!"), tabs); }
        SaveValue::SaveString(s)  => { print_with_tab(&format!("String: {s:?}!"), tabs); }
        SaveValue::SaveArray(v) => { print_with_tab(&format!("ARRAY!"), tabs); print_arr(&v, tabs + 1); }
        SaveValue::SaveObject(s, b) => { print_with_tab_no_lb(&format!("Object: {s:?}=!"), tabs); print_sv(&b, tabs); }
    }
}

fn print_arr(arr: &Vec<Box<SaveValue>>, tabs: u32) {
    for s in arr.iter() {
        print_sv(&s, 0);
    }
    println!("ARRAY ENDE!");
}

fn print_with_tab(s: &str, t: u32)
{
    let mut out = String::new();
    let mut i = 0;
    while i < t {
        out.push_str("\t");
        i += 1;
    }
    out.push_str(s);
    println!("{}", out);
}

fn print_with_tab_no_lb(s: &str, t: u32)
{
    let mut out = String::new();
    let mut i = 0;
    while i < t {
        out.push_str("\t");
        i += 1;
    }
    out.push_str(s);
    print!("{}", out);
}

fn make_image(all_provinces: Vec<Province>, country_tags: Vec<String>, map_path: String, bmp_path: String) -> Result<(), Box<dyn Error>> {
    let pdx_bmp = image::open(bmp_path).unwrap();

    let rdr = csv::ReaderBuilder::new()
    .delimiter(b',')
    .from_path(map_path);

    let col_sea = image::Rgb([25, 100, 160]);
    let col_gray = image::Rgb([125, 125, 125]);

    let img_x = pdx_bmp.width();
    let img_y = pdx_bmp.height();
    let mut imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img_x, img_y);

    let st = Instant::now();
    let mut cnt = 0;

    let mut prev_id : i32 = -1;
    let mut prev_x : u32 = 0;
    let mut prev_y : u32 = 0;
    for result in rdr?.records() {
        let r = result?;
        let x = r[0].parse::<u32>().unwrap();
        let y = r[1].parse::<u32>().unwrap();
        let id = r[2].parse::<i32>().unwrap();

        if prev_id == -1 {
            prev_id = id;
            prev_x = x;
            prev_y = y;
            continue;
        }

        let mut x_adjusted = x;

        if x == 0 {
            x_adjusted = img_x - 1;
        }

        let mut px_col = col_gray;

        for p in &all_provinces {
            if i32::from(p.id) == prev_id {
                let is_country : bool = match p.owner.as_ref() {
                    Some(_) => true,
                    None => false,
                };
                if is_country {
                    if country_tags.contains(&p.owner.as_ref().unwrap().tag) {
                        px_col = image::Rgb([p.owner.as_ref().unwrap().color_r, p.owner.as_ref().unwrap().color_g, p.owner.as_ref().unwrap().color_b]);
                    }
                }
                else if p.is_used == false {
                    // idk what provinces are unused or not x
                    px_col = col_gray;
                }
                else if p.is_lake || p.is_sea {
                    px_col = col_sea;
                } else {
                    px_col = col_gray;
                }
                break;
            }
        }

        for i in prev_x..x_adjusted{
            imgbuf.put_pixel(i, prev_y, px_col);
        }
        cnt += 1;


        prev_id = id;
        prev_x = x;
        prev_y = y;
    }

    for i in prev_x..img_x{
        for p in &all_provinces {
            if i32::from(p.id) == prev_id {
                let p_buf = imgbuf.get_pixel_mut(i.try_into().unwrap(), img_y-1);
                let is_country : bool = match p.owner.as_ref() {
                    Some(_) => true,
                    None => false,
                };
                if is_country {
                    let own = p.owner.as_ref().unwrap();
                    if country_tags.contains(&own.tag) {
                        *p_buf = image::Rgb([own.color_r, own.color_g, own.color_b]);
                    }
                } else if p.is_used == false {
                    *p_buf = col_gray;
                }
                else if p.is_lake || p.is_sea {
                    *p_buf = col_sea;
                } else {
                    *p_buf = col_gray;
                }
                break;
            }
        }
    }

    println!("Overall: {}", st.elapsed().as_micros());
    println!("Count: {}", cnt);
    println!("Avg: {}", st.elapsed().as_micros() / cnt);

    let img_dyn: DynamicImage = DynamicImage::ImageRgb8(imgbuf);
    //img_dyn = img_dyn.resize(img_x * 2, img_y * 2, image::imageops::FilterType::Lanczos3);
    //let kernel: [f32; 9] = [1.0; 9];
    //img_dyn = img_dyn.filter3x3(&kernel);
    img_dyn.save("/mnt/MassenData/Programmieren/Rust/Orlice/misc/test.png")?;

    Ok(())
}

