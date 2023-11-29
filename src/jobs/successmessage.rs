use serde_json::{json, Value};

pub fn print_success(input: Value) -> Value {
    eprintln!("Success: ");
    println!("{}", input);
    json!({"result": {}, "status": 0})
}
