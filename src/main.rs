use crate::game_types::Nation;
use crate::game_types::Province;

use std::time::Instant;
use std::error::Error;

use image::{GenericImageView, Pixel, ImageBuffer, Rgb, DynamicImage};

pub mod game_types;
pub mod savegame_reader;
pub mod map_converter;

fn main() {
    //let cv = crate::game_types::get_boh_prov();
    let mut st = Instant::now();
    let def_path = String::from("/mnt/MassenData/Programmieren/Rust/Orlice/resources/map/definition.csv");
    let wb_path = String::from("/mnt/MassenData/Programmieren/Rust/Orlice/resources/map/water_bodies.csv");
    let out_path = String::from("/mnt/MassenData/Programmieren/Rust/Orlice/resources/generation");
    let bmp_path = String::from("/mnt/MassenData/Programmieren/Rust/Orlice/resources/map/provinces.bmp");
    let save_path = String::from("/mnt/MassenData/Programmieren/Rust/Orlice/resources/savegame/gamestate");
    let map_path = String::from("/mnt/MassenData/Programmieren/Rust/Orlice/resources/generation/coord_id_map.csv");

    let all_provinces_result = crate::game_types::read_provinces(def_path.clone(), wb_path.clone());

    println!("Read Provinces: {}", st.elapsed().as_micros());
    st = Instant::now();
    let all_provinces = match all_provinces_result {
        Ok(pv) => pv,
        Err(error) => panic!("Problem opening the definition or wb file: {error:?}"),
    };

    let (c_boh, mut boh_provinces, all_provinces) = savegame_reader::read_savegame(save_path.clone(), String::from("BOH"), all_provinces);
    let (c_ita, mut ita_provinces, all_provinces) = savegame_reader::read_savegame(save_path.clone(), String::from("ITA"), all_provinces);
    let (c_egy, mut egy_provinces, all_provinces) = savegame_reader::read_savegame(save_path.clone(), String::from("EGY"), all_provinces);

    println!("Read savegame: {}", st.elapsed().as_micros());
    st = Instant::now();
    let mut country_provinces = Vec::new();

    country_provinces.append(&mut boh_provinces);
    country_provinces.append(&mut ita_provinces);
    country_provinces.append(&mut egy_provinces);

    //crate::map_converter::create_coord_to_id_csv(bmp_path.clone(), def_path.clone(), wb_path.clone(), out_path.clone());
    let out = c_boh.get_title();
    println!("Title: {}, P1: {}, P2: {}", out, country_provinces[2].name, all_provinces[1265].name);
    make_image(all_provinces, country_provinces, map_path.clone(), bmp_path.clone());
    println!("Make Image: {}", st.elapsed().as_micros());
}

fn make_image(all_provinces: Vec<Province>, country_provinces: Vec<Province>, map_path: String, bmp_path: String) -> Result<(), Box<dyn Error>> {
    let pdx_bmp = image::open(bmp_path).unwrap();

    let mut rdr = csv::ReaderBuilder::new()
    .delimiter(b',')
    .from_path(map_path);

    let col_sea = image::Rgb([25, 100, 160]);
    let col_gray = image::Rgb([125, 125, 125]);
    let col_pink = image::Rgb([204, 0, 100]);

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
        let mut prov_of_country = false;
        for p in &country_provinces {
            //let p_col = image::Rgb([p.color_r, p.color_g, p.color_b]);
            if i32::from(p.id) == prev_id {
                px_col = image::Rgb([p.owner.as_ref().unwrap().color_r, p.owner.as_ref().unwrap().color_g, p.owner.as_ref().unwrap().color_b]);
                prov_of_country = true;
                break;
            }
        }

        if !prov_of_country {
            for p in &all_provinces {
                //let p_col = image::Rgb([p.color_r, p.color_g, p.color_b]);
                if i32::from(p.id) == prev_id {
                    if p.is_used == false {
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
                if p.is_used == false {
                    // idk what provinces are unused or not x
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
        for p in &country_provinces {
            //let p_col = image::Rgb([p.color_r, p.color_g, p.color_b]);
            if i32::from(p.id) == prev_id {
                let p_buf = imgbuf.get_pixel_mut(i.try_into().unwrap(), img_y-1);
                *p_buf = image::Rgb([p.owner.as_ref().unwrap().color_r, p.owner.as_ref().unwrap().color_g, p.owner.as_ref().unwrap().color_b]);
                break;
            }
        }
    }

    println!("Overall: {}", st.elapsed().as_micros());
    println!("Count: {}", cnt);
    println!("Avg: {}", st.elapsed().as_micros() / cnt);

    let mut img_dyn: DynamicImage = DynamicImage::ImageRgb8(imgbuf);
    //img_dyn = img_dyn.resize(img_x * 2, img_y * 2, image::imageops::FilterType::Lanczos3);
    //let kernel: [f32; 9] = [1.0; 9];
    //img_dyn = img_dyn.filter3x3(&kernel);
    img_dyn.save("/mnt/MassenData/Programmieren/Rust/Orlice/test.png")?;

    Ok(())
}

