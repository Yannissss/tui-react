use crate::command::Command;
use crate::event::Event;

pub trait React {
    type Message: Send + 'static;

    fn init(&mut self) -> Command<Self::Message>;

    fn handle(&mut self, event: Event) -> Command<Self::Message>;

    fn update(&mut self, message: Self::Message) -> Command<Self::Message>;

    fn view(&self, frame: ());
}
