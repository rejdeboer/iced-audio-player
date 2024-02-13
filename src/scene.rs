mod spectrometer;

use iced::mouse::Cursor;
use iced::time::Duration;
use iced::widget::canvas::{stroke, Cache, Geometry, Path, Stroke};
use iced::widget::{canvas, Canvas};
use iced::{Color, Element, Length, Point, Renderer, Theme};
use iced::{Rectangle, Size};

use crate::message::Message;
use crate::player::FftSpectrum;
use spectrometer::Spectrometer;

const RESOLUTION: usize = 2000;

pub struct Scene {
    spectrometer: Spectrometer,
    spectrum: Vec<f32>,
    cache: Cache,
}

impl Scene {
    pub fn new() -> Self {
        let scene = Self {
            spectrometer: Spectrometer::new(RESOLUTION),
            spectrum: vec![0f32; RESOLUTION],
            cache: Cache::default(),
        };

        scene
    }

    pub fn update_spectrum(
        &mut self,
        fft_spectrum: &FftSpectrum,
        dt: Duration,
    ) {
        self.cache.clear();
        self.spectrum = self.spectrometer.generate_spectrum(fft_spectrum, dt);
    }

    pub fn view(&self) -> Element<Message> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl<Message> canvas::Program<Message> for Scene {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let frame_size = frame.size();
            let points = get_points_for_spectrum(&self.spectrum, frame_size);

            let path = Path::new(|b| {
                b.move_to(Point::new(0f32, frame_size.height));

                for point in points {
                    b.line_to(point);
                }

                b.line_to(Point::new(frame_size.width, frame_size.height));
            });

            frame.fill(&path, Color::from_rgba8(108, 122, 137, 0.3));
            frame.stroke(
                &path,
                Stroke {
                    style: stroke::Style::Solid(theme.palette().text),
                    width: 1.0,
                    ..Stroke::default()
                },
            );
        });

        vec![geometry]
    }
}

fn get_points_for_spectrum(data: &[f32], frame_size: Size) -> Vec<Point> {
    let mut points: Vec<Point> = vec![];
    let step_size = frame_size.width / data.len() as f32;

    for (i, vertex) in data.iter().enumerate() {
        let x = i as f32 * step_size;
        let y = frame_size.height - vertex * frame_size.height * 0.7;

        points.push(Point::new(x, y));
    }

    points
}
