use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use serde_json::{json, Value};

fn get_context(reader: &mut BufReader<File>, start: u64, len: usize) -> String {
    reader
        .lines()
        .skip(start as usize)
        .take(len)
        .flat_map(|line_result| line_result.ok())
        .collect()
}

fn replace_entries(filename: &str, error_array: &mut Vec<Value>) -> Result<(), Value> {
    if let Ok(file) = File::open(filename) {
        let mut reader = BufReader::new(file);
        for error in error_array {
            if let Some(line) = error["line"].as_u64() {
                let context = get_context(&mut reader, 0.min(line - 2), 5);
                error["context"] = Value::String(context);
            } else {
                return Err(json!({ "error" : "line must exist and be an Integer type" }));
            }
        }
    }
    Ok(())
}

/// Adds context to the file error, but including the line of the error as well as 2 lines below and above
pub fn read_context(input: Value) -> Value {
    let mut output = input;
    if let Some(files) = output["files"].as_array_mut() {
        for file in files {
            let filename = match file["filename"].as_str() {
                Some(filename) => filename.to_string(),
                None => {
                    return json!({"error" : "files[]->filename is not a String or may not exist"})
                }
            };
            if let Some(error_array) = file["errors"].as_array_mut() {
                if let Err(e) = replace_entries(&filename, error_array) {
                    return e;
                }
            } else {
                return json!({"error" : "files[]->errors[] is not an array or may not exist or files[]->filename is not a string or may not exist"});
            }
        }
    } else {
        return json!({"error" : "files[] is not an array or may not exist"});
    }
    output
}
