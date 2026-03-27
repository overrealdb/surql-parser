use rmcp::ServiceExt;
use surql_mcp::SurqlMcp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt()
		.with_env_filter(
			tracing_subscriber::EnvFilter::from_default_env()
				.add_directive(tracing::Level::INFO.into()),
		)
		.with_writer(std::io::stderr)
		.init();

	let server = SurqlMcp::new().await?;
	let transport = rmcp::transport::io::stdio();
	let ct = server.serve(transport).await?;
	ct.waiting().await?;
	Ok(())
}
