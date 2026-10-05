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
        let mut child = Command::new(env!("CARGO_BIN_EXE_mcp"))
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
