use std::error::Error;
use image::{GenericImageView, Pixel, ImageBuffer, Rgb, DynamicImage};

/*
 I could add something like, copy def.csv and out.csv in out_dir
 */
pub fn create_coord_to_id_csv(bmp_path: String, def_path: String, wb_path: String, out_dir: String) -> Result<(), Box<dyn Error>> {
    let all_provinces = crate::game_types::read_provinces(def_path.clone(), wb_path.clone()).unwrap();

    let pdx_bmp = image::open(bmp_path).unwrap();
    let mut wtr = csv::Writer::from_path(out_dir.to_string() + "/coord_id_map.csv")?;
    wtr.write_record(&["X", "Y", "ID"])?;

    let mut prev_id: u16 = (pdx_bmp.height() + 2).try_into().unwrap();
    let mut prev_y: u32 = (all_provinces.len() + 2).try_into().unwrap();
    for (x, y, pixel) in pdx_bmp.pixels() {
        let col = pixel.to_rgb();

        for p in &all_provinces {

            if p.color_r == col[0] && p.color_g == col[1] && p.color_b == col[2] {
                if !(y == prev_y && p.id == prev_id){
                    wtr.write_record(&[x.to_string(), y.to_string(), p.id.to_string()])?;
                    prev_y = y;
                    prev_id = p.id;
                }
                break;
            }
        }
    }

    wtr.flush()?;

    Ok(())
}
