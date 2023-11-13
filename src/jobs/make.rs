use std::process::Command;

use serde_json::{json, Value};

/// Parser, which will launch the make target specified by the the `target` key in the json input
pub(crate) fn output(input: Value) -> Value {
    if let Some(target) = input["input"]["target"].as_str() {
        match Command::new("make").arg(target).output() {
            Ok(c) => {
                json!({"result": {"clang_output": String::from_utf8_lossy(&c.stderr)}, "status": 0})
            }
            Err(e) => {
                json!({"result": {"message": e.to_string()}, "status": 1})
            }
        }
    } else {
        json!({
            "result": {"message": "no 'target' key found in input"},
            "status": 2
        })
    }
}
