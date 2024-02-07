use std::path::PathBuf;
use iced_audio_player::scene::{Scene};

use iced::executor;
use iced::time::Instant;
use iced::widget::{column, container, row, shader, button, slider};
use iced::window;
use iced::{
    Alignment, Application, Command, Element, Length, Subscription,
    Theme,
};
use iced_audio_player::player::Player;

fn main() -> iced::Result {
    AudioPlayer::run(iced::Settings::default())
}

struct AudioPlayer {
    time: Instant,
    scene: Scene,
    player: Player,
    seek_bar_value: f32,
    duration: f32,
}

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    Play,
    Pause,
    LoadFile(PathBuf),
    SetPositionPreview(f32),
    SetPosition,
}

impl AudioPlayer {
    fn update_scene(&mut self, time: Instant) {
        let fft_spectrum = self.player.get_fft_spectrum();
        self.scene.update(fft_spectrum, time - self.time);
    }
}

impl Application for AudioPlayer {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                time: Instant::now(),
                scene: Scene::new(),
                player: Player::default(),
                seek_bar_value: 0f32,
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
                self.seek_bar_value = self.player.get_position();
                self.update_scene(time);
                self.time = time;
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
                self.seek_bar_value = position;
            }
            Message::SetPosition => {
                self.player.set_position(self.seek_bar_value);
            }
        }

        Command::none()
    }

    #[allow(unused)]
    fn view(&self) -> Element<'_, Self::Message> {
        let shader =
            shader(&self.scene).width(Length::Fill).height(Length::Fill);

        let load_file_btn = button("Load file").on_press(Message::LoadFile("./media/song.wav".into()));
        let play_btn = if self.player.is_playing {
            button("Pause").on_press(Message::Pause)
        } else {
            button("Play").on_press(Message::Play)
        };

        let seek_bar = slider(0f32..=self.duration, self.seek_bar_value, Message::SetPositionPreview)
            .on_release(Message::SetPosition);

        let top_controls = row![
            load_file_btn,
            play_btn,
        ];

        let bottom_controls = row![seek_bar];

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

    fn subscription(&self) -> Subscription<Self::Message> {
        window::frames().map(Message::Tick)
    }
}
