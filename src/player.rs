use apodize::hamming_iter;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{ChannelCount, Device, Stream, SupportedStreamConfig};
use hound::WavReader;
use rtrb::{Consumer, Producer, RingBuffer};
use rustfft::num_complex::Complex;
use rustfft::{Fft, FftPlanner};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub const BUFFER_SIZE: usize = 4096;

const MAX_FREQUENCY: f32 = 20000.;

pub struct FftSpectrum {
    pub values: Vec<f32>,
    pub bin_size: f32,
}

impl FftSpectrum {
    pub fn empty() -> Self {
        FftSpectrum {
            values: vec![0f32; BUFFER_SIZE],
            bin_size: 0f32,
        }
    }
}

pub struct Player {
    device: Device,
    sample_rate: cpal::SampleRate,
    channels: ChannelCount,
    stream: Option<Stream>,
    is_playing: bool,
    position: Arc<AtomicU32>,
    duration: u32,
    fft: Arc<dyn Fft<f32>>,
    hamming_window: Vec<f32>,
    output_len: usize,
    fft_output: FftSpectrum,
    buffer_consumer: Option<Consumer<f32>>,
}

impl Player {
    pub fn new() -> Self {
        let mut fft_planner = FftPlanner::new();
        let hamming_window: Vec<f32> = hamming_iter(BUFFER_SIZE)
            .map(|f| f as f32)
            .collect::<Vec<f32>>();

        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .expect("no output device available");

        Self {
            device,
            sample_rate: cpal::SampleRate(44100),
            channels: 2,
            stream: None,
            is_playing: false,
            position: Arc::new(AtomicU32::new(0)),
            duration: 0,
            fft: fft_planner.plan_fft_forward(BUFFER_SIZE),
            hamming_window,
            output_len: BUFFER_SIZE,
            fft_output: FftSpectrum::empty(),
            buffer_consumer: None,
        }
    }

    pub fn load_file(&mut self, path: PathBuf) {
        let mut reader =
            WavReader::open(path).expect("Failed to open wav file");

        let spec = reader.spec();

        let (mut input_producer, mut input_consumer) =
            RingBuffer::new(BUFFER_SIZE * 3);

        self.sample_rate = cpal::SampleRate(spec.sample_rate);
        self.channels = spec.channels;
        self.position.store(0, Ordering::Relaxed);
        self.duration = reader.duration();
        let position = self.position.clone();
        let channels = self.channels.clone();

        std::thread::spawn(move || loop {
            let pos = position.load(Ordering::Relaxed);

            while input_producer.is_full() {
                std::thread::sleep(Duration::from_millis(100));
            }

            let mut chunk = input_producer
                .write_chunk(std::cmp::min(input_producer.slots(), BUFFER_SIZE))
                .unwrap();

            let mut samples = reader
                .samples::<i16>()
                .take(chunk.len())
                .map(|s| s.expect("Failed to read sample"))
                .collect::<Vec<i16>>();
            samples.resize(chunk.len(), 0);

            let (first, second) = chunk.as_mut_slices();
            let mid = first.len();

            first.copy_from_slice(&samples[..mid]);
            second.copy_from_slice(&samples[mid..]);

            chunk.commit_all();

            match position.compare_exchange(
                pos,
                pos + (samples.len() as u16 / channels) as u32,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => (),
                // Position was changed from set_position
                Err(p) => reader.seek(p).expect("Failed to seek"),
            };
        });

        let bin_size: f32 = self.sample_rate.0 as f32 / BUFFER_SIZE as f32 * 2.;
        self.output_len = (MAX_FREQUENCY / bin_size).ceil() as usize;

        let (mut output_producer, output_consumer) =
            RingBuffer::new(BUFFER_SIZE * 3);
        self.buffer_consumer = Some(output_consumer);

        let supported_config = self.get_stream_config();

        self.stream = Some(
            self.device
                .build_output_stream(
                    &supported_config.into(),
                    move |data, _| {
                        process_samples(
                            data,
                            &mut input_consumer,
                            &mut output_producer,
                        )
                    },
                    move |_err| panic!("ERROR"),
                    None,
                )
                .expect("Building output stream failed"),
        );
        self.is_playing = true;
    }

    pub fn play(&mut self) {
        if let Some(ref stream) = self.stream {
            stream.play().unwrap();
            self.is_playing = true;
        }
    }

    pub fn set_position(&mut self, seconds: f32) {
        let new_position = self.seconds_to_samples(seconds).max(0) as u32;
        self.position.store(new_position, Ordering::Relaxed);
    }

    pub fn get_position(&self) -> f32 {
        self.samples_to_seconds(self.position.load(Ordering::Relaxed))
    }

    pub fn get_duration(&self) -> f32 {
        self.samples_to_seconds(self.duration)
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn is_streaming(&self) -> bool {
        self.stream.is_some()
    }

    pub fn pause(&mut self) {
        if let Some(ref stream) = self.stream {
            stream.pause().unwrap();
            self.is_playing = false;
        }
    }

    pub fn get_fft_spectrum(&mut self) -> &FftSpectrum {
        if self.buffer_consumer == None {
            return &self.fft_output;
        }

        let consumer = self.buffer_consumer.as_mut().unwrap();
        if consumer.slots() < BUFFER_SIZE {
            return &self.fft_output;
        }

        let mut buffer: Vec<Complex<f32>> = vec![];
        for i in 0..BUFFER_SIZE {
            let sample = consumer.pop().unwrap();
            buffer.push(Complex::new(
                self.hamming_window[i] * sample.clone(),
                0f32,
            ));
        }

        self.fft.process(&mut buffer);

        self.fft_output.values = buffer
            .iter()
            .take(self.output_len)
            .map(|elem| elem.norm())
            .collect::<Vec<_>>();

        self.fft_output.bin_size =
            self.sample_rate.0 as f32 / BUFFER_SIZE as f32 * 2.;

        &self.fft_output
    }

    fn seconds_to_samples(&self, seconds: f32) -> i32 {
        (self.sample_rate.0 as f32 * seconds) as i32
    }

    fn samples_to_seconds(&self, samples: u32) -> f32 {
        samples as f32 / self.sample_rate.0 as f32
    }

    fn get_stream_config(&self) -> SupportedStreamConfig {
        let mut supported_configs_range = self
            .device
            .supported_output_configs()
            .expect("error while querying configs");

        supported_configs_range
            .find(|range| {
                range.sample_format() == cpal::SampleFormat::F32
                    && range.max_sample_rate() >= self.sample_rate
                    && range.min_sample_rate() <= self.sample_rate
                    && range.channels() == self.channels
            })
            .expect("Could not find supported audio config")
            .with_sample_rate(self.sample_rate)
    }
}

fn process_samples(
    samples: &mut [f32],
    input_consumer: &mut Consumer<i16>,
    output_producer: &mut Producer<f32>,
) {
    for sample in samples.iter_mut() {
        let value = input_consumer.pop().unwrap_or(0);
        *sample = cpal::Sample::from_sample(value);

        // If the buffer is full, we should do one of the following:
        // - Increase BUFFER_SIZE
        // - Optimize UI thread
        // - Decrease spectrum resolution
        if let Some(_) = output_producer.push(value as f32).err() {
            eprintln!("Output buffer is full");
        }
    }
}
