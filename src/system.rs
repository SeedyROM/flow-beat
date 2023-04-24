use color_eyre::Result;
use multiqueue::{MPMCReceiver, MPMCSender};

pub type ShutdownSender = MPMCSender<()>;
pub type ShutdownReceiver = MPMCReceiver<()>;

pub fn setup() -> Result<()> {
    setup_backtracing()?;
    setup_logging()
}

fn setup_backtracing() -> Result<()> {
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1");
    }

    color_eyre::install()
}

fn setup_logging() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    // Setup env-filter
    let filter = tracing_subscriber::EnvFilter::from_default_env();

    // Setup tracing subscriber
    let subscriber = tracing_subscriber::fmt().with_env_filter(filter).finish();

    // Set subscriber
    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

pub fn shutdown() -> (ShutdownSender, ShutdownReceiver) {
    let (shutdown_tx, shutdown_rx) = multiqueue::mpmc_queue::<()>(1);

    ctrlc::set_handler({
        let shutdown_tx = shutdown_tx.clone();
        move || {
            println!("");
            // Send two messages for each both the audio and graphics threads
            for _ in 0..2 {
                shutdown_tx
                    .try_send(())
                    .expect("Failed to send shutdown signal");
            }
        }
    })
    .unwrap();

    (shutdown_tx, shutdown_rx)
}
