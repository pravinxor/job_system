use serde_json::{json, Value};

pub fn display_error(input: Value) -> Value {
    eprint!("Error: ");
    println!("{}", input);
    json!({"result": {}, "status": 0})
}
