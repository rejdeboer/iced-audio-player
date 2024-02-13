use crate::message::Message;
use iced::widget::text;
use iced::{Element, Font};

pub enum Icon {
    PLAY,
    PAUSE,
}

impl Icon {
    pub fn into_element<'a>(self) -> Element<'a, Message> {
        match self {
            Icon::PLAY => icon_to_element('\u{E805}'),
            Icon::PAUSE => icon_to_element('\u{E807}'),
        }
    }
}

fn icon_to_element<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("icons");
    text(codepoint).font(ICON_FONT).into()
}
