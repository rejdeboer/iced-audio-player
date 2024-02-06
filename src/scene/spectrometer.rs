use std::time::Instant;
use crate::player::{BUFFER_SIZE, FftSpectrum};

const SMOOTHING_SPEED: f32 = 7.;

#[derive(Clone, Debug)]
struct FrequencyVertex {
    frequency: f32,
    volume: f32,
    position: f32,
}

pub struct Spectrometer {
    smoothing_buffer: [f32; BUFFER_SIZE],
    updated: Instant,
    resolution: usize,
}

impl FrequencyVertex {
    pub fn empty() -> Self {
        FrequencyVertex {
            frequency: 0.,
            position: 0.,
            volume: 0.,
        }
    }
}

impl Spectrometer {
    pub fn new(resolution: usize) -> Self {
        Spectrometer {
            smoothing_buffer: [0f32; BUFFER_SIZE],
            updated: Instant::now(),
            resolution,
        }
    }

    pub fn generate_spectrum(&mut self, fft_spectrum: FftSpectrum) -> Vec<f32> {
        let frequency_vertices = self.generate_frequency_vertices(fft_spectrum);
        let spectrum = frequency_vertices.iter()
            .map(|vertex| vertex.volume)
            .collect();

        spectrum
    }

    fn generate_frequency_vertices(&mut self, fft_spectrum: FftSpectrum) -> Vec<FrequencyVertex> {
        let mut vertices = fft_spectrum.values.iter()
            .enumerate()
            .map(|(i, magnitude)| FrequencyVertex {
                position: i as f32,
                volume: magnitude.clone(),
                frequency: (i as f32 + 1.) * fft_spectrum.bin_size,
            })
            .collect::<Vec<FrequencyVertex>>();

        self.apply_logarithmic_scaling(&mut vertices);
        self.apply_normalised_positioning(&mut vertices);
        self.apply_smoothing(&mut vertices);
        vertices = self.get_interpolated_vertices(vertices);

        vertices
    }

    fn apply_logarithmic_scaling(&self, vertices: &mut Vec<FrequencyVertex>) {
        for vertex in vertices {
            vertex.position = vertex.position.sqrt();
            vertex.volume = vertex.volume.sqrt();
        }
    }

    fn apply_normalised_positioning(&self, vertices: &mut Vec<FrequencyVertex>) {
        let max_vol = 5000.;
        let max_pos = match vertices.last() {
            Some(vertex) => vertex.position,
            None => 1f32,
        };
        for vertex in vertices {
            vertex.position /= max_pos;
            vertex.volume /= max_vol;
        }
    }

    fn apply_smoothing(&mut self, vertices: &mut Vec<FrequencyVertex>) {
        let now = Instant::now();
        let dt = now.duration_since(self.updated).as_secs_f32();

        for (i, vertex) in vertices.iter_mut().enumerate() {
            if !vertex.volume.is_nan() {
                self.smoothing_buffer[i] += (vertex.volume - self.smoothing_buffer[i]) * dt * SMOOTHING_SPEED;
            }
            vertex.volume = self.smoothing_buffer[i];
        };

        self.updated = now;
    }

    // Source: https://codeberg.org/BrunoWallner/audioviz/src/branch/main/src/spectrum/processor.rs
    fn get_interpolated_vertices(&self, vertices: Vec<FrequencyVertex>) -> Vec<FrequencyVertex> {
        let mut interpolated: Vec<FrequencyVertex> = vec![FrequencyVertex::empty(); self.resolution];

        let mut fb = vertices.clone();

        fb.insert(0, vertices.first().unwrap_or(&FrequencyVertex::empty()).clone());
        fb.push(FrequencyVertex::empty());

        if fb.len() > 4 {
            for i in 0..fb.len() - 3 {
                let y0 = fb[i].volume;
                let y1 = fb[i + 1].volume;
                let y2 = fb[i + 2].volume;
                let y3 = fb[i + 3].volume;

                let start = (fb[i + 1].position * interpolated.len() as f32) as usize;
                let end = (fb[i + 2].position * interpolated.len() as f32) as usize;

                if start < self.resolution && end < self.resolution {
                    for j in start..=end {
                        let pos: usize = j - start;
                        let gap_size = end - start;
                        let mut percentage: f32 = pos as f32 / gap_size as f32;
                        if percentage.is_nan() {
                            percentage = 0.5
                        }

                        let t = percentage;
                        let t2 = percentage.powi(2);

                        // explanation: http://paulbourke.net/miscellaneous/interpolation/
                        // cubic volume interpolation
                        let a0 = y3 - y2 - y0 + y1;
                        let a1 = y0 - y1 - a0;
                        let a2 = y2 - y0;
                        let a3 = y1;

                        // math magic
                        let volume = a0 * t * t2 + a1 * t2 + a2 * t + a3;

                        // linear freq interpolation
                        let f1 = fb[i + 1].frequency;
                        let f2 = fb[i + 2].frequency;
                        let frequency = f1 * (1.0 - t) + f2 * t;

                        if interpolated.len() > j && interpolated[j].volume < volume {
                            interpolated[j] = FrequencyVertex {
                                volume,
                                frequency,
                                position: 0f32,
                            };
                        }
                    }
                }
            }
        }

        interpolated
    }
}