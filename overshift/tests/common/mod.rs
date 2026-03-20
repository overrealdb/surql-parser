//! DB helpers for integration tests (mem / docker).

// ─── In-memory mode ───

#[cfg(feature = "validate-mem")]
pub mod mem {
	use std::sync::OnceLock;
	use surrealdb::Surreal;
	use surrealdb::engine::any::Any;

	static DB: OnceLock<Surreal<Any>> = OnceLock::new();
	static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

	fn get_runtime() -> &'static tokio::runtime::Runtime {
		RT.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
	}

	fn get_db() -> &'static Surreal<Any> {
		DB.get_or_init(|| {
			get_runtime().block_on(async {
				let db = surrealdb::engine::any::connect("mem://").await.unwrap();
				db
			})
		})
	}

	/// Connect to the shared in-memory SurrealDB instance.
	///
	/// Returns the singleton `Surreal<Any>` handle. Tests should use unique
	/// NS/DB names to avoid interference.
	pub fn connect() -> Surreal<Any> {
		get_db().clone()
	}

	pub fn runtime() -> &'static tokio::runtime::Runtime {
		get_runtime()
	}
}

// ─── Docker mode (testcontainers) ───

#[cfg(feature = "validate-docker")]
pub mod docker {
	use std::sync::OnceLock;
	use surrealdb::Surreal;
	use surrealdb::engine::any::Any;

	struct TestDb {
		url: String,
		_container: testcontainers::ContainerAsync<testcontainers::GenericImage>,
	}

	static TEST_DB: OnceLock<TestDb> = OnceLock::new();
	static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

	fn get_runtime() -> &'static tokio::runtime::Runtime {
		RT.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
	}

	fn get_test_db() -> &'static TestDb {
		TEST_DB.get_or_init(|| {
			get_runtime().block_on(async {
				use testcontainers::core::{ContainerPort, WaitFor};
				use testcontainers::runners::AsyncRunner;
				use testcontainers::{GenericImage, ImageExt};
				let image = GenericImage::new("surrealdb/surrealdb", "v3")
					.with_exposed_port(ContainerPort::Tcp(8000))
					.with_wait_for(WaitFor::message_on_stdout("Started web server on"));
				let container = image
					.with_cmd([
						"start",
						"--user",
						"root",
						"--pass",
						"root",
						"--bind",
						"0.0.0.0:8000",
					])
					.start()
					.await
					.expect("Failed to start SurrealDB v3 (is Docker running?)");
				let port = container.get_host_port_ipv4(8000).await.unwrap();
				let url = format!("http://127.0.0.1:{port}");

				// Extra health check
				let client = reqwest::Client::new();
				for _ in 0..30 {
					if client.get(&format!("{url}/health")).send().await.is_ok() {
						break;
					}
					tokio::time::sleep(std::time::Duration::from_millis(200)).await;
				}

				TestDb {
					url,
					_container: container,
				}
			})
		})
	}

	/// Connect to the shared Docker SurrealDB instance.
	///
	/// Each call returns a fresh `Surreal<Any>` handle connected to the same
	/// container. Tests should use unique NS/DB names to avoid interference.
	pub fn connect() -> Surreal<Any> {
		let db_info = get_test_db();
		let rt = get_runtime();
		rt.block_on(async {
			let db = surrealdb::engine::any::connect(&db_info.url).await.unwrap();
			db.signin(surrealdb::opt::auth::Root {
				username: "root".to_string(),
				password: "root".to_string(),
			})
			.await
			.unwrap();
			db
		})
	}

	/// Get the tokio runtime for blocking on async operations.
	pub fn runtime() -> &'static tokio::runtime::Runtime {
		get_runtime()
	}
}
