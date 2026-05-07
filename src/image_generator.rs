use crate::game_types::Province;
use crate::game_types::Nation;

use std::error::Error;
use std::io::Cursor;

use image::{ImageBuffer, Rgb, DynamicImage, ImageFormat};


pub struct ImageOptions {
    pub show_subjects: bool,
    pub blend_subjects: bool,
    pub show_allies: bool,
    pub blend_allies: bool,
    pub blend_factor: f32,
}

pub async fn make_image(all_provinces: &Vec<Province>, country_tags: Vec<String>, countries: &Vec<Nation>, opt: ImageOptions, map_path: String) -> Result<Vec<u8>, Box<dyn Error>> {
    let resp = gloo_net::http::Request::get(&map_path).send().await?;
    //let csv_str = String::from_utf8_lossy(&resp.text().await?);
    let csv_str = resp.text().await?;

    let mut rdr = csv::ReaderBuilder::new()
    .delimiter(b',')
    .from_reader(csv_str.as_bytes());

    let col_sea = image::Rgb([25, 100, 160]);
    let col_gray = image::Rgb([125, 125, 125]);

    // embedd this in the csv also, but for now hardcode
    let img_x = 5632;
    let img_y = 2048;
    let mut imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img_x, img_y);

    // build a country_tags to include allies and show_subjects
    let mut country_tags_all : Vec<String> = Vec::new();
    for n in countries {
        if country_tags.contains(&n.tag) {
            if opt.show_allies {
                for ally in &n.allies {
                    if !country_tags_all.contains(&ally) {
                        country_tags_all.push(ally.clone());
                    }
                }
            }
            if opt.show_subjects {
                for sub in &n.subjects {
                    if !country_tags_all.contains(&sub) {
                        country_tags_all.push(sub.clone());
                    }
                }
            }
        }
    }

    // add original guys
    for country_original in &country_tags {
        if !country_tags_all.contains(country_original) {
            country_tags_all.push(country_original.clone());
        }
    }

    let mut prev_id : i32 = -1;
    let mut prev_x : u32 = 0;
    let mut prev_y : u32 = 0;
    for result in rdr.records() {
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

        for p in all_provinces {
            if i32::from(p.id) == prev_id {
                let is_country : bool = match p.owner.as_ref() {
                    Some(_) => true,
                    None => false,
                };
                if is_country {
                    let owner = p.owner.as_ref().unwrap();
                    if country_tags_all.contains(&owner.tag) {
                        if country_tags.contains(&owner.tag) {
                            px_col = image::Rgb([owner.color_r, owner.color_g, owner.color_b]);
                        }
                        else {
                            let mut col_owner_r = 0;
                            let mut col_owner_g = 0;
                            let mut col_owner_b = 0;
                            let mut col_other_r = 0;
                            let mut col_other_g = 0;
                            let mut col_other_b = 0;
                            let mut is_ally = false;
                            let mut is_subject = false;
                            // gotta be ally or subject
                            for n in countries {
                                if n.tag == owner.tag {
                                    col_other_r = owner.color_r;
                                    col_other_g = owner.color_g;
                                    col_other_b = owner.color_b;
                                }
                                if !country_tags.contains(&n.tag) {
                                    continue;
                                }
                                if opt.show_allies && !is_ally {
                                    for ally in &n.allies {
                                        if ally == &owner.tag {
                                            is_ally = true;
                                            col_owner_r = n.color_r;
                                            col_owner_g = n.color_g;
                                            col_owner_b = n.color_b;
                                            break;
                                        }
                                    }
                                }
                                if opt.show_subjects && !is_subject {
                                    for sub in &n.subjects {
                                        if sub == &owner.tag {
                                            is_subject = true;
                                            col_owner_r = n.color_r;
                                            col_owner_g = n.color_g;
                                            col_owner_b = n.color_b;
                                            break;
                                        }
                                    }
                                }
                            }
                            if is_ally {
                                if opt.blend_allies {
                                    let red = blend_colors(col_owner_r, col_other_r, opt.blend_factor);
                                    let green = blend_colors(col_owner_g, col_other_g, opt.blend_factor);
                                    let blue = blend_colors(col_owner_b, col_other_b, opt.blend_factor);
                                    px_col = image::Rgb([red, green, blue]);
                                }
                                else {
                                    px_col = image::Rgb([col_other_r, col_other_g, col_other_b]);
                                }
                            }
                            if is_subject {
                                if opt.blend_subjects {
                                    let red = blend_colors(col_owner_r, col_other_r, opt.blend_factor);
                                    let green = blend_colors(col_owner_g, col_other_g, opt.blend_factor);
                                    let blue = blend_colors(col_owner_b, col_other_b, opt.blend_factor);
                                    px_col = image::Rgb([red, green, blue]);
                                }
                                else {
                                    px_col = image::Rgb([col_other_r, col_other_g, col_other_b]);
                                }
                            }
                        }
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


        prev_id = id;
        prev_x = x;
        prev_y = y;
    }

    for i in prev_x..img_x{
        for p in all_provinces {
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

    let img_dyn: DynamicImage = DynamicImage::ImageRgb8(imgbuf);
    //img_dyn = img_dyn.resize(img_x * 2, img_y * 2, image::imageops::FilterType::Lanczos3);
    //let kernel: [f32; 9] = [1.0; 9];
    //img_dyn = img_dyn.filter3x3(&kernel);
    //img_dyn.save(opt.dest_path)?;
    let mut buffer = Cursor::new(Vec::new());
    let _ = img_dyn.write_to(&mut buffer, ImageFormat::Png);
    Ok(buffer.into_inner())
}

fn blend_colors(original: u8, sub: u8, factor: f32) -> u8 {
    let mut ret = original;
    let fraction = (sub as f32 * factor) as u8;
    if original >= fraction {
        ret -= fraction;
    }
    else {
        if 255 - fraction < ret {
            ret = 255;
        }
        else {
            ret += fraction;
        }
    }
    ret as u8
}
