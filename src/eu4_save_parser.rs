use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::num::ParseIntError;
use std::num::ParseFloatError;
use std::fmt;

pub enum SaveValue {
    SaveNull,
    SaveBool(bool),
    SaveNumber(i32),
    SaveFloat(f32),
    SaveDate(String),
    SaveString(String),
    SaveArray(Vec<Box<SaveValue>>),
    SaveObject(String, Box<SaveValue>),
}

// skip these tags, because they are not need right now
// or maybe not, i will implement it fully and then try to leave stuff out, if time is a concern

pub fn parse_savegame(save_path: String) -> Result<Vec<Box<SaveValue>>, SaveGameParsingError> {

    // open file and convert to utf8, because savegames are in Windows-1252 Format
    let mut f = File::open(save_path)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;

    let contents_string = String::from_utf8_lossy(&buffer);

    let file_header = "EU4txt\n";

    if !contents_string.starts_with(file_header) {
        return Err(SaveGameParsingError::FileParsing(String::from("Invalid file format. The file must start with 'EU4txt'!")));
    }

    let contents: Vec<char> = contents_string.trim_start_matches(file_header).chars().collect();

    let mut res: Vec<Box<SaveValue>> = Vec::new();

    let mut i = 0;
    let mut d = 0;
    while i < contents.len() {
        let (s, i_new, d_r) = parse_recursive(&contents, i, 0)?;

        if d_r > d {
            d = d_r;
        }

        i = i_new;
        res.push(s);
    }
    println!("MAX DEPTH: {}", d);
    /*let mut i = 0;
    let mut c = 0;
    let mut d = 0;
    while c < 2 {
        let (s, i_new, d_r) = parse_recursive(&contents, i, 0)?;

        if d_r > d {
            d = d_r;
        }

        i = i_new;
        c += 1;
        res.push(s);
    }
    println!("MAX DEPTH: {}", d);*/

    Ok(res)
}

pub fn parse_recursive(remain: &Vec<char>, mut index: usize, mut d: usize) -> Result<(Box<SaveValue>, usize, usize), SaveGameParsingError> { // 8Byte + 8Byte
    if index >= remain.len() {
        return Ok((Box::new(SaveValue::SaveNull), index, d));
    }
    index = skip_whitespace(&remain, index)?;                           // 24 Byte
    if index >= remain.len() {
        return Ok((Box::new(SaveValue::SaveNull), index, d));
    }

    if remain[index] == '{' {
        // SaveArray -> Recursion
        return Ok(parse_array(&remain, index + 1, d)?);                    // 24 Byte
    }
    else if remain[index] == '\n' {
        return Ok((Box::new(SaveValue::SaveNull), index + 1, d));
    }
    else if remain[index] == '"' {
        // string
        index += 1;
        let mut val = String::new();
        while index < remain.len() && remain[index] != '"'  {
            val.push(remain[index]);
            index += 1;
        }

        if remain[index] == '\0' {
            return Ok((Box::new(SaveValue::SaveString(val)), index + 1, d));
        }

        // this is also a Object
        if remain[index] == '=' {
            let (sv, index_r, d_n) = parse_recursive(&remain, index + 1, d)?;
            return Ok((Box::new(SaveValue::SaveObject(val, sv)), index_r, d_n));
        }

        return Ok((Box::new(SaveValue::SaveString(val)), index + 1, d));
    }
    else if remain[index].is_ascii_digit() || remain[index] == '-' {
        // int
        let mut val = String::new();                                        // 24 Bytes
        let mut is_obj = false;                                             // 1 Byte
        let mut count_dot : u8 = 0;                                         // 1 Byte
        while remain[index] != '\n' && remain[index] != ' ' && remain[index] != '\t' {

            if remain[index] == '=' {
                is_obj = true;
                index += 1;
                break;
            }

            if remain[index] == '.' {
                count_dot += 1;
            }

            val.push(remain[index]);
            index += 1;
        }

        let mut ret: (Box<SaveValue>, usize, usize);                           // 8Byte + 8Byte

        if is_obj {
            let (sv, index_r, d_n) = parse_recursive(&remain, index, d)?;      //  24 Byte + 8 Byte + 8 Byte
            return Ok((Box::new(SaveValue::SaveObject(val, sv)), index_r, d_n));
        }

        if count_dot == 1 {
            let r = match val.parse::<f32>() {                          // 24 Byte + 8 Byte
                Ok(l) => l,
                Err(r) => 999.999,
            };

            ret = (Box::new(SaveValue::SaveFloat(r)), index + 1, d);
        }
        else if count_dot == 0 {
            let r = match val.parse::<i32>() {
                Ok(l) => l,
                Err(r) => 999,
            };

            ret = (Box::new(SaveValue::SaveNumber(r)), index + 1, d);
        }
        else {
            ret = (Box::new(SaveValue::SaveDate(val)), index + 1, d);
        }


        return Ok(ret);
    }
    else if remain[index] == '\0' { /*or eof*/
        return Err(SaveGameParsingError::FileParsing(format!("Unexpected EOF at index: {}!", index)));
    }
    else { // it should be TAG=, else it is some error
        let mut val = String::new();
        while remain[index] != '=' {

            // this wierd map_area{
            if remain[index] == '{' {
                let (sv, index_r, d_n) = parse_recursive(&remain, index, d)?;
                return Ok((Box::new(SaveValue::SaveObject(val, sv)), index_r, d_n));
            }

            // can also be a string without ""
            if remain[index] == ' ' || remain[index] == '\t' || remain[index] == '\n' {
                if val == "yes" {
                    return Ok((Box::new(SaveValue::SaveBool(true)), index + 1, d));
                }

                if val == "no" {
                    return Ok((Box::new(SaveValue::SaveBool(false)), index + 1, d));
                }

                return Ok((Box::new(SaveValue::SaveString(val)), index + 1, d));
            }

            val.push(remain[index]);

            index += 1;
        }
        let (sv, index_r, d_n) = parse_recursive(&remain, index + 1, d)?;

        if d_n == 17 {
            match *sv {
                SaveValue::SaveNull => { panic!("DDDD {} -> NULL", val); }
                SaveValue::SaveBool(b) => { panic!("DDDD {} -> {b:?}", val); }
                SaveValue::SaveNumber(b) => { panic!("DDDD {} -> {b:?}", val); }
                SaveValue::SaveFloat(b) => { panic!("DDDD {} -> {b:?}", val); }
                SaveValue::SaveDate(b) => { panic!("DDDD {} -> {b:?}", val); }
                SaveValue::SaveString(b)  => { panic!("DDDD {} -> {b:?}", val); }
                SaveValue::SaveArray(v) => { panic!("DDDD {} -> ARRAY", val); }
                SaveValue::SaveObject(s, b) => { panic!("DDDD {} -> OBJECT", val); }
            }
        }

        return Ok((Box::new(SaveValue::SaveObject(val, sv)), index_r, d_n));
    }
}

