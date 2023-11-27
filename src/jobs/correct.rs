use openai_api_rust::{chat::*, *};
use serde_json::{json, Value};
use std::error::Error;

pub fn error_fix(llm: &OpenAI, error: &Value) -> Result<Value, Box<dyn Error>> {
    let compiler_msg = error["message"].as_str().ok_or("message not found")?;
    let chunk = &error["context"];

    let body = ChatBody {
        model: "model".into(),
        max_tokens: Some(99999),
        temperature: Some(0.2),
        top_p: Some(0.1),
        n: None,
        stream: None,
        stop: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
        messages: vec![Message {
            role: Role::User,
            content: format!(
                r#"The code chunk: "{}" causes the error: "{}". A fully JSON style response with the requirements: NO additional plain text or trailing characters, JSON schema only contains a "message" field, explaining the error (in the context of the code), and "fix" field, with an updated code chunk which ONLY fixes the specified error: "#,
                chunk, compiler_msg
            ),
        }],
    };

    let rs = llm.chat_completion_create(&body).unwrap();
    let choice = rs.choices.first().ok_or("No response yielded")?;
    let response = choice
        .message
        .as_ref()
        .ok_or("No message yielded")?
        .content
        .as_str();
    dbg!(response);
    Ok(serde_json::from_str(response)?)
}

pub fn correct(x: Value) -> Value {
    let input = x["input"].to_owned();
    let base_url = "http://localhost:4891/v1/";
    let auth = Auth::new("not needed for a local LLM");
    let llm = OpenAI::new(auth, base_url);

    let compiler_errors = input["files"].as_array().unwrap();
    let fixes: Vec<Value> = compiler_errors
        .iter()
        .flat_map(|f| f["errors"].as_array())
        .flatten()
        .map(|e| error_fix(&llm, e))
        .filter_map(|f| {
            if let Err(e) = f {
                eprintln!("Parsing error: {}", e);
                None
            } else {
                f.ok()
            }
        })
        .collect();
    json!({"result" :{"fixes" : fixes}, "status" : 0})
}
