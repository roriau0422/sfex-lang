use crate::runtime::value::Value;
use bigdecimal::BigDecimal;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;

// 1. GLOBAL CLIENT
static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .expect("Failed to create HTTP client")
});

// --- Request Structs ---

#[derive(Serialize)]
struct CreateResponseRequest {
    model: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    input: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
struct ResponseUsage {
    #[serde(default)]
    input_tokens: i64,
    #[serde(default)]
    output_tokens: i64,
    #[serde(default)]
    total_tokens: i64,
}

#[derive(Deserialize, Debug)]
struct OutputContent {
    #[serde(rename = "type")]
    content_type: Option<String>,

    #[serde(default)]
    text: String,
}

#[derive(Deserialize, Debug)]
struct OutputItem {
    #[serde(rename = "type")]
    item_type: Option<String>,
    role: Option<String>,
    content: Option<Vec<OutputContent>>,
    status: Option<String>,
}

#[derive(Deserialize, Debug)]
struct OpenAIResponseObj {
    id: Option<String>,
    model: Option<String>,
    output: Option<Vec<OutputItem>>,
    usage: Option<ResponseUsage>,
}

fn extract_string(val: &Value) -> Option<String> {
    Some(val.to_display_string())
}

// Main logic to call v1/responses
fn call_responses_api(
    input_payload: serde_json::Value,
    instructions: Option<String>,
    options: Option<&Value>,
) -> Result<Value, String> {
    let mut model = "gpt-4o".to_string();
    let mut max_output_tokens: Option<i64> = None;
    let mut api_key: Option<String> = None;
    let mut temperature: Option<f64> = None;
    let mut reasoning_config: Option<serde_json::Value> = None;
    let mut text_config: Option<serde_json::Value> = None;

    if let Some(opts) = options {
        if let Value::Map(options_map) = opts {
            let opts = options_map.read().expect("lock poisoned");

            if let Some(m) = opts.get("model").or_else(|| opts.get("Model")) {
                model = extract_string(m).unwrap_or_default();
            }
            if let Some(k) = opts.get("api_key").or_else(|| opts.get("ApiKey")) {
                api_key = extract_string(k);
            }
            if let Some(mt) = opts
                .get("max_output_tokens")
                .or_else(|| opts.get("MaxOutputTokens"))
            {
                if let Value::Number(n) = mt {
                    let s = n.to_string();
                    if let Ok(parsed) = s.parse::<i64>() {
                        max_output_tokens = Some(parsed);
                    }
                }
                else if let Some(mt_legacy) = opts.get("max_tokens") {
                    if let Value::Number(n) = mt_legacy {
                        let s = n.to_string();
                        if let Ok(parsed) = s.parse::<i64>() {
                            max_output_tokens = Some(parsed);
                        }
                    }
                }
            }
            if let Some(t) = opts.get("temperature").or_else(|| opts.get("Temperature")) {
                if let Value::Number(n) = t {
                    let s = n.to_string();
                    if let Ok(parsed) = s.parse::<f64>() {
                        temperature = Some(parsed);
                    }
                }
            }
            if let Some(r) = opts
                .get("reasoning_effort")
                .or_else(|| opts.get("ReasoningEffort"))
            {
                let effort = extract_string(r).unwrap_or("medium".to_string());
                reasoning_config = Some(json!({ "effort": effort }));
            }
        }
    }

    // for model specific parameters
    let is_reasoning_model = model.starts_with("o1")
        || model.starts_with("o3")
        || model.starts_with("o4")
        || model.starts_with("gpt-5");

    if is_reasoning_model {
        // Remove temperature
        temperature = None;

        // Make sure reasoning block exists
        if reasoning_config.is_none() {
            reasoning_config = Some(json!({ "effort": "medium" }));
        }

        // Add text formatting config
        text_config = Some(json!({
            "format": { "type": "text" },
            "verbosity": "medium"
        }));
    }

    let key = api_key
        .or_else(|| std::env::var("OPENAI_API_KEY").ok())
        .ok_or("OPENAI_API_KEY not found")?;

    let request_body = CreateResponseRequest {
        model,
        input: Some(input_payload),
        instructions,
        max_output_tokens,
        temperature,
        reasoning: reasoning_config,
        text: text_config,
    };

    let response = HTTP_CLIENT
        .post("https://api.openai.com/v1/responses")
        .header("Authorization", format!("Bearer {}", key))
        .json(&request_body)
        .send()
        .map_err(|e| format!("Network error: {}", e))?;

    let status = response.status();
    let response_text = response.text().unwrap_or_default();

    if !status.is_success() {
        return Err(format!(
            "OpenAI Responses API Error {}: {}",
            status, response_text
        ));
    }

    let api_response: OpenAIResponseObj = serde_json::from_str(&response_text).map_err(|e| {
        format!(
            "Failed to parse JSON. \nError: {} \nRaw Response: {}",
            e, response_text
        )
    })?;

    let mut result_map = HashMap::new();

    result_map.insert(
        "Status".to_string(),
        Value::Number(BigDecimal::from(status.as_u16())),
    );
    result_map.insert(
        "Id".to_string(),
        Value::String(api_response.id.unwrap_or_default()),
    );
    result_map.insert(
        "Model".to_string(),
        Value::String(api_response.model.unwrap_or_default()),
    );

    let mut main_content = String::new();
    let mut role = String::new();
    let mut finish_status = String::new();

    if let Some(outputs) = api_response.output {
        for item in outputs {
            if item.item_type.as_deref().unwrap_or("message") == "message" {
                if let Some(r) = &item.role {
                    role = r.clone();
                }

                finish_status = item.status.unwrap_or_else(|| "unknown".to_string());

                if let Some(contents) = &item.content {
                    for content in contents {
                        if content.content_type.as_deref().unwrap_or("output_text") == "output_text"
                        {
                            main_content.push_str(&content.text);
                        }
                    }
                }
            }
        }
    }

    result_map.insert("Content".to_string(), Value::String(main_content));
    result_map.insert("Role".to_string(), Value::String(role));
    result_map.insert(
        "FinishStatus".to_string(),
        Value::String(finish_status.clone()),
    );
    result_map.insert("FinishReason".to_string(), Value::String(finish_status));

    if let Some(usage) = api_response.usage {
        let mut usage_map = HashMap::new();
        let input = BigDecimal::from(usage.input_tokens);
        let output = BigDecimal::from(usage.output_tokens);
        let total = BigDecimal::from(usage.total_tokens);

        usage_map.insert("InputTokens".to_string(), Value::Number(input.clone()));
        usage_map.insert("OutputTokens".to_string(), Value::Number(output.clone()));
        usage_map.insert("PromptTokens".to_string(), Value::Number(input));
        usage_map.insert("CompletionTokens".to_string(), Value::Number(output));
        usage_map.insert("TotalTokens".to_string(), Value::Number(total));

        result_map.insert(
            "Usage".to_string(),
            Value::Map(Arc::new(std::sync::RwLock::new(usage_map))),
        );
    }

    Ok(Value::Map(Arc::new(std::sync::RwLock::new(result_map))))
}

pub fn create_llm_module() -> Value {
    let mut methods = HashMap::new();

    methods.insert(
        "Simple".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.is_empty() {
                return Err("Requires prompt string".to_string());
            }
            let prompt = args[0].to_display_string();
            let options = args.get(1);
            call_responses_api(json!(prompt), None, options)
        }))),
    );

    methods.insert(
        "ChatWithSystem".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.len() < 2 {
                return Err("Requires system and user prompt".to_string());
            }
            let system_prompt = args[0].to_display_string();
            let user_prompt = args[1].to_display_string();
            let options = args.get(2);
            call_responses_api(json!(user_prompt), Some(system_prompt), options)
        }))),
    );

    methods.insert(
        "Chat".to_string(),
        Value::NativeFunction(Arc::new(Box::new(|args| {
            if args.is_empty() {
                return Err("Requires messages list".to_string());
            }

            let msgs_list = match &args[0] {
                Value::List(l) => l.read().expect("lock poisoned").clone(),
                _ => return Err("First arg must be list".to_string()),
            };

            let mut input_array = Vec::new();
            let mut instruction_text = None;

            for item in msgs_list {
                if let Value::Map(m) = item {
                    let m = m.read().expect("lock poisoned");
                    let role = m
                        .get("role")
                        .or_else(|| m.get("Role"))
                        .map(|v| v.to_display_string())
                        .unwrap_or("user".to_string());

                    let content = m
                        .get("content")
                        .or_else(|| m.get("Content"))
                        .map(|v| v.to_display_string())
                        .unwrap_or_default();

                    if role == "system" {
                        instruction_text = Some(content);
                    } else {
                        let content_type = if role == "assistant" {
                            "output_text"
                        } else {
                            "input_text"
                        };
                        input_array.push(json!({
                            "type": "message",
                            "role": role,
                            "content": [{ "type": content_type, "text": content }]
                        }));
                    }
                }
            }

            let options = args.get(1);
            call_responses_api(json!(input_array), instruction_text, options)
        }))),
    );

    Value::Map(Arc::new(std::sync::RwLock::new(methods)))
}
