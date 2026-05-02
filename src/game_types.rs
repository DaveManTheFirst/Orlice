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
