use std::fs::File;
use std::io::{Read, Error};
use std::string::String;

pub fn as_string(filename: &str) -> Result<String, Error> {
    let mut result = String::new();
        
    if let Err(x) = File::open(filename).unwrap().read_to_string(&mut result) {
        Err(x)
    } else {
    	Ok(result)
    }
}