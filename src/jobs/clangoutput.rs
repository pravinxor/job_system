use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{json, Value};

lazy_static! {
    static ref LINKER_TXT_EXPR: Regex =
        Regex::new(r"\(.\w+\+0x\w+\): undefined reference to `\w+'").unwrap();
    static ref LINKER_EXPR: Regex = Regex::new(r"clang-\d+: error: (?P<message>.*)").unwrap();
    static ref COMPILER_EXPR: Regex = Regex::new(
        r"(?P<filename>.*):(?P<line>\d+):(?P<column>\d+): (?:error|warning): (?P<message>.*)"
    )
    .unwrap();
}

pub fn parse(input: Value) -> Value {
    if let Some(clang_output) = input["clang_output"].as_str() {
        let mut output = json!({"files": [], "linker": {"message" : "", "symbols": []}});
        for line in clang_output.lines() {
            if let Some(caps) = COMPILER_EXPR.captures(line) {
                let filename = &caps["filename"];
                let error_entry = json!({
                    "line": &caps["line"].parse::<u64>().unwrap_or_default(),
                    "column": &caps["column"].parse::<u64>().unwrap_or_default(),
                    "message": &caps["message"]
                });

                let file_entry = match output["files"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|file| file["filename"] == filename)
                {
                    Some(file_entry) => file_entry,
                    None => {
                        let file_entry = json!({"filename" : filename, "errors": []});
                        let files = output["files"].as_array_mut().unwrap();
                        files.push(file_entry);
                        files.last_mut().unwrap()
                    }
                };

                file_entry["errors"]
                    .as_array_mut()
                    .unwrap()
                    .push(error_entry);
            } else if let Some(caps) = LINKER_EXPR.captures(line) {
                output["linker"]["message"] = json!(&caps["message"]);
            } else if LINKER_TXT_EXPR.is_match(line) {
                output["linker"]["symbols"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"message": line }));
            } else {
                continue;
            }
        }
        json!({"result": output, "status": 0})
    } else {
        json!({"result": {"message": {"error": "no 'clang_output' key found in input"}}, "status": 1})
    }
}
