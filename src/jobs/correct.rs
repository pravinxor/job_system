use openai_api_rust::{chat::*, *};
use serde_json::{json, Value};
use std::error::Error;

fn fix_linker_err(llm: &OpenAI, error: &Value, prompt: &str) -> Result<Value, Box<dyn Error>> {
    let linker_msg = error["message"].as_str().ok_or("message not found")?;

    if linker_msg.is_empty() {
        return Ok(json!(null));
    }

    let sym_messages = error["symbols"]
        .as_array()
        .ok_or("symbols not found")?
        .iter()
        .flat_map(|sym| sym["message"].as_str())
        .fold(String::new(), |mut acc, line| {
            acc.push_str(line);
            acc.push('\n');
            acc
        });

    let content = format!(
        r#"The following linker error: "{}" with the related symbols: "{}". {}"#,
        linker_msg, sym_messages, prompt
    );
    error_fix(llm, content)
}

fn fix_compile_err(llm: &OpenAI, error: &Value, prompt: &str) -> Result<Value, Box<dyn Error>> {
    let compiler_msg = error["message"].as_str().ok_or("message not found")?;
    let chunk = &error["context"];

    let content = format!(
        r#"The code chunk: "{}" causes the error: "{}". {}"#,
        chunk, compiler_msg, prompt
    );

    error_fix(llm, content)
}

fn error_fix(llm: &OpenAI, content: String) -> Result<Value, Box<dyn Error>> {
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
            // prompt only tested on OpenOrca models
            content,
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
    Ok(serde_json::from_str(response)?)
}

pub fn correct(input: Value) -> Value {
    dbg!(&input);
    let base_url = match input["base_url"].as_str() {
        Some(url) => url,
        None => return json!({"result" : {"message" : "no base URL provided"}, "status" : 1}),
    };
    let auth = Auth::new("not needed for a local LLM");
    let llm = OpenAI::new(auth, base_url);

    let compiler_err_prompt = match input["compiler_err_prompt"].as_str() {
        Some(prompt) => prompt,
        None => {
            return json!({"result" : {"message" : "compiler_err_prompt was not passed into job"}, "status" : 1})
        }
    };

    let linker_err_prompt = match input["linker_err_prompt"].as_str() {
        Some(prompt) => prompt,
        None => {
            return json!({"result" : {"message" : "linker_err_prompt was not passed into job"}, "status" : 1})
        }
    };

    let compiler_errors = match input["files"].as_array() {
        Some(compiler_errors) => compiler_errors,
        None => {
            return json!({"result" : {"message" : "compiler errors was not found, or is not an array"}, "status" : 1})
        }
    };

    let linker_fixes = match fix_linker_err(&llm, &input["linker"], linker_err_prompt) {
        Ok(fixes) => fixes,
        Err(e) => return json!({"result" : {"message" : e.to_string()}, "status" : 1}),
    };

    let compiler_fixes: Vec<Value> = compiler_errors
        .iter()
        .flat_map(|f| f["errors"].as_array())
        .flatten()
        .map(|e| fix_compile_err(&llm, e, compiler_err_prompt))
        .filter_map(|f| {
            if let Err(e) = f {
                eprintln!("Parsing error: {}", e);
                None
            } else {
                f.ok()
            }
        })
        .collect();

    json!({"result" :{"compiler_fixes" : compiler_fixes, "linker_fixes": linker_fixes}, "status" : 0})
}
