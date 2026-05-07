use std::error::Error;
use crate::game_types::Province;

pub async fn read_provinces(def_path: String, wb_path: String) -> Result<Vec<Province>, Box<dyn Error>> {
    let mut pv = Vec::new();

    let resp = gloo_net::http::Request::get(&def_path).send().await?;
    let csv_str = resp.text().await?;

    let mut rdr = csv::ReaderBuilder::new()
    .delimiter(b';')
    .from_reader(csv_str.as_bytes());
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
    let resp_wb = gloo_net::http::Request::get(&wb_path).send().await?;
    let csv_str_wb = resp_wb.text().await?;

    let mut rdr_water = csv::ReaderBuilder::new()
    .delimiter(b';')
    .from_reader(csv_str_wb.as_bytes());

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
