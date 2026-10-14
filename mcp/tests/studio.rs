use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::Write;

fn brushes() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../brushes")
}

fn session(lines: &[Value]) -> Vec<Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_bot-ross-mcp")).arg("--headless").arg("--brushes").arg(brushes()).arg("--size").arg("64").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let input: String = lines.iter().map(|line| format!("{line}\n")).collect();
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    String::from_utf8(output.stdout).unwrap().lines().map(|line| serde_json::from_str(line).unwrap()).collect()
}

fn call(id: u64, name: &str, arguments: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": { "name": name, "arguments": arguments } })
}

fn text(response: &Value) -> String {
    response["result"]["content"].as_array().unwrap().iter().filter_map(|item| item["text"].as_str()).collect::<Vec<_>>().join("\n")
}

#[test]
fn initialize_lists_tools_and_ignores_notifications() {
    let replies = session(&[
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": { "name": "test", "version": "0" } } }),
        json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
        json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }),
        json!({ "jsonrpc": "2.0", "id": 3, "method": "nothing" }),
    ]);
    assert_eq!(replies.len(), 3);
    assert_eq!(replies[0]["result"]["serverInfo"]["name"], "bot-ross");
    let names: Vec<&str> = replies[1]["result"]["tools"].as_array().unwrap().iter().map(|tool| tool["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["new_canvas", "list_brushes", "draw", "add_layer", "view", "save", "clear"]);
    assert_eq!(replies[2]["error"]["code"], -32601);
}

#[test]
fn drawing_changes_the_canvas_and_view_returns_a_png() {
    let replies = session(&[
        call(1, "view", json!({ "max_side": 32 })),
        call(2, "draw", json!({ "brush": "pen", "strokes": [{ "points": [[4, 4], [60, 60]], "radius": 2, "colour": "#000000", "opacity": 1 }] })),
        call(3, "view", json!({ "max_side": 32 })),
        call(4, "view", json!({ "region": [0, 0, 16, 16], "max_side": 64 })),
        call(5, "add_layer", json!({ "blend": "multiply" })),
        call(6, "list_brushes", json!({})),
        call(7, "draw", json!({ "brush": "missing", "strokes": [{ "points": [[1, 1]] }] })),
        call(8, "draw", json!({ "brush": "pen", "strokes": [{ "points": [[1, 1]], "width": 3 }] })),
    ]);
    let image = |reply: &Value| reply["result"]["content"][0].clone();
    assert_eq!(image(&replies[0])["mimeType"], "image/png");
    assert!(image(&replies[0])["data"].as_str().unwrap().starts_with("iVBORw0KGgo"));
    assert!(text(&replies[1]).starts_with("painted 1 strokes with Pen on layer 1"), "{}", text(&replies[1]));
    assert_ne!(image(&replies[0])["data"], image(&replies[2])["data"]);
    assert!(text(&replies[3]).contains("region x 0 y 0 16x16"));
    assert!(text(&replies[4]).contains("layer 2"));
    assert!(text(&replies[5]).contains("pencil: Pencil"));
    assert_eq!(replies[6]["result"]["isError"], true);
    assert!(text(&replies[7]).contains("unknown field `width`"), "{}", text(&replies[7]));
}
