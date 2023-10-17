use std::{error::Error, process::Command};

use serde_json::{json, Value};

/// Parser, which will launch the make target specified by the the `target` key in the json input
pub(crate) fn output(input: Value) -> Value {
    if let Some(target) = input["target"].as_str() {
        match Command::new("make").arg(target).output() {
            Ok(c) => json!({"clang_output" : String::from_utf8_lossy(&c.stderr)}),
            Err(e) => {
                json!({"error" : e.to_string()})
            }
        }
    } else {
        json!({
            "error": "no 'target' key found in input"
        })
    }
}
