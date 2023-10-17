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
        let mut output = json!({"files" : {}, "linker": {"message" : "", "symbols": []}});
        for line in clang_output.lines() {
            if let Some(caps) = COMPILER_EXPR.captures(line) {
                let file_entry = output["files"]
                    .as_object_mut()
                    .unwrap()
                    .entry(&caps["filename"])
                    .or_insert_with(|| json!([]))
                    .as_array_mut()
                    .unwrap();

                file_entry.push(json!({
                    "line": &caps["line"],
                    "column": &caps["column"],
                    "message": &caps["message"]
                }));
            } else if let Some(caps) = LINKER_EXPR.captures(line) {
                output["linker"]["message"] = json!({"message": &caps["message"]});
            } else if LINKER_TXT_EXPR.is_match(line) {
                output["linker"]["symbols"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({ "message" : line }));
            } else {
                continue;
            }
        }
        output
    } else {
        json!({"error": "no 'clang_output' key found in input"})
    }
}
