use briks::{Application, Color, Command, Event, Frame, KeyCode, Modifier, Style, run};

struct Counter {
    value: i32,
}

enum Action {
    Increment,
    Decrement,
    Quit,
}

impl Application for Counter {
    type Action = Action;

    fn on_event(&self, event: Event) -> Option<Self::Action> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('+') => Some(Action::Increment),
                KeyCode::Char('-') => Some(Action::Decrement),
                KeyCode::Char('q') => Some(Action::Quit),
                _ => None,
            },
            _ => None,
        }
    }

    fn update(&mut self, msg: Self::Action) -> Command {
        match msg {
            Action::Increment => self.value += 1,
            Action::Decrement => self.value -= 1,
            Action::Quit => return Command::Quit,
        }
        Command::None
    }

    fn draw(&self, frame: &mut Frame) {
        frame.with_style(
            Style::new()
                .fg(Color::Magenta)
                .bg(Color::Rgb(0, 0, 255))
                .modifier(Modifier::BOLD),
            |f| {
                f.write_str(0, 0, format!("Count: {}\r\n", self.value).as_str());
            },
        );
        frame.write_str(0, 1, "Press +/-, q to quit");
    }
}

fn main() -> std::io::Result<()> {
    let counter = Counter { value: 0 };

    run(counter)
}
