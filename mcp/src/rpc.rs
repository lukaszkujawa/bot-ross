use crate::studio::Studio;
use serde_json::{Value, json};

const PROTOCOL_VERSION: &str = "2024-11-05";
const PARSE_ERROR: i64 = -32700;
const METHOD_NOT_FOUND: i64 = -32601;

pub fn handle(line: &str, studio: &mut Studio) -> Option<Value> {
    let message: Value = match serde_json::from_str(line) {
        Ok(message) => message,
        Err(problem) => return Some(error(Value::Null, PARSE_ERROR, &problem.to_string())),
    };
    let id = message.get("id").cloned()?;
    let method = message["method"].as_str().unwrap_or_default();
    let params = message.get("params").cloned().unwrap_or_else(|| json!({}));
    Some(match method {
        "initialize" => result(id, json!({ "protocolVersion": PROTOCOL_VERSION, "capabilities": { "tools": {} }, "serverInfo": { "name": "bot-ross", "version": env!("CARGO_PKG_VERSION") } })),
        "ping" => result(id, json!({})),
        "tools/list" => result(id, json!({ "tools": tools() })),
        "tools/call" => {
            let name = params["name"].as_str().unwrap_or_default();
            let arguments = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
            match studio.call(name, arguments) {
                Ok(content) => result(id, json!({ "content": content })),
                Err(problem) => result(id, json!({ "content": [{ "type": "text", "text": problem }], "isError": true })),
            }
        }
        _ => error(id, METHOD_NOT_FOUND, &format!("unknown method {method}")),
    })
}

fn result(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

pub fn tools() -> Value {
    let stroke = json!({
        "type": "object",
        "required": ["points"],
        "properties": {
            "points": { "type": "array", "minItems": 1, "items": { "type": "array", "items": { "type": "number" }, "minItems": 2, "maxItems": 2 }, "description": "[x, y] pixels from the top-left corner; one smooth curve is fitted through them, so give 2 to 6 points per stroke and use several strokes for complex shapes; a single point makes a dot" },
            "radius": { "type": "number", "default": 8, "description": "half-width in px; the mark is about 2 × radius wide" },
            "colour": { "type": "string", "default": "#222222", "description": "#rrggbb" },
            "opacity": { "type": "number", "default": 0.8, "minimum": 0, "maximum": 1 },
            "pressure": { "type": "string", "enum": ["hand", "flat", "fading"], "default": "hand", "description": "hand eases in and out, flat is constant, fading trails off" }
        }
    });
    json!([
        {
            "name": "new_canvas",
            "description": "Start a blank canvas of the given size with one normal layer; the previous drawing is discarded",
            "inputSchema": { "type": "object", "properties": {
                "width": { "type": "integer", "default": 1024 },
                "height": { "type": "integer", "default": 1024 },
                "background": { "type": "string", "default": "#ffffff", "description": "#rrggbb or white" }
            } }
        },
        {
            "name": "list_brushes",
            "description": "List the brush presets that draw can use, by name with a short description of each",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "draw",
            "description": "Paint strokes on the top layer with a brush preset. Each stroke is one smooth curve through its points, rendered with the brush's real physics: bristles, paint load, washes or a knife, so the same path looks different with each brush",
            "inputSchema": { "type": "object", "required": ["brush", "strokes"], "properties": {
                "brush": { "type": "string", "description": "a preset name from list_brushes, or a path to a brush .toml" },
                "strokes": { "type": "array", "minItems": 1, "items": stroke }
            } }
        },
        {
            "name": "add_layer",
            "description": "Add a new layer on top; later strokes go onto it. multiply layers darken what is below like transparent pigment, normal layers cover it like opaque paint",
            "inputSchema": { "type": "object", "properties": {
                "blend": { "type": "string", "enum": ["normal", "multiply"], "default": "normal" }
            } }
        },
        {
            "name": "view",
            "description": "Return the current canvas as a PNG image to look at, optionally cropped to a region and scaled down to fit max_side",
            "inputSchema": { "type": "object", "properties": {
                "region": { "type": "array", "items": { "type": "integer" }, "minItems": 4, "maxItems": 4, "description": "[x, y, width, height] in canvas pixels; omit for the whole canvas" },
                "max_side": { "type": "integer", "default": 768, "description": "longest side of the returned image in px" }
            } }
        },
        {
            "name": "save",
            "description": "Write the current canvas to a PNG file",
            "inputSchema": { "type": "object", "required": ["path"], "properties": { "path": { "type": "string" } } }
        },
        {
            "name": "clear",
            "description": "Wipe the canvas back to its background with one normal layer, keeping its size",
            "inputSchema": { "type": "object", "properties": {} }
        }
    ])
}
