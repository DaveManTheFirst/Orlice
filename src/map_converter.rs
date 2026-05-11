use std::error::Error;
use image::{GenericImageView, Pixel};

/*
 I could add something like: copy def.csv, wb.csv and out.csv in out_dir
 */
pub async fn create_coord_to_id_csv(bmp_path: String, def_path: String, wb_path: String, out_dir: String) -> Result<(), Box<dyn Error>> {
    let all_provinces = crate::eu4_province_reader::read_provinces(def_path.clone(), wb_path.clone()).await?;

    let pdx_bmp = image::open(bmp_path)?;
    let mut wtr = csv::Writer::from_path(out_dir.to_string() + "/coord_id_map.csv")?;
    wtr.write_record(&["X", "Y", "ID"])?;

    let mut prev_id: u16 = (pdx_bmp.height() + 2).try_into()?;
    let mut prev_y: u32 = (all_provinces.len() + 2).try_into()?;
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
