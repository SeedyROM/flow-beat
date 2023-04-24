use std::thread;

use color_eyre::Result;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, SizedSample, Stream, StreamConfig,
};
use crossbeam::channel::{Receiver, Sender};
use tracing::{info, warn};

pub struct Audio {
    commands_rx: Receiver<AudioCommand>,
    shutdown_rx: Receiver<()>,
    device: cpal::Device,
}

impl Audio {
    pub fn new(commands_rx: Receiver<AudioCommand>, shutdown_rx: Receiver<()>) -> Result<Self> {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .expect("no output device available");

        Ok(Self {
            commands_rx,
            shutdown_rx,
            device,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        let config = self.device.default_output_config()?;
        let sample_rate = config.sample_rate().0 as f32;
        let channels = config.channels() as usize;

        info!("Sample rate: {}", sample_rate);
        info!("Channels: {}", channels);

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => self.stream::<f32>(config.into(), sample_rate, channels)?,
            cpal::SampleFormat::I16 => self.stream::<i16>(config.into(), sample_rate, channels)?,
            cpal::SampleFormat::U16 => self.stream::<u16>(config.into(), sample_rate, channels)?,
            _ => panic!("unsupported sample format"),
        };

        info!("Stream starting...");
        stream.play()?;

        self.shutdown_rx
            .recv()
            .expect("Failed to receive shutdown signal");

        Ok(())
    }

    fn stream<T>(
        &mut self,
        config: StreamConfig,
        _sample_rate: f32,
        _channels: usize,
    ) -> Result<Stream>
    where
        T: SizedSample + FromSample<f32>,
    {
        let commands_rx = self.commands_rx.clone();
        let mut amplitude = 0.0;
        let stream = self.device.build_output_stream(
            &config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                match commands_rx.try_recv() {
                    Ok(command) => match command {
                        AudioCommand::SetAmplitude(new_amplitude) => {
                            amplitude = new_amplitude;
                        }
                    },
                    Err(err) => match err {
                        crossbeam::channel::TryRecvError::Empty => {}
                        crossbeam::channel::TryRecvError::Disconnected => {
                            warn!("Audio command channel disconnected");
                        }
                    },
                }

                for sample in data.iter_mut() {
                    let x = rand::random::<f32>() * 2.0 - 1.0;
                    *sample = T::from_sample(x * amplitude);
                }
            },
            |err| {
                info!("Error occurred on audio stream: {}", err);
            },
            None,
        )?;

        Ok(stream)
    }
}

pub enum AudioCommand {
    SetAmplitude(f32),
}

pub fn run(commands_rx: Receiver<AudioCommand>, shutdown_rx: Receiver<()>) {
    let mut audio = Audio::new(commands_rx, shutdown_rx).expect("failed to create audio");

    thread::spawn(move || {
        audio.start().expect("failed to start audio");
    });
}

pub fn commands() -> (Sender<AudioCommand>, Receiver<AudioCommand>) {
    crossbeam::channel::unbounded()
}
