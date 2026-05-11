use crate::game_types::Province;
use crate::game_types::Nation;

use std::error::Error;
use std::io::Cursor;

use image::{ImageBuffer, Rgb, DynamicImage, ImageFormat};


#[derive(Clone)]
pub struct ImageOptions {
    pub show_subjects: bool,
    pub blend_subjects: bool,
    pub show_allies: bool,
    pub blend_allies: bool,
    pub blend_factor: f32,
    pub sub_overlord_col: bool,
    pub show_all: bool,
    pub continent: ContinentCoord,
}

#[derive(Clone)]
pub struct ContinentCoord(([u32; 2], [u32; 2]));

impl ContinentCoord {
    pub const ALL: ContinentCoord = ContinentCoord(([0, 0], [5632, 2048]));
    pub const NORTH_AMERICA: ContinentCoord = ContinentCoord(([0, 0],[2200, 1130]));
    pub const SOUTH_AMERICA: ContinentCoord = ContinentCoord(([1170, 930],[2500, 2048]));
    pub const EUROPE: ContinentCoord = ContinentCoord(([2330, 0],[3600, 950]));
    pub const AFRICA: ContinentCoord = ContinentCoord(([2280, 700],[3850, 2048]));
    pub const ASIA: ContinentCoord = ContinentCoord(([3280, 290],[5632, 1600]));
    pub const SE_ASIA_OCEANIA: ContinentCoord = ContinentCoord(([4200, 1080],[5632, 2048]));
    pub const INDIA_PERSIA: ContinentCoord = ContinentCoord(([3440, 780],[4370, 1385]));
    pub const SE_ASIA: ContinentCoord = ContinentCoord(([4100, 380],[5300, 1460]));
}

impl PartialEq for ContinentCoord {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for ContinentCoord {}


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
                    px_col = get_province_color_for_country(p, &country_tags, &country_tags_all, countries, opt.clone())?;
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

    let mut img_dyn: DynamicImage = DynamicImage::ImageRgb8(imgbuf);

    if opt.continent != ContinentCoord::ALL {
        let l = opt.continent.0.1[0] - opt.continent.0.0[0];
        let h = opt.continent.0.1[1] - opt.continent.0.0[1];
        img_dyn = img_dyn.crop_imm(opt.continent.0.0[0], opt.continent.0.0[1], l , h);
    }
    //img_dyn = img_dyn.resize(img_x * 2, img_y * 2, image::imageops::FilterType::Lanczos3);
    //let kernel: [f32; 9] = [1.0; 9];
    //img_dyn = img_dyn.filter3x3(&kernel);
    //img_dyn.save(opt.dest_path)?;
    let mut buffer = Cursor::new(Vec::new());
    let _ = img_dyn.write_to(&mut buffer, ImageFormat::Png);
    Ok(buffer.into_inner())
}

// TODO: i should add a color attribute to province, so that i only have to call this function once / it only does this search once per province
fn get_province_color_for_country(p: &Province, country_tags_all: &Vec<String>, country_tags: &Vec<String>, countries: &Vec<Nation>, opt: ImageOptions) -> Result<image::Rgb<u8>, Box<dyn Error>> {
    let mut px_col = image::Rgb([125, 125, 125]);
    let owner = p.owner.as_ref().unwrap();

    // TODO: the logic for show all is flawed
    if opt.show_all && !opt.sub_overlord_col {
        return Ok(image::Rgb([owner.color_r, owner.color_g, owner.color_b]));
    }

    if !country_tags_all.contains(&owner.tag) && !opt.show_all {
        return Ok(px_col);
    }

    if country_tags.contains(&owner.tag) {
        return Ok(image::Rgb([owner.color_r, owner.color_g, owner.color_b]));
    }

    // either a subject or a ally or not to be shown
    let mut lord_r = 0;
    let mut lord_g = 0;
    let mut lord_b = 0;
    let mut sub_r = 0;
    let mut sub_g = 0;
    let mut sub_b = 0;
    let mut is_ally = false;
    let mut is_subject = false;

    for n in countries {
        if n.tag == owner.tag {
            sub_r = owner.color_r;
            sub_g = owner.color_g;
            sub_b = owner.color_b;
        }
        if !country_tags.contains(&n.tag) && !opt.show_all {
            continue;
        }
        if opt.show_allies && !is_ally {
            for ally in &n.allies {
                if ally == &owner.tag {
                    is_ally = true;
                    lord_r = n.color_r;
                    lord_g = n.color_g;
                    lord_b = n.color_b;
                    break;
                }
            }
        }
        if opt.show_subjects && !is_subject {
            for sub in &n.subjects {
                if sub == &owner.tag {

                    if opt.sub_overlord_col {
                        return Ok(image::Rgb([n.color_r, n.color_g, n.color_b]));
                    }

                    is_subject = true;
                    lord_r = n.color_r;
                    lord_g = n.color_g;
                    lord_b = n.color_b;
                    break;
                }
            }
        }
    }

    if opt.show_all && !is_subject {
        return Ok(image::Rgb([sub_r, sub_g, sub_b]));
    }

    if is_ally {
        if opt.blend_allies {
            let red = blend_colors(lord_r, sub_r, opt.blend_factor);
            let green = blend_colors(lord_g, sub_g, opt.blend_factor);
            let blue = blend_colors(lord_b, sub_b, opt.blend_factor);
            px_col = image::Rgb([red, green, blue]);
        }
        else {
            px_col = image::Rgb([sub_r, sub_g, sub_b]);
        }
    }
    if is_subject {
        if opt.blend_subjects {
            let red = blend_colors(lord_r, sub_r, opt.blend_factor);
            let green = blend_colors(lord_g, sub_g, opt.blend_factor);
            let blue = blend_colors(lord_b, sub_b, opt.blend_factor);
            px_col = image::Rgb([red, green, blue]);
        }
        else {
            px_col = image::Rgb([sub_r, sub_g, sub_b]);
        }
    }

    Ok(px_col)
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
