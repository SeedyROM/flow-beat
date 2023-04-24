use color_eyre::Result;

mod audio;
mod graphics;
mod parameter;
mod system;

fn main() -> Result<()> {
    system::setup()?;

    let (shutdown_tx, shutdown_rx) = system::shutdown();
    let (commands_tx, commands_rx) = audio::commands();
    audio::run(commands_rx, shutdown_rx);
    graphics::run(commands_tx, shutdown_tx)?;

    Ok(())
}
