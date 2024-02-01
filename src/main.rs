use iced_audio_player::scene::{Scene};

use iced::executor;
use iced::time::Instant;
use iced::widget::{column, container, row, shader, text};
use iced::window;
use iced::{
    Alignment, Application, Command, Element, Length, Subscription,
    Theme,
};

fn main() -> iced::Result {
    AudioPlayer::run(iced::Settings::default())
}

struct AudioPlayer {
    start: Instant,
    scene: Scene,
}

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
}

impl Application for AudioPlayer {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                start: Instant::now(),
                scene: Scene::new(),
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
                self.scene.update(time - self.start);
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let shader =
            shader(&self.scene).width(Length::Fill).height(Length::Fill);

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
