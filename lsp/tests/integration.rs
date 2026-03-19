//! Integration tests — full LSP protocol over in-memory transport.
//!
//! Spawns a real LSP server via duplex channels and sends JSON-RPC messages.
//! Verifies actual protocol responses, not just unit handler outputs.

use bytes::{Buf, BufMut, BytesMut};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::codec::{Decoder, Encoder};

// ─── LSP Codec (Content-Length framing) ───

struct LspCodec;

impl Decoder for LspCodec {
	type Item = Value;
	type Error = std::io::Error;

	fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Value>, Self::Error> {
		// Find "Content-Length: N\r\n\r\n"
		let header_end = src.windows(4).position(|w| w == b"\r\n\r\n");

		let header_end = match header_end {
			Some(pos) => pos,
			None => return Ok(None),
		};

		let header = std::str::from_utf8(&src[..header_end])
			.map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

		let content_length: usize = header
			.lines()
			.find_map(|line| {
				line.strip_prefix("Content-Length: ")
					.and_then(|v| v.trim().parse().ok())
			})
			.ok_or_else(|| {
				std::io::Error::new(std::io::ErrorKind::InvalidData, "missing Content-Length")
			})?;

		let total = header_end + 4 + content_length;
		if src.len() < total {
			return Ok(None); // Need more data
		}

		let _ = src.split_to(header_end + 4); // consume header
		let body = src.split_to(content_length);
		let value: Value = serde_json::from_slice(&body)
			.map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
		Ok(Some(value))
	}
}

impl Encoder<Value> for LspCodec {
	type Error = std::io::Error;

	fn encode(&mut self, item: Value, dst: &mut BytesMut) -> Result<(), Self::Error> {
		let body = serde_json::to_vec(&item)
			.map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
		let header = format!("Content-Length: {}\r\n\r\n", body.len());
		dst.put_slice(header.as_bytes());
		dst.put_slice(&body);
		Ok(())
	}
}

// ─── Test helpers ───

async fn send_raw(writer: &mut (impl AsyncWriteExt + Unpin), msg: &Value) {
	let body = serde_json::to_vec(msg).unwrap();
	let header = format!("Content-Length: {}\r\n\r\n", body.len());
	writer.write_all(header.as_bytes()).await.unwrap();
	writer.write_all(&body).await.unwrap();
	writer.flush().await.unwrap();
}

async fn recv_raw(reader: &mut (impl AsyncReadExt + Unpin)) -> Value {
	let mut buf = BytesMut::with_capacity(8192);
	let mut codec = LspCodec;

	loop {
		let mut tmp = [0u8; 4096];
		let n = tokio::time::timeout(std::time::Duration::from_secs(10), reader.read(&mut tmp))
			.await
			.expect("timeout waiting for LSP response")
			.expect("read failed");
		if n == 0 {
			panic!("server closed connection");
		}
		buf.extend_from_slice(&tmp[..n]);
		if let Some(msg) = codec.decode(&mut buf).unwrap() {
			return msg;
		}
	}
}

/// Read messages until we find one matching the predicate.
async fn recv_until(
	reader: &mut (impl AsyncReadExt + Unpin),
	pred: impl Fn(&Value) -> bool,
) -> Value {
	let mut buf = BytesMut::with_capacity(8192);
	let mut codec = LspCodec;
	let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);

	loop {
		let mut tmp = [0u8; 4096];
		let remaining = deadline - tokio::time::Instant::now();
		let n = tokio::time::timeout(remaining, reader.read(&mut tmp))
			.await
			.expect("timeout waiting for LSP message")
			.expect("read failed");
		if n == 0 {
			panic!("server closed connection before matching message found");
		}
		buf.extend_from_slice(&tmp[..n]);
		while let Some(msg) = codec.decode(&mut buf).unwrap() {
			if pred(&msg) {
				return msg;
			}
		}
	}
}

// ─── Spawn server ───

struct TestServer {
	to_server: tokio::io::DuplexStream,
	from_server: tokio::io::DuplexStream,
	_handle: tokio::task::JoinHandle<()>,
}

impl TestServer {
	async fn start() -> Self {
		let (client_read, server_write) = tokio::io::duplex(1024 * 64);
		let (server_read, client_write) = tokio::io::duplex(1024 * 64);

		let handle = tokio::spawn(async move {
			let (service, socket) = tower_lsp::LspService::new(surql_lsp::server::Backend::new);
			tower_lsp::Server::new(server_read, server_write, socket)
				.serve(service)
				.await;
		});

		TestServer {
			to_server: client_write,
			from_server: client_read,
			_handle: handle,
		}
	}

