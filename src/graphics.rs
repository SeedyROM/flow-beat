use std::{env, path::PathBuf};

use crossbeam::channel::Sender;
use ggez::{
    conf::{WindowMode, WindowSetup},
    event::EventHandler,
    glam::Vec2,
    graphics::{Canvas, Color, FontData, Text},
    Context, ContextBuilder, GameResult,
};
use tracing::warn;

use crate::{audio::AudioCommand, parameter::Parameter};

struct Window {
    amplitude: Parameter<f32>,
    commands_tx: Sender<AudioCommand>,
    shutdown_tx: Sender<()>,
}

impl Window {
    fn new(
        ctx: &mut Context,
        commands_tx: Sender<AudioCommand>,
        shutdown_tx: Sender<()>,
    ) -> GameResult<Self> {
        let amplitude = Parameter::new(0.0);

        let window = Self {
            amplitude,
            commands_tx,
            shutdown_tx,
        };

        window.load_assets(ctx)?;

        Ok(window)
    }

    fn load_assets(&self, ctx: &mut Context) -> GameResult<()> {
        self.load_fonts(ctx)?;

        Ok(())
    }

    fn load_fonts(&self, ctx: &mut Context) -> GameResult<()> {
        ctx.gfx.add_font(
            "MajorMonoDisplay",
            FontData::from_path(ctx, "/fonts/MajorMonoDisplay-Regular.ttf")?,
        );

        Ok(())
    }
}

impl EventHandler for Window {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if ctx.quit_requested {
            warn!("Received quit request");
            self.shutdown_tx.send(()).unwrap();
        }

        self.amplitude.set(ctx.mouse.position().x / 4000.0);

        if self.amplitude.has_changed() {
            self.commands_tx
                .send(AudioCommand::SetAmplitude(self.amplitude.get()))
                .unwrap();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let c = self.amplitude.get();

        let mut canvas = Canvas::from_frame(ctx, Color::from([c, c, c, 1.0]));

        canvas.draw(
            Text::new(format!("Amplitude: {:.2}", self.amplitude.get()))
                .set_font("MajorMonoDisplay")
                .set_scale(32.0),
            Vec2::new(32.0, 32.0),
        );

        canvas.finish(ctx)
    }
}

pub fn run(commands_tx: Sender<AudioCommand>, shutdown_tx: Sender<()>) -> GameResult {
    let (mut ctx, event_loop) = build_context()?;
    let window = Window::new(&mut ctx, commands_tx, shutdown_tx)?;

    ggez::event::run(ctx, event_loop, window)
}

fn build_context() -> GameResult<(Context, ggez::event::EventLoop<()>)> {
    ContextBuilder::new("flow-beat", "SeedyROM (Zack Kollar)")
        .window_setup(WindowSetup::default().title("Flow Beat"))
        .window_mode(WindowMode {
            logical_size: Some((1280.0, 720.0).into()),
            ..Default::default()
        })
        .add_resource_path(get_resource_path())
        .build()
        .map_err(Into::into)
}

fn get_resource_path() -> PathBuf {
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        PathBuf::from("./resources")
    }
}
