use regex::Regex;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::error::Error;
use crate::game_types::Nation;
use crate::game_types::Province;

//pub mod game_types;

pub fn read_savegame(save_path: String, tag: String, provsAll: Vec<Province>) -> (Nation, Vec<Province>, Vec<Province>) {
    // insert tag to read
    //let re_prov = Regex::new(r"\ncountries=\{[\S\s]*\n\tBOH=\{[\s]*(?:human=yes)?[\s]*(?:was_player=yes)?[\s]*(?:has_set_government_name=yes)?[\s]*government_rank=(?<gov_rank>\d)[\S\s]*owned_provinces=\{[\s]*(?<provinces>(?:\d*[\s])*)[\s]*}").unwrap();
    let pattern = r"\ncountries=\{[\S\s]*?\n\t".to_string() + &tag + r"=\{[\s]*?(?:human=yes)?[\s]*?(?:pillaged_capital_state=\{[\S\s[^}]]*?})?[\s]*?(?:was_player=yes)?[\s]*(?:has_set_government_name=yes)?[\s]*government_rank=(?<gov_rank>\d)[\S\s]*?map_color=\{[\s]*(?<col>(?:\d*[\s])*)[\s]*}[\S\s]*?owned_provinces=\{[\s]*(?<provinces>(?:\d*[\s])*)[\s]*}";
    let re_prov = Regex::new(&pattern).unwrap();
    // this is not yet working
    //let re_col = Regex::new(r"[\t]MAJ={[\S\s]*country_color={[\s]*((?:\d*[ \n])*).*owned_provinces={[\s]*((?:\d*[ \n])*).*").unwrap();

    let mut country = Nation {
        id: 1,
        name: String::from("Bohemia"),
        tag: tag,
        color_r: 180,
        color_g: 15,
        color_b: 15,
        rank: 1, // Enum: Duchy, Kingom, Empire
        gov_type: String::from("Království"), // Empire, Kralvsti, Most Serene Republic, etc.
        religion: String::from("Hussite")
    };

    let mut f = File::open(save_path).unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap();
    let contents = String::from_utf8_lossy(&buffer);

    let Some(caps) = re_prov.captures(&contents) else { return (country, Vec::new(), provsAll) };

    let provs = &caps["provinces"].trim();
    let cols = &caps["col"].trim().split(" ").collect::<Vec<&str>>();;

    country.color_r = cols[0].parse::<u8>().unwrap();
    country.color_g = cols[1].parse::<u8>().unwrap();
    country.color_b = cols[2].parse::<u8>().unwrap();

    let mut provsVec = Vec::new();

    for p in provs.split(" ") {
        for pa in &provsAll {
            let pi = p.parse::<u16>().unwrap();
            if pa.id == pi {
                let pc = Province {
                    id: pi,
                    is_used: pa.is_used,
                    is_sea: pa.is_sea,
                    is_lake: pa.is_lake,
                    dev: pa.dev,
                    name: pa.name.clone(),
                    color_r: pa.color_r,
                    color_g: pa.color_g,
                    color_b: pa.color_b,
                    col: image::Rgb([pa.color_r, pa.color_g, pa.color_b]),
                    owner: Some(country.clone()),
                };
                provsVec.push(pc);
                break;
            }
        }
    }

    return (country, provsVec, provsAll)
}