	async fn initialize(&mut self) -> Value {
		let init_request = json!({
			"jsonrpc": "2.0",
			"id": 1,
			"method": "initialize",
			"params": {
				"capabilities": {},
				"rootUri": null
			}
		});
		send_raw(&mut self.to_server, &init_request).await;
		let response = recv_raw(&mut self.from_server).await;

		// Send initialized notification
		let initialized = json!({
			"jsonrpc": "2.0",
			"method": "initialized",
			"params": {}
		});
		send_raw(&mut self.to_server, &initialized).await;

		response
	}

	async fn open_document(&mut self, uri: &str, text: &str) {
		let did_open = json!({
			"jsonrpc": "2.0",
			"method": "textDocument/didOpen",
			"params": {
				"textDocument": {
					"uri": uri,
					"languageId": "surql",
					"version": 1,
					"text": text
				}
			}
		});
		send_raw(&mut self.to_server, &did_open).await;
	}

	async fn request_completion(&mut self, uri: &str, line: u32, character: u32) -> Value {
		let req = json!({
			"jsonrpc": "2.0",
			"id": 2,
			"method": "textDocument/completion",
			"params": {
				"textDocument": { "uri": uri },
				"position": { "line": line, "character": character }
			}
		});
		send_raw(&mut self.to_server, &req).await;
		recv_until(&mut self.from_server, |msg| {
			msg.get("id") == Some(&json!(2))
		})
		.await
	}

	async fn request_formatting(&mut self, uri: &str) -> Value {
		let req = json!({
			"jsonrpc": "2.0",
			"id": 3,
			"method": "textDocument/formatting",
			"params": {
				"textDocument": { "uri": uri },
				"options": { "tabSize": 4, "insertSpaces": true }
			}
		});
		send_raw(&mut self.to_server, &req).await;
		recv_until(&mut self.from_server, |msg| {
			msg.get("id") == Some(&json!(3))
		})
		.await
	}

	async fn request_hover(&mut self, uri: &str, line: u32, character: u32) -> Value {
		let req = json!({
			"jsonrpc": "2.0",
			"id": 4,
			"method": "textDocument/hover",
			"params": {
				"textDocument": { "uri": uri },
				"position": { "line": line, "character": character }
			}
		});
		send_raw(&mut self.to_server, &req).await;
		recv_until(&mut self.from_server, |msg| {
			msg.get("id") == Some(&json!(4))
		})
		.await
	}

	async fn shutdown(&mut self) {
		let req = json!({
			"jsonrpc": "2.0",
			"id": 99,
			"method": "shutdown",
			"params": null
		});
		send_raw(&mut self.to_server, &req).await;
		let _ = recv_until(&mut self.from_server, |msg| {
			msg.get("id") == Some(&json!(99))
		})
		.await;

		let exit = json!({
			"jsonrpc": "2.0",
			"method": "exit",
			"params": null
		});
		send_raw(&mut self.to_server, &exit).await;
	}
}

// ─── Tests ───

#[tokio::test]
async fn initialize_returns_capabilities() {
	let mut server = TestServer::start().await;
	let resp = server.initialize().await;

	let caps = &resp["result"]["capabilities"];
	assert!(caps["textDocumentSync"].is_number() || caps["textDocumentSync"].is_object());
	assert!(caps["completionProvider"].is_object());
	assert!(
		caps["documentFormattingProvider"]
			.as_bool()
			.unwrap_or(false)
			|| caps["documentFormattingProvider"].is_object()
	);
	assert!(caps["hoverProvider"].as_bool().unwrap_or(false) || caps["hoverProvider"].is_object());
	assert!(caps["signatureHelpProvider"].is_object());

	server.shutdown().await;
}

#[tokio::test]
async fn diagnostics_published_on_open_invalid() {
	let mut server = TestServer::start().await;
	server.initialize().await;

	let uri = "file:///test.surql";
	server.open_document(uri, "SELEC * FROM user").await;

	// Wait for publishDiagnostics notification
	let notif = recv_until(&mut server.from_server, |msg| {
		msg.get("method") == Some(&json!("textDocument/publishDiagnostics"))
	})
	.await;

	let diags = &notif["params"]["diagnostics"];
	assert!(diags.is_array());
	assert!(
		!diags.as_array().unwrap().is_empty(),
		"expected diagnostics for invalid SQL"
	);
	assert_eq!(notif["params"]["uri"], uri);

	server.shutdown().await;
}

#[tokio::test]
async fn diagnostics_empty_on_open_valid() {
	let mut server = TestServer::start().await;
	server.initialize().await;

	let uri = "file:///valid.surql";
	server.open_document(uri, "SELECT * FROM user").await;

	let notif = recv_until(&mut server.from_server, |msg| {
		msg.get("method") == Some(&json!("textDocument/publishDiagnostics"))
	})
	.await;

	let diags = &notif["params"]["diagnostics"];
	assert!(
		diags.as_array().unwrap().is_empty(),
		"expected no diagnostics for valid SQL"
	);

	server.shutdown().await;
}

