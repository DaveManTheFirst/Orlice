use std::io;
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

pub fn parse_savegame(save_buffer: &[u8]) -> Result<Vec<Box<SaveValue>>, SaveGameParsingError> {
    // convert to utf8, because savegames are in Windows-1252 Format
    let contents_string = String::from_utf8_lossy(&save_buffer);

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

    Ok(res)
}

pub fn parse_recursive(remain: &Vec<char>, mut index: usize) -> Result<(Box<SaveValue>, usize), SaveGameParsingError> {
    if index >= remain.len() {
        return Ok((Box::new(SaveValue::SaveNull), index));
    }
    index = skip_whitespace(&remain, index)?;
    if index >= remain.len() {
        return Ok((Box::new(SaveValue::SaveNull), index));
    }

    if remain[index] == '{' {
        // SaveArray -> Recursion
        return Ok(parse_array(&remain, index + 1)?);
    }
    else if remain[index] == '\n' {
        return Ok((Box::new(SaveValue::SaveNull), index + 1));
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
            return Ok((Box::new(SaveValue::SaveString(val)), index + 1));
        }

        // this is also a Object
        if remain[index] == '=' {
            let (sv, index_r) = parse_recursive(&remain, index + 1)?;
            return Ok((Box::new(SaveValue::SaveObject(val, sv)), index_r));
        }

        return Ok((Box::new(SaveValue::SaveString(val)), index + 1));
    }
    else if remain[index].is_ascii_digit() || remain[index] == '-' {
        // int
        let mut val = String::new();
        let mut is_obj = false;
        let mut count_dot : u8 = 0;
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

        let ret: (Box<SaveValue>, usize);

        if is_obj {
            let (sv, index_r) = parse_recursive(&remain, index)?;
            return Ok((Box::new(SaveValue::SaveObject(val, sv)), index_r));
        }

        if count_dot == 1 {
            let r = match val.parse::<f32>() {
                Ok(l) => l,
                Err(_) => 999.999,
            };

            ret = (Box::new(SaveValue::SaveFloat(r)), index + 1);
        }
        else if count_dot == 0 {
            let r = match val.parse::<i32>() {
                Ok(l) => l,
                Err(_) => 999,
            };

            ret = (Box::new(SaveValue::SaveNumber(r)), index + 1);
        }
        else {
            ret = (Box::new(SaveValue::SaveDate(val)), index + 1);
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
                let (sv, index_r) = parse_recursive(&remain, index)?;
                return Ok((Box::new(SaveValue::SaveObject(val, sv)), index_r));
            }

            // can also be a string without ""
            if remain[index] == ' ' || remain[index] == '\t' || remain[index] == '\n' {
                if val == "yes" {
                    return Ok((Box::new(SaveValue::SaveBool(true)), index + 1));
                }

                if val == "no" {
                    return Ok((Box::new(SaveValue::SaveBool(false)), index + 1));
                }

                return Ok((Box::new(SaveValue::SaveString(val)), index + 1));
            }

            val.push(remain[index]);

            index += 1;
        }

        let (sv, index_r,) = parse_recursive(&remain, index + 1)?;
        return Ok((Box::new(SaveValue::SaveObject(val, sv)), index_r));
    }
}

// parse an array to see what it contains
pub fn parse_array(remain: &Vec<char>, mut index: usize) -> Result<(Box<SaveValue>, usize), SaveGameParsingError> {
    // we have to find seperate little objects

    let mut ret : Vec<Box<SaveValue>> = Vec::new();

    // skip whitespaces incase of empty array
    index = skip_whitespace(&remain, index)?;
    let mut c = remain[index];

    while c != '}' {
        // initally there was a skip whitespace here, which "belongs here"
        // but it is also done py parse recursive, so no need
        let (s, i_new) = parse_recursive(&remain, index)?;

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

    Ok((Box::new(SaveValue::SaveArray(ret)), index + 1))
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
