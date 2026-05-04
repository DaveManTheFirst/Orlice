use std::{error::Error, fs::File};
use encoding_rs::WINDOWS_1252; // ISO 8859-1
use encoding_rs_io::DecodeReaderBytesBuilder;
use image::Rgb;

use crate::eu4_save_parser::SaveValue;

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
    pub allies: Vec<String>,
    pub subjects: Vec<String>,
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

pub fn from_savevalues(sv: &Vec<Box<SaveValue>>, pv: &mut Vec<Province>) -> Result<Vec<Nation>, Box<dyn Error>> {

    let mut countries : Vec<Nation> = Vec::new();

    for sa in sv {
        match sa.as_ref() {
            SaveValue::SaveObject(v, u) => {
                if *v == String::from("countries") {
                    match u.as_ref() {
                        SaveValue::SaveArray(a) => {
                            for ca in a {
                                match ca.as_ref() {
                                    SaveValue::SaveObject(ta, inf) => {
                                        match inf.as_ref() {
                                            SaveValue::SaveArray(info_arr) => {
                                                let mut c = Nation {
                                                    id: 1,
                                                    name: ta.clone(),
                                                    tag: ta.clone(),
                                                    color_r: 0,
                                                    color_g: 0,
                                                    color_b: 0,
                                                    rank: 1, // Enum: Duchy, Kingom, Empire
                                                    gov_type: String::from("Království"), // Empire, Kralvsti, Most Serene Republic, etc.
                                                    religion: String::from("Hussite"),
                                                    allies: Vec::new(),
                                                    subjects: Vec::new(),
                                                };

                                                for info_o in info_arr {
                                                    match info_o.as_ref() {
                                                        SaveValue::SaveObject(name, val) => {
                                                            if *name == String::from("government_name") {
                                                                match val.as_ref() {
                                                                    SaveValue::SaveString(s) => { c.gov_type = s.clone(); },
                                                                    _ => continue,
                                                                }
                                                            } else if *name == String::from("government_rank") {
                                                                match val.as_ref() {
                                                                    SaveValue::SaveString(s) => { c.gov_type = s.clone(); },
                                                                    _ => continue,
                                                                }
                                                            } else if *name == String::from("religion") {
                                                                match val.as_ref() {
                                                                    SaveValue::SaveString(s) => { c.religion= s.clone(); },
                                                                    _ => continue,
                                                                }
                                                            } else if *name == String::from("colors") {
                                                                match val.as_ref() {
                                                                    SaveValue::SaveArray(s) => {
                                                                        for colcat in s {
                                                                            match colcat.as_ref() {
                                                                                SaveValue::SaveObject(ccname, carr) => {
                                                                                    if *ccname == String::from("map_color") {
                                                                                        match carr.as_ref() {
                                                                                            SaveValue::SaveArray(cols) => {
                                                                                                let mut cnt = 0;
                                                                                                for colval in cols {
                                                                                                    //println!("{}", c.tag);
                                                                                                    match colval.as_ref() {
                                                                                                        SaveValue::SaveNumber(colnum) => {
                                                                                                            if cnt == 0 {
                                                                                                                c.color_r = *colnum as u8;
                                                                                                            } else if cnt == 1 {
                                                                                                                c.color_g = *colnum as u8;
                                                                                                            } else {
                                                                                                                c.color_b = *colnum as u8;
                                                                                                            }
                                                                                                            cnt += 1;
                                                                                                        },
                                                                                                        _ => continue,
                                                                                                    }
                                                                                                }
                                                                                            },
                                                                                            _ => continue,
                                                                                        }
                                                                                    }
                                                                                },
                                                                                _ => continue,
                                                                            }
                                                                        }
                                                                    },
                                                                    _ => continue,
                                                                }
                                                            } else if *name == String::from("owned_provinces") {
                                                                match val.as_ref() {
                                                                    SaveValue::SaveArray(opa) => {
                                                                        for opp in opa {
                                                                            match opp.as_ref() {
                                                                                SaveValue::SaveNumber(opn) => {
                                                                                    for prov in &mut *pv {
                                                                                        if prov.id == (*opn as u16) {
                                                                                            prov.owner = Some(c.clone());
                                                                                        }
                                                                                    }
                                                                                },
                                                                                _ => continue,
                                                                            }
                                                                        }
                                                                    },
                                                                    _ => continue,
                                                                }
                                                            }
                                                            else if *name == String::from("allies") {
                                                                match val.as_ref() {
                                                                    SaveValue::SaveArray(allies) => {
                                                                        for ally in allies {
                                                                            match ally.as_ref() {
                                                                                SaveValue::SaveString(allytag) => {
                                                                                    c.allies.push(allytag.clone());
                                                                                },
                                                                                _ => continue,
                                                                            }
                                                                        }
                                                                    },
                                                                    _ => continue,
                                                                }
                                                            }
                                                            else if *name == String::from("subjects") {
                                                                match val.as_ref() {
                                                                    SaveValue::SaveArray(subs) => {
                                                                        for sub in subs {
                                                                            match sub.as_ref() {
                                                                                SaveValue::SaveString(subtag) => {
                                                                                    c.subjects.push(subtag.clone());
                                                                                },
                                                                                _ => continue,
                                                                            }
                                                                        }
                                                                    },
                                                                    _ => continue,
                                                                }
                                                            }
                                                            else {
                                                                continue;
                                                            }
                                                        },
                                                        _ => continue,
                                                    }
                                                }

                                                countries.push(c);
                                            },
                                            _ => continue,
                                        }
                                    },
                                    _ => continue,
                                }
                            }
                        },
                        _ => continue,
                    }
                }
            },
            _ => continue,
        }
    }

    Ok(countries)
}
