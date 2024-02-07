use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{ChannelCount, Stream};
use hound::{WavReader};
use dasp::ring_buffer::{Fixed};
use std::path::{PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use apodize::hamming_iter;
use log::info;
use rustfft::{Fft, FftPlanner};
use rustfft::num_complex::Complex;

pub const BUFFER_SIZE: usize = 8192;

const MAX_FREQUENCY: f32 = 20000.;

pub struct FftSpectrum {
    pub values: Vec<f32>,
    pub bin_size: f32,
}

pub struct Player {
    pub sample_rate: cpal::SampleRate,
    pub channels: ChannelCount,
    pub stream: Option<Stream>,
    pub is_playing: bool,
    position: Arc<AtomicUsize>,
    buffer: Arc<Vec<i16>>,
    fft: Arc<dyn Fft<f32>>,
    rb: Arc<Mutex<Fixed<[i32; BUFFER_SIZE]>>>,
    hamming_window: Vec<f32>,
    output_len: usize,
}

impl Player {
    pub fn default() -> Self {
        let rb = Arc::new(Mutex::new(Fixed::from([0; BUFFER_SIZE])));
        let mut fft_planner = FftPlanner::new();
        let hamming_window: Vec<f32> = hamming_iter(BUFFER_SIZE)
            .map(|f| f as f32)
            .collect::<Vec<f32>>();

        Self {
            sample_rate: cpal::SampleRate(44100),
            channels: 2,
            stream: None,
            is_playing: false,
            position: Arc::new(AtomicUsize::new(0)),
            buffer: Arc::new(Vec::new()),
            fft: fft_planner.plan_fft_forward(BUFFER_SIZE),
            rb,
            hamming_window,
            output_len: BUFFER_SIZE,
        }
    }

    pub fn load_file(&mut self, path: PathBuf) {
        info!("Reading file: {}", path.file_name().unwrap().to_string_lossy());
        let mut reader = WavReader::open(path).expect("Failed to open wav file");

        let spec = reader.spec();
        let samples: Vec<i16> = reader.samples()
            .map(|sample| sample.expect("Failed to get sample"))
            .collect();

        info!("WAV SPEC: {} CHANNELS and sample rate of {}", spec.channels, spec.sample_rate);

        self.sample_rate = cpal::SampleRate(spec.sample_rate);
        self.channels = spec.channels;
        self.buffer = Arc::new(samples);

        let bin_size: f32 = self.sample_rate.0 as f32 / BUFFER_SIZE as f32 * 2.;
        self.output_len = (MAX_FREQUENCY / bin_size).ceil() as usize;

        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .expect("no output device available");

        let mut supported_configs_range = device
            .supported_output_configs()
            .expect("error while querying configs");

        let supported_config = supported_configs_range
            .find(|range| {
                range.sample_format() == cpal::SampleFormat::F32
                    && range.max_sample_rate() >= self.sample_rate
                    && range.min_sample_rate() <= self.sample_rate
                    && range.channels() == self.channels
            })
            .expect("Could not find supported audio config")
            .with_sample_rate(self.sample_rate);

        let rb = self.rb.clone();
        let buffer = self.buffer.clone();
        let position = self.position.clone();

        self.stream = Some(
            device.build_output_stream(
                &supported_config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut pos = position.load(Ordering::Relaxed);
                    let mut r_b = rb.lock().unwrap();
                    for sample in data.iter_mut() {
                        let value = if pos < buffer.len() { buffer[pos] } else { 0 };
                        *sample = cpal::Sample::from_sample(value);

                        let mut n = r_b.clone();
                        n.push(value as i32);
                        *r_b = n;

                        pos += 1;
                    }
                    let _ = position.compare_exchange(pos - data.len(), pos, Ordering::Relaxed, Ordering::Relaxed);
                },
                move |_err| panic!("ERROR"),
                None
            )
            .expect("Building output stream failed"),
        );
        self.is_playing = true;
    }

    pub fn play(&mut self) {
        if let Some(ref stream) = self.stream {
            info!("Playing stream");
            stream.play().unwrap();
            self.is_playing = true;
            return;
        }
    }

    pub fn set_position(&mut self, seconds: f32) {
        let new_position: usize = self.seconds_to_samples(seconds).max(0) as usize;
        self.position.store(new_position, Ordering::Relaxed);
    }

    pub fn get_position(&self) -> f32 {
        self.samples_to_seconds(self.position.load(Ordering::Relaxed))
    }

    pub fn get_duration(&self) -> f32 {
        self.samples_to_seconds(self.buffer.len())
    }

    pub fn pause(&mut self) {
        if let Some(ref stream) = self.stream {
            stream.pause().unwrap();
            self.is_playing = false;
        }
    }

    fn seconds_to_samples(&self, seconds: f32) -> i32 {
        (self.sample_rate.0 as f32 * seconds) as i32 * self.channels as i32
    }

    fn samples_to_seconds(&self, samples: usize) -> f32 {
        samples as f32 / self.channels as f32 / self.sample_rate.0 as f32
    }

    pub fn get_fft_spectrum(&self) -> FftSpectrum {
        let rb = *self.rb.lock().unwrap();

        let (left, right) = rb.slices();

        let data = &[left, right].concat();

        let mut buffer = data.iter()
            .enumerate()
            .map(|(i, sample)| Complex::new(self.hamming_window[i] * sample.clone() as f32, 0f32))
            .collect::<Vec<_>>();

        self.fft.process(&mut buffer);

        let values = buffer.iter()
            .take(self.output_len)
            .map(|elem| elem.norm())
            .collect::<Vec<_>>();

        let bin_size: f32 = self.sample_rate.0 as f32 / BUFFER_SIZE as f32 * 2.;

        FftSpectrum {
            values,
            bin_size,
        }
    }
}