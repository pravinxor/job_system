use serde_json::{json, Value};

pub fn display_error(x: Value) -> Value {
    let input = &x["input"];
    eprint!("Error: ");
    println!("{}", input);
    json!({"result": {}, "status": 0})
}
