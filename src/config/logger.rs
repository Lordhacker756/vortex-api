use tracing::info;
use tracing::Level;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub fn initialize_logger() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_level(true)
        .with_file(true)
        .with_target(true)
        .pretty()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "vortex_api=info,tower_http=info".into()),
        )
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set up logging");
    info!("Logger Initializaed:: ✅");
}
