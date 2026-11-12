//! Bayleaf shell UI. Placeholder until ADR-0002 picks the toolkit.

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    tracing::info!("bayleaf-shell: not implemented yet");
    Ok(())
}
