//! Role: Turning a protocol result or a tool error into an MCP `CallToolResult`.

use kivact_devtools_protocol as p;
use serde_json::{Value, json};

use crate::target::ToolError;

/// A successful call: the result as text and as structured content.
pub(crate) fn success(method: &str, value: Value) -> Value {
    if method == "screenshot"
        && let Ok(image) = serde_json::from_value::<p::Image>(value.clone())
    {
        return screenshot(&image);
    }
    json!({
        "content": [{ "type": "text", "text": value.to_string() }],
        "structuredContent": value,
    })
}

/// A failed call: the reason as text, marked as an error.
pub(crate) fn failure(error: &ToolError) -> Value {
    let text = match error {
        ToolError::Rpc(error) => format!("{}: {}", error.code, error.message),
        ToolError::Text(text) => text.clone(),
    };
    json!({
        "content": [{ "type": "text", "text": text }],
        "isError": true,
    })
}

fn screenshot(image: &p::Image) -> Value {
    json!({
        "content": [{
            "type": "image",
            "data": image.png_base64,
            "mimeType": "image/png",
        }],
        "structuredContent": {
            "width": image.width,
            "height": image.height,
        },
    })
}
