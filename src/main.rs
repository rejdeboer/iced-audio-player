use iced_audio_player::scene::Scene;

use iced::executor;
use iced::time::Instant;
use iced::widget::{column, container, row, shader, button, slider, text};
use iced::window;
use iced::{
    Alignment, Application, Command, Element, Length, Subscription, Theme,
};
use iced_audio_player::icon::Icon;
use iced_audio_player::message::Message;
use iced_audio_player::player::Player;

fn main() -> iced::Result {
    AudioPlayer::run(iced::Settings {
        fonts: vec![include_bytes!("../fonts/icons.ttf").as_slice().into()],
        ..iced::Settings::default()
    })
}

struct AudioPlayer {
    last_updated: Instant,
    scene: Scene,
    player: Player,
    seek_bar_value: f32,
    seek_bar_dragging: bool,
    duration: f32,
}

impl Application for AudioPlayer {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                last_updated: Instant::now(),
                scene: Scene::new(),
                player: Player::new(),
                seek_bar_value: 0f32,
                seek_bar_dragging: false,
                duration: 0f32,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Audio player".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Tick(time) => {
                if !self.seek_bar_dragging {
                    self.seek_bar_value = self.player.get_position();
                }
                self.scene.update_spectrum(
                    &self.player.get_fft_spectrum(),
                    time - self.last_updated,
                );
                self.last_updated = time;
            }
            Message::Play => {
                self.player.play();
            }
            Message::Pause => {
                self.player.pause();
            }
            Message::LoadFile(path) => {
                self.player.load_file(path);
                self.duration = self.player.get_duration();
            }
            Message::SetPositionPreview(position) => {
                self.seek_bar_dragging = true;
                self.seek_bar_value = position;
            }
            Message::SetPosition => {
                self.player.set_position(self.seek_bar_value);
                self.seek_bar_dragging = false;
            }
        }

        Command::none()
    }

    #[allow(unused)]
    fn view(&self) -> Element<'_, Self::Message> {
        let shader =
            shader(&self.scene).width(Length::Fill).height(Length::Fill);

        let load_file_btn = button("Load file")
            .on_press(Message::LoadFile("./media/song.wav".into()));
        let play_btn = if self.player.is_playing() {
            button(Icon::PAUSE.into_element()).on_press(Message::Pause)
        } else {
            button(Icon::PLAY.into_element()).on_press(Message::Play)
        };

        let seek_bar = slider(
            0f32..=self.duration,
            self.seek_bar_value,
            Message::SetPositionPreview,
        )
        .on_release(Message::SetPosition);
        let time_played_label =
            text(seconds_to_minutes(self.seek_bar_value)).width(35);
        let duration_label = text(seconds_to_minutes(self.duration)).width(35);

        let top_controls = row![load_file_btn, play_btn,].spacing(10);

        let bottom_controls =
            row![time_played_label, seek_bar, duration_label].spacing(10);

        let controls = column![top_controls, bottom_controls,]
            .align_items(Alignment::Center)
            .padding(10)
            .spacing(10);

        let controls = column![
            top_controls,
            bottom_controls,
        ].align_items(Alignment::Center).padding(10).spacing(10);

        container(column![shader, controls].align_items(Alignment::Center))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        window::frames().map(Message::Tick)
    }
}

fn seconds_to_minutes(seconds: f32) -> String {
    let minutes = seconds as u32 / 60;
    let seconds_left = seconds as u32 % 60;
    format!("{}:{:0>2}", minutes, seconds_left)
}
