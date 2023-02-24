use crossterm::{
    cursor,
    event::{Event, KeyCode},
    execute, queue, terminal,
};
use std::{
    io::{stdout, Write},
    time::Duration,
};
use tui_react::prelude::*;

struct App(i32);

impl React for App {
    type Message = i32;

    fn init(&mut self) -> Command<Self::Message> {
        Command::stream(stream! {
            for i in 0..10 {
                yield 7 * i - 19;
            }
        })
    }

    fn handle(&mut self, event: Event) -> Command<Self::Message> {
        match event {
            Event::Key(event) if event.code == KeyCode::Up => Command::instant(5),
            Event::Key(event) if event.code == KeyCode::Down => Command::instant(-5),
            Event::Key(event) if event.code == KeyCode::Right => Command::single(async {
                tokio::time::sleep(Duration::from_millis(200)).await;
                10
            }),
            Event::Key(event) if event.code == KeyCode::Left => Command::single(async {
                tokio::time::sleep(Duration::from_millis(200)).await;
                -10
            }),
            Event::Key(event) if event.code == KeyCode::Char(' ') => {
                Command::stream(stream! {
                    for i in 0..5 {
                        yield 2 * i - 3;
                        tokio::time::sleep(Duration::from_millis(200)).await;
                    }
                })
            }
            _ => Command::none(),
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        // let old = self.0;
        self.0 = self.0 + message;
        // println!(
        //     "App::update() := Received: {} | App({}) => App({})",
        //     message, old, self.0
        // );
        Command::none()
    }

    fn view(&self, _frame: ()) {
        queue!(stdout(), terminal::Clear(terminal::ClearType::All)).unwrap();
        queue!(stdout(), cursor::MoveTo(1, 1)).unwrap();
        write!(stdout(), "App::view() := {}", self.0).unwrap();
        stdout().flush().unwrap();
    }
}

fn main() {
    let exec = Executor::new(250);
    let app = App(0);
    exec.run(app).unwrap();
}
