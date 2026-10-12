//! Spawns the `mcp` binary and talks MCP (JSON-RPC over stdio) to it:
//! initialize, tools/list, three tools/call, resources/list and resources/read.

use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc,
    thread,
    time::Duration,
};

use serde_json::{json, Value};

struct Server {
    child: Child,
    stdin: ChildStdin,
    lines: mpsc::Receiver<String>,
}

impl Server {
    fn start() -> Self {
        Self::start_with(&[])
    }

    fn start_with(env: &[(&str, &str)]) -> Self {
        let mut command = Command::new(env!("CARGO_BIN_EXE_mcp"));
        for (key, value) in env {
            command.env(key, value);
        }
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("start mcp binary");
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, lines) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if sender.send(line).is_err() {
                    break;
                }
            }
        });
        Self {
            child,
            stdin,
            lines,
        }
    }

    fn send(&mut self, message: Value) {
        writeln!(self.stdin, "{message}").unwrap();
        self.stdin.flush().unwrap();
    }

    fn send_raw(&mut self, line: &str) {
        writeln!(self.stdin, "{line}").unwrap();
        self.stdin.flush().unwrap();
    }

    /// The next message on stdout; panics after 60 s, which is what a
    /// client without an answer would do.
    fn next(&mut self) -> Value {
        let line = self
            .lines
            .recv_timeout(Duration::from_secs(60))
            .expect("the server answers");
        serde_json::from_str(&line).unwrap()
    }

    fn request(&mut self, id: u64, method: &str, params: Value) -> Value {
        self.send(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        loop {
            let line = self
                .lines
                .recv_timeout(Duration::from_secs(60))
                .unwrap_or_else(|_| panic!("no answer to {method}"));
            let message: Value = serde_json::from_str(&line).unwrap();
            if message["id"] == id {
                assert!(message.get("error").is_none(), "{method}: {message}");
                return message["result"].clone();
            }
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn example_project() -> Value {
    let path = format!(
        "{}/../../training-data/nta8800-example-terraced-dwelling.json",
        env!("CARGO_MANIFEST_DIR")
    );
    fn strip(value: Value) -> Value {
        match value {
            Value::Object(map) => Value::Object(
                map.into_iter()
                    .filter(|(_, item)| !item.is_null())
                    .map(|(key, item)| (key, strip(item)))
                    .collect(),
            ),
            Value::Array(items) => Value::Array(items.into_iter().map(strip).collect()),
            other => other,
        }
    }
    strip(serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap())
}

#[test]
fn mcp_server_speaks_the_protocol() {
    let mut server = Server::start();
    let init = server.request(
        1,
        "initialize",
        json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1" }
        }),
    );
    assert_eq!(init["protocolVersion"], "2025-06-18");
    assert_eq!(init["serverInfo"]["name"], "open-energy-studio-nta8800");
    assert!(init["capabilities"]["tools"].is_object());
    assert!(init["capabilities"]["resources"].is_object());
    server.send(json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));

    let tools = server.request(2, "tools/list", json!({}));
    let names: Vec<&str> = tools["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    assert_eq!(names.len(), nta8800_service::operations::operations().len());
    for name in [
        "calculate_project_performance",
        "assess_residential_survey",
        "assess_relabel",
        "get_version",
    ] {
        assert!(names.contains(&name), "{name}");
    }
    let schema = &tools["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["name"] == "calculate_project_performance")
        .unwrap()["inputSchema"];
    assert_eq!(schema["required"], json!(["project"]));

    let version = server.request(
        3,
        "tools/call",
        json!({ "name": "get_version", "arguments": {} }),
    );
    assert_eq!(version["isError"], false);
    assert_eq!(
        version["structuredContent"]["kernelVersion"],
        nta8800_core::KERNEL_VERSION
    );

    let performance = server.request(
        4,
        "tools/call",
        json!({ "name": "calculate_project_performance", "arguments": { "project": example_project() } }),
    );
    assert_eq!(performance["isError"], false, "{performance}");
    assert_eq!(
        performance["structuredContent"]["status"],
        "calculated_unverified"
    );
    let text = performance["content"][0]["text"].as_str().unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(text).unwrap(),
        performance["structuredContent"]
    );

    let refused = server.request(
        5,
        "tools/call",
        json!({ "name": "calculate_project_performance", "arguments": {} }),
    );
    assert_eq!(refused["isError"], true);
    assert_eq!(
        refused["structuredContent"]["code"],
        "missing_request_member"
    );
    assert_eq!(refused["structuredContent"]["path"], "project");

    let resources = server.request(6, "resources/list", json!({}));
    let uris: Vec<&str> = resources["resources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|resource| resource["uri"].as_str().unwrap())
        .collect();
    assert!(uris.contains(&"oes://examples/terraced-dwelling"));
    assert!(uris.contains(&"oes://api/openapi.json"));
    assert!(uris.contains(&"oes://manual/index"));

    let read = server.request(
        7,
        "resources/read",
        json!({ "uri": "oes://kernel/interpretations" }),
    );
    let text = read["contents"][0]["text"].as_str().unwrap();
    let groups: Value = serde_json::from_str(text).unwrap();
    assert!(!groups.as_array().unwrap().is_empty());
}

/// Lines the MCP library would leave unanswered get a JSON-RPC error, and
/// the server keeps answering afterwards: invalid and truncated JSON,
/// nesting deeper than 128 levels, non-object arguments and a line longer
/// than the cap (set to 4096 bytes here; 64 MiB by default).
#[test]
fn mcp_server_answers_malformed_lines() {
    let mut server = Server::start_with(&[("OES_MCP_MAX_LINE_BYTES", "4096")]);
    server.request(
        1,
        "initialize",
        json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1" }
        }),
    );
    server.send(json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));

    server.send_raw("{bad json");
    let bad = server.next();
    assert_eq!(bad["error"]["code"], -32700, "{bad}");
    assert_eq!(bad["id"], Value::Null);

    server.send_raw(r#"{"jsonrpc":"2.0","id":11,"method":"tools/list""#);
    let truncated = server.next();
    assert_eq!(truncated["error"]["code"], -32700, "{truncated}");
    assert_eq!(truncated["id"], 11);

    let deep = format!(
        r#"{{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{{"name":"get_version","arguments":{{"a":{}{}}}}}}}"#,
        "[".repeat(200),
        "]".repeat(200)
    );
    server.send_raw(&deep);
    let deep = server.next();
    assert_eq!(deep["error"]["code"], -32600, "{deep}");
    assert_eq!(deep["id"], 10);

    server.send(json!({ "jsonrpc": "2.0", "id": 12, "method": "tools/call",
        "params": { "name": "get_version", "arguments": [1] } }));
    let params = server.next();
    assert_eq!(params["error"]["code"], -32602, "{params}");
    assert_eq!(params["id"], 12);

    server.send_raw(&"x".repeat(5000));
    let long = server.next();
    assert_eq!(long["error"]["code"], -32600, "{long}");

    // Still answering.
    let version = server.request(
        13,
        "tools/call",
        json!({ "name": "get_version", "arguments": {} }),
    );
    assert_eq!(version["isError"], false);
}
