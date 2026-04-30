use std::{error::Error, fs::File};
use encoding_rs::WINDOWS_1252; // ISO 8859-1
use encoding_rs_io::DecodeReaderBytesBuilder;
use image::Rgb;

#[derive(Clone)]
pub struct Nation
{
    pub id: u32,
    pub name: String,
    pub tag: String,
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
    pub rank: u8, // Enum: Duchy, Kingom, Empire
    pub gov_type: String, // Empire, Kralvsti, Most Serene Republic, etc.
    pub religion: String,
}

pub struct Province
{
    pub id: u16,
    pub name: String,
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
    pub dev: u16,
    pub is_used: bool,
    pub is_sea: bool,
    pub is_lake: bool,
    pub col: Rgb<u8>,
    pub owner: Option<Nation>,
}

impl Nation {
    /*pub fn clone(&self) -> Nation {
        return Nation {
            id: self.id,
            name: self.name.clone(),
            tag: self.tag.clone(),
            color_r: self.color_r,
            color_g: self.color_g,
            color_b: self.color_b,
            rank: self.rank,
            gov_type: self.gov_type.clone(),
            religion: self.religion.clone(),
        };
    }*/

    pub fn get_title(&self) -> String {
        let mut gt: String = self.gov_type.clone();
        gt.push_str(&String::from(" of "));
        gt.push_str(&self.name);
        return gt;
    }
}

pub fn read_provinces(def_path: String, wb_path: String) -> Result<Vec<Province>, Box<dyn Error>> {
    let mut pv = Vec::new();

    let file = DecodeReaderBytesBuilder::new()
    .encoding(Some(WINDOWS_1252))
    .build(File::open(def_path).unwrap());

    let mut rdr = csv::ReaderBuilder::new()
    .delimiter(b';')
    .from_reader(file);
    for result in rdr.records() {
        // The iterator yields Result<StringRecord, Error>, so we check the
        // error here..
        let record = result?;
        let p = Province {
            id: record[0].parse::<u16>().unwrap(),
            name: record[4].to_string(),
            color_r: record[1].parse::<u8>().unwrap(),
            color_g: record[2].parse::<u8>().unwrap(),
            color_b: record[3].parse::<u8>().unwrap(),
            dev: 12,
            is_used: if record[5].to_string() == String::from("x") {true} else {false},
            is_sea: false,
            is_lake: false,
            col: image::Rgb([record[1].parse::<u8>().unwrap(), record[2].parse::<u8>().unwrap(), record[3].parse::<u8>().unwrap()]),
            owner: None,
        };
        pv.push(p);
    }

    // read water bodies
    let file_water = DecodeReaderBytesBuilder::new()
    .encoding(Some(WINDOWS_1252))
    .build(File::open(wb_path).unwrap());

    let mut rdr_water = csv::ReaderBuilder::new()
    .delimiter(b';')
    .from_reader(file_water);

    for r in rdr_water.records() {
        let rec = r?;

        let id = rec[0].parse::<u16>().unwrap();

        for p in &mut pv
        {
            if p.id == id {
                p.is_sea = if rec[1].to_string() == String::from("true") {true} else {false};
                p.is_lake = if rec[2].to_string() == String::from("true") {true} else {false};
            }
        }
    }
    Ok(pv)
}

pub fn get_boh_prov() -> Vec<Province> {
    let mut bv = Vec::new();
    let ege = Province {
        id: 2967,
        name: String::from("Eger"),
        color_r: 32,
        color_g: 251,
        color_b: 212,
        dev: 12,
        is_used: true,
        is_sea: false,
        is_lake: false,
        col: image::Rgb([32, 251, 212]),
        owner: None,
    };
    bv.push(ege);
    let bud = Province {
        id: 2968,
        name: String::from("Budejovice"),
        color_r: 88,
        color_g: 163,
        color_b: 215,
        dev: 14,
        is_used: true,
        is_sea: false,
        is_lake: false,
        col: image::Rgb([88, 163, 215]),
        owner: None,
    };
    bv.push(bud);
    let hrd = Province {
        id: 2970,
        name: String::from("Hradecko"),
        color_r: 255,
        color_g: 239,
        color_b: 163,
        dev: 13,
        is_used: true,
        is_sea: false,
        is_lake: false,
        col: image::Rgb([255, 239, 163]),
        owner: None,
    };
    bv.push(hrd);
    let pra = Province {
        id: 266,
        name: String::from("Praha"),
        color_r: 38,
        color_g: 52,
        color_b: 64,
        dev: 25,
        is_used: true,
        is_sea: false,
        is_lake: false,
        col: image::Rgb([38, 52, 64]),
        owner: None,
    };
    bv.push(pra);
    let plz = Province {
        id: 267,
        name: String::from("Plzen"),
        color_r: 166,
        color_g: 54,
        color_b: 128,
        dev: 14,
        is_used: true,
        is_sea: false,
        is_lake: false,
        col: image::Rgb([166, 54, 128]),
        owner: None,
    };
    bv.push(plz);
    let ruh = Province {
        id: 1771,
        name: String::from("Erz"),
        color_r: 198,
        color_g: 126,
        color_b: 173,
        dev: 11,
        is_used: true,
        is_sea: false,
        is_lake: false,
        col: image::Rgb([198, 126, 173]),
        owner: None,
    };
    bv.push(ruh);
    let brn = Province {
        id: 265,
        name: String::from("Brno"),
        color_r: 165,
        color_g: 50,
        color_b: 0,
        dev: 12,
        is_used: true,
        is_sea: false,
        is_lake: false,
        col: image::Rgb([165, 50, 0]),
        owner: None,
    };
    bv.push(brn);
    let ost = Province {
        id: 4726,
        name: String::from("Ostrava"),
        color_r: 14,
        color_g: 8,
        color_b: 38,
        dev: 15,
        is_used: true,
        is_sea: false,
        is_lake: false,
        col: image::Rgb([14, 8, 38]),
        owner: None,
    };
    bv.push(ost);
    let jih = Province {
        id: 4725,
        name: String::from("Jindrichuv Hradec"),
        color_r: 182,
        color_g: 22,
        color_b: 144,
        dev: 11,
        is_used: true,
        is_sea: false,
        is_lake: false,
        col: image::Rgb([182, 22, 144]),
        owner: None,
    };
    bv.push(jih);
    let par = Province {
        id: 4724,
        name: String::from("Pardubice"),
        color_r: 28,
        color_g: 140,
        color_b: 117,
        dev: 16,
        is_used: true,
        is_sea: false,
        is_lake: false,
        col: image::Rgb([28, 140, 117]),
        owner: None,
    };
    bv.push(par);
    let olm = Province {
        id: 4724,
        name: String::from("Olomouc"),
        color_r: 48,
        color_g: 237,
        color_b: 244,
        dev: 13,
        is_used: true,
        is_sea: false,
        is_lake: false,
        col: image::Rgb([48, 237, 244]),
        owner: None,
    };
    bv.push(olm);
    return bv;
}