// parse an array to see what it contains
pub fn parse_array(remain: &Vec<char>, mut index: usize, d: usize) -> Result<(Box<SaveValue>, usize, usize), SaveGameParsingError> {
    // we have to find seperate little objects

    let mut ret : Vec<Box<SaveValue>> = Vec::new();
    let mut d_r = d;

    // skip whitespaces incase of empty array
    index = skip_whitespace(&remain, index)?;
    let mut c = remain[index];

    while c != '}' {
        // initally there was a skip whitespace here, which "belongs here"
        // but it is also done py parse recursive, so no need
        let (s, i_new, d_n) = parse_recursive(&remain, index, d + 1)?;

        if d_n > d_r {
            d_r = d_n;
        }

        index = i_new;
        ret.push(s);

        if index >= remain.len() {
            break;
        }

        index = skip_whitespace(&remain, index)?;
        c = remain[index];
    }

    // reasonable to skip to next line i think
    index = skip_whitespace(&remain, index)?;

    Ok((Box::new(SaveValue::SaveArray(ret)), index + 1, d_r))
}

pub fn skip_whitespace(remain: &Vec<char>, mut index: usize) -> Result<usize, SaveGameParsingError> {
    if index >= remain.len() {
        return Ok(index);
    }

    let mut c = remain[index];
    while c == ' ' || c == '\t' || c == '\n' {
        index += 1;

        if index >= remain.len() {
            break;
        }

        c = remain[index];
    }

    Ok(index)
}

/*
 Errors, this could be in a seperate file
 */

#[derive(Debug)]
pub enum SaveGameParsingError {
    Io(io::Error),
    IntParse(ParseIntError),
    FloatParse(ParseFloatError),
    FileParsing(String),
}

impl fmt::Display for SaveGameParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveGameParsingError::Io(e) => write!(f, "IO error: {}", e),
            SaveGameParsingError::IntParse(e) => write!(f, "IntParsing error: {}", e),
            SaveGameParsingError::FloatParse(e) => write!(f, "FloatParsing error: {}", e),
            SaveGameParsingError::FileParsing(s) => write!(f, "{}", s),
        }
    }
}

impl From<io::Error> for SaveGameParsingError {
    fn from(err: io::Error) -> Self {
        SaveGameParsingError::Io(err)
    }
}

impl From<ParseIntError> for SaveGameParsingError {
    fn from(err: ParseIntError) -> Self {
        SaveGameParsingError::IntParse(err)
    }
}

impl From<ParseFloatError> for SaveGameParsingError {
    fn from(err: ParseFloatError) -> Self {
        SaveGameParsingError::FloatParse(err)
    }
}

impl std::error::Error for SaveGameParsingError {}
