use std::path::PathBuf;
use iced_audio_player::scene::{Scene};

use iced::executor;
use iced::time::Instant;
use iced::widget::{column, container, row, shader, text, button};
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
}

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    Play,
    Pause,
    LoadFile(PathBuf),
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
                let fft_spectrum = self.player.get_fft_spectrum();
                self.scene.update(fft_spectrum, time - self.time);
                self.time = time;
            },
            Message::Play => {
                self.player.play();
            },
            Message::Pause => {
                self.player.pause();
            },
            Message::LoadFile(path) => {
                self.player.load_file(path);
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let shader =
            shader(&self.scene).width(Length::Fill).height(Length::Fill);

        let controls = column![
            row![
                button("Load file").on_press(Message::LoadFile("./media/song.wav".into())),
                button("Play").on_press(Message::Play),
                button("Pause").on_press(Message::Pause),
            ]
        ];

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

fn control<'a>(
    label: &'static str,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    row![text(label), control.into()].spacing(10).into()
}
