//! Bayleaf compositor. Placeholder until ADR-0001 picks the Smithay starting point.

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    tracing::info!(mode = ?bayleaf_ipc::Mode::default(), "bayleaf-comp: not implemented yet");
    Ok(())
}
