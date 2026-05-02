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
    while i < contents.len() {
        let (s, i_new) = parse_recursive(&contents, i)?;
        i = i_new;
        res.push(s);
    }
    /*let mut i = 0;
    let mut c = 0;
    while c < 2 {
        let (s, i_new) = parse_recursive(&contents, i)?;
        i = i_new;
        c += 1;
        res.push(s);
    }*/

    Ok(res)
}

pub fn parse_recursive(remain: &Vec<char>, start_index: usize) -> Result<(Box<SaveValue>, usize), SaveGameParsingError> {
    if start_index >= remain.len() {
        return Ok((Box::new(SaveValue::SaveNull), start_index));
    }
    let index = skip_whitespace(&remain, start_index)?;
    if index >= remain.len() {
        return Ok((Box::new(SaveValue::SaveNull), index));
    }

    let c = remain[index];
    if c == '{' {
        // SaveArray -> Recursion
        return Ok(parse_array(&remain, index + 1)?);
    }
    else if c == '\n' {
        return Ok((Box::new(SaveValue::SaveNull), index + 1));
    }
    else if c == '"' {
        // string
        let mut i = index + 1;
        let mut val = String::new();
        while i < remain.len() && remain[i] != '"'  {
            val.push(remain[i]);
            i += 1;
        }

        if remain[i] == '\0' {
            return Ok((Box::new(SaveValue::SaveString(val)), i + 1));
        }

        if remain[i] == '=' {
            let (sv, index_r) = parse_recursive(&remain, i + 1)?;
            return Ok((Box::new(SaveValue::SaveObject(val, sv)), index_r));
        }

        return Ok((Box::new(SaveValue::SaveString(val)), i + 1));
    }
    else if c.is_ascii_digit() {
        // int
        let mut i = index + 1;
        let mut val = String::new();
        let mut is_obj = false;
        let mut count_dot = 0;
        while remain[i] != '\n' {

            if remain[i] == '=' {
                is_obj = true;
                i += 1;
                break;
            }

            if remain[i] == '.' {
                count_dot += 1;
            }

            val.push(remain[i]);
            i += 1;
        }

        let mut ret: (Box<SaveValue>, usize);

        if is_obj {
            let (sv, index_r) = parse_recursive(&remain, i)?;
            return Ok((Box::new(SaveValue::SaveObject(val, sv)), index_r));
        }

        if count_dot == 1 {
            let r = match val.parse::<f32>() {
                Ok(l) => l,
                Err(r) => 999.999,
            };

            ret = (Box::new(SaveValue::SaveFloat(r)), i + 1);
        }
        else if count_dot == 0 {
            let r = match val.parse::<i32>() {
                Ok(l) => l,
                Err(r) => 999,
            };

            ret = (Box::new(SaveValue::SaveNumber(r)), i + 1);
        }
        else {
            ret = (Box::new(SaveValue::SaveDate(val)), i + 1);
        }


        return Ok(ret);
    }
    else if c == '\0' { /*or eof*/
        return Err(SaveGameParsingError::FileParsing(format!("Unexpected EOF at index: {}!", index)));
    }
    else { // it should be TAG=, else it is some error
        let mut i = index;
        let mut val = String::new();
        while remain[i] != '=' {

            if c == '\n' || c == ' ' {
                return Err(SaveGameParsingError::FileParsing(format!("Unknown token at index: {}!", index)));
            }

            if val == "yes" {
                return Ok((Box::new(SaveValue::SaveBool(true)), index + 4));
            }

            if val == "no" {
                return Ok((Box::new(SaveValue::SaveBool(false)), index + 3));
            }

            val.push(remain[i]);
            i += 1;
        }
        let (sv, index_r) = parse_recursive(&remain, i + 1)?;
        return Ok((Box::new(SaveValue::SaveObject(val, sv)), index_r));
    }
}

// parse an array to see what it contains
pub fn parse_array(remain: &Vec<char>, index: usize) -> Result<(Box<SaveValue>, usize), SaveGameParsingError> {
    let mut c = remain[index];
    let mut i = index;

    // we have to find seperate little objects

    let mut ret : Vec<Box<SaveValue>> = Vec::new();
    while c != '}' {

        i = skip_whitespace(&remain, i)?;

        let (s, i_new) = parse_recursive(&remain, i)?;
        i = i_new;
        ret.push(s);

        if i >= remain.len() {
            break;
        }

        c = remain[i];
    }

    // reasonable to skip to next line i think
    i = skip_whitespace(&remain, i)?;

    Ok((Box::new(SaveValue::SaveArray(ret)), i + 1))
}

pub fn skip_whitespace(remain: &Vec<char>, index: usize) -> Result<usize, SaveGameParsingError> {
    if index >= remain.len() {
        return Ok(index);
    }

    let mut c = remain[index];
    let mut i = index;
    while c == ' ' || c == '\t' || c == '\n' {
        i += 1;

        if i >= remain.len() {
            break;
        }

        c = remain[i];
    }

    Ok(i)
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