#[tokio::test]
async fn completion_returns_keywords() {
	let mut server = TestServer::start().await;
	server.initialize().await;

	let uri = "file:///comp.surql";
	server.open_document(uri, "").await;
	// Consume the publishDiagnostics notification first
	let _ = recv_until(&mut server.from_server, |msg| {
		msg.get("method") == Some(&json!("textDocument/publishDiagnostics"))
	})
	.await;

	let resp = server.request_completion(uri, 0, 0).await;
	let items = resp["result"]
		.as_array()
		.expect("completion result should be array");
	assert!(!items.is_empty(), "expected keyword completions");

	// Check that SELECT is among completions
	let has_select = items.iter().any(|item| item["label"] == "SELECT");
	assert!(has_select, "SELECT should be in completions");

	server.shutdown().await;
}

#[tokio::test]
async fn formatting_valid_document() {
	let mut server = TestServer::start().await;
	server.initialize().await;

	let uri = "file:///fmt.surql";
	server.open_document(uri, "SELECT  *   FROM   user").await;
	let _ = recv_until(&mut server.from_server, |msg| {
		msg.get("method") == Some(&json!("textDocument/publishDiagnostics"))
	})
	.await;

	let resp = server.request_formatting(uri).await;
	let result = &resp["result"];
	assert!(
		result.is_array(),
		"formatting should return text edits array"
	);
	let edits = result.as_array().unwrap();
	assert!(!edits.is_empty(), "should have at least one edit");
	assert!(edits[0]["newText"].is_string());

	server.shutdown().await;
}

#[tokio::test]
async fn formatting_invalid_returns_null() {
	let mut server = TestServer::start().await;
	server.initialize().await;

	let uri = "file:///bad.surql";
	server.open_document(uri, "SELEC * FROM user").await;
	let _ = recv_until(&mut server.from_server, |msg| {
		msg.get("method") == Some(&json!("textDocument/publishDiagnostics"))
	})
	.await;

	let resp = server.request_formatting(uri).await;
	assert!(
		resp["result"].is_null(),
		"formatting invalid SQL should return null"
	);

	server.shutdown().await;
}

#[tokio::test]
async fn hover_returns_null_on_empty() {
	let mut server = TestServer::start().await;
	server.initialize().await;

	let uri = "file:///hover.surql";
	server.open_document(uri, "SELECT * FROM user").await;
	let _ = recv_until(&mut server.from_server, |msg| {
		msg.get("method") == Some(&json!("textDocument/publishDiagnostics"))
	})
	.await;

	// Hover on space (col 7 = the * position, col 6 = space)
	let resp = server.request_hover(uri, 0, 7).await;
	// Without schema, hover returns null for unknown identifiers
	assert!(resp["result"].is_null());

	server.shutdown().await;
}

#[tokio::test]
async fn multiple_documents_independent() {
	let mut server = TestServer::start().await;
	server.initialize().await;

	let uri1 = "file:///a.surql";
	let uri2 = "file:///b.surql";

	server.open_document(uri1, "SELECT * FROM user").await;
	let notif1 = recv_until(&mut server.from_server, |msg| {
		msg.get("method") == Some(&json!("textDocument/publishDiagnostics"))
			&& msg["params"]["uri"] == uri1
	})
	.await;
	assert!(
		notif1["params"]["diagnostics"]
			.as_array()
			.unwrap()
			.is_empty()
	);

	server.open_document(uri2, "SELEC broken").await;
	let notif2 = recv_until(&mut server.from_server, |msg| {
		msg.get("method") == Some(&json!("textDocument/publishDiagnostics"))
			&& msg["params"]["uri"] == uri2
	})
	.await;
	assert!(
		!notif2["params"]["diagnostics"]
			.as_array()
			.unwrap()
			.is_empty()
	);

	server.shutdown().await;
}

#[tokio::test]
async fn full_lifecycle() {
	let mut server = TestServer::start().await;

	// 1. Initialize
	let init = server.initialize().await;
	assert!(init["result"]["capabilities"].is_object());

	// 2. Open valid document
	let uri = "file:///lifecycle.surql";
	server.open_document(uri, "SELECT * FROM user").await;
	let diag = recv_until(&mut server.from_server, |msg| {
		msg.get("method") == Some(&json!("textDocument/publishDiagnostics"))
	})
	.await;
	assert!(diag["params"]["diagnostics"].as_array().unwrap().is_empty());

	// 3. Request completion
	let comp = server.request_completion(uri, 0, 0).await;
	assert!(!comp["result"].as_array().unwrap().is_empty());

	// 4. Request formatting
	let fmt = server.request_formatting(uri).await;
	// Already formatted → null or empty edits
	let _ = fmt;

	// 5. Shutdown
	server.shutdown().await;
}
