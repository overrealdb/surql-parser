/// Validate SQL depending on enabled features.
/// No-op without validation features — zero overhead.
#[allow(unused_variables)]
pub fn validate(sql: &str) {
	#[cfg(feature = "validate-docker")]
	docker::validate_sql(sql);

	#[cfg(all(feature = "validate-mem", not(feature = "validate-docker")))]
	mem::validate_sql(sql);
}

// ─── In-memory mode ───

#[cfg(feature = "validate-mem")]
mod mem {
	use std::sync::OnceLock;

	static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
	static DB: OnceLock<surrealdb::Surreal<surrealdb::engine::any::Any>> = OnceLock::new();

	fn get_runtime() -> &'static tokio::runtime::Runtime {
		RT.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
	}

	fn get_db() -> &'static surrealdb::Surreal<surrealdb::engine::any::Any> {
		DB.get_or_init(|| {
			get_runtime().block_on(async {
				let db = surrealdb::engine::any::connect("mem://").await.unwrap();
				db.use_ns("test").use_db("test").await.unwrap();
				db
			})
		})
	}

	pub fn validate_sql(sql: &str) {
		use std::hash::{BuildHasher, Hasher};
		let db = get_db();
		let rt = get_runtime();
		let mut h = std::hash::RandomState::new().build_hasher();
		h.write(sql.as_bytes());
		let db_name = format!("m_{:x}", h.finish());

		rt.block_on(async {
			db.use_ns("test").use_db(&db_name).await.unwrap();
			match db.query(sql).await {
				Ok(mut r) => {
					for (idx, err) in &r.take_errors() {
						let msg = err.to_string();
						if msg.contains("parse") || msg.contains("Unexpected") {
							panic!("SurrealDB (mem) rejected stmt {idx}: {msg}\nSQL: {sql}");
						}
					}
				}
				Err(e) => panic!("SurrealDB (mem) error: {e}\nSQL: {sql}"),
			}
		});
	}
}

// ─── Docker mode ───

#[cfg(feature = "validate-docker")]
mod docker {
	use std::sync::OnceLock;

	struct TestDb {
		url: String,
		client: reqwest::Client,
		_container: testcontainers::ContainerAsync<testcontainers_modules::surrealdb::SurrealDb>,
	}

	static TEST_DB: OnceLock<TestDb> = OnceLock::new();
	static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

	fn get_runtime() -> &'static tokio::runtime::Runtime {
		RT.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
	}

	fn get_test_db() -> &'static TestDb {
		TEST_DB.get_or_init(|| {
			get_runtime().block_on(async {
				use testcontainers::runners::AsyncRunner;
				let container = testcontainers_modules::surrealdb::SurrealDb::default()
					.start()
					.await
					.expect("Failed to start SurrealDB (is Docker running?)");
				let port = container.get_host_port_ipv4(8000).await.unwrap();
				let url = format!("http://127.0.0.1:{port}");
				let client = reqwest::Client::new();
				for _ in 0..30 {
					if client.get(&format!("{url}/health")).send().await.is_ok() {
						break;
					}
					tokio::time::sleep(std::time::Duration::from_millis(200)).await;
				}
				TestDb {
					url,
					client,
					_container: container,
				}
			})
		})
	}

	pub fn validate_sql(sql: &str) {
		use std::hash::{BuildHasher, Hasher};
		let db = get_test_db();
		let rt = get_runtime();
		let mut h = std::hash::RandomState::new().build_hasher();
		h.write(sql.as_bytes());
		let db_name = format!("d_{:x}", h.finish());

		rt.block_on(async {
			let resp = db
				.client
				.post(&format!("{}/sql", db.url))
				.header("Accept", "application/json")
				.header("NS", "test")
				.header("DB", &db_name)
				.basic_auth("root", Some("root"))
				.body(sql.to_string())
				.send()
				.await
				.unwrap_or_else(|e| panic!("HTTP failed: {e}\nSQL: {sql}"));

			let status = resp.status();
			let body = resp.text().await.unwrap_or_default();
			if status == reqwest::StatusCode::BAD_REQUEST {
				panic!("SurrealDB (docker) rejected (400):\n{body}\nSQL: {sql}");
			}
		});
	}
}
