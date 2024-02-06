mod renderer;
mod spectrometer;

use renderer::Renderer;

use iced::mouse;
use iced::time::Duration;
use iced::widget::shader;
use iced::{Rectangle, Size};

use crate::player::FftSpectrum;
use spectrometer::Spectrometer;

const RESOLUTION: usize = 80000;

pub const MAX: u32 = 500;

pub struct Scene {
    pub size: f32,
    spectrometer: Spectrometer,
    spectrum: Vec<f32>,
}

impl Scene {
    pub fn new() -> Self {
        let mut scene = Self {
            size: 0.2,
            spectrometer: Spectrometer::new(RESOLUTION),
            spectrum: vec![0f32; RESOLUTION],
        };

        scene
    }

    pub fn update(&mut self, fft_spectrum: FftSpectrum, dt: Duration) {
        self.spectrum = self.spectrometer.generate_spectrum(fft_spectrum, dt);
    }
}

impl<Message> shader::Program<Message> for Scene {
    type State = ();
    type Primitive = Primitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        Primitive::new(
            bounds,
            &self.spectrum,
        )
    }
}

#[derive(Debug)]
pub struct Primitive {
    vertices: Vec<renderer::vertex::Vertex>,
}

impl Primitive {
    pub fn new(
        bounds: Rectangle,
        spectrum: &[f32],
    ) -> Self {
        let vertices = renderer::vertex::generate_vertices(spectrum);
        Self {
            vertices,
        }
    }
}

impl shader::Primitive for Primitive {
    fn prepare(
        &self,
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: Rectangle,
        target_size: Size<u32>,
        _scale_factor: f32,
        storage: &mut shader::Storage,
    ) {
        if !storage.has::<Renderer>() {
            storage.store(Renderer::new(device, queue, format, target_size, RESOLUTION));
        }

        let pipeline = storage.get_mut::<Renderer>().unwrap();

        // upload data to GPU
        pipeline.update(
            queue,
            &self.vertices,
        );
    }

    fn render(
        &self,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        _target_size: Size<u32>,
        viewport: Rectangle<u32>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // at this point our pipeline should always be initialized
        let pipeline = storage.get::<Renderer>().unwrap();

        // render primitive
        pipeline.render(
            target,
            encoder,
            viewport,
        );
    }
}
