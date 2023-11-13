use serde_json::{json, Value};

pub fn print_success(x: Value) -> Value {
    let input = &x["input"];
    eprintln!("Success: ");
    println!("{}", input);
    json!({"result": {}, "status": 0})
}
