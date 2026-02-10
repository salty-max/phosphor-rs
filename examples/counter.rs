use phosphor::{
    Application, Color, Command, Constraint, Direction, Event, Frame, KeyCode, Layout, Modifier,
    Style, Widget, run,
    widgets::{Block, Borders, Text},
};

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
        let [header, body, footer] = Layout::new(
            Direction::Vertical,
            vec![
                Constraint::Length(1),
                Constraint::Fill,
                Constraint::Length(1),
            ],
        )
        .split_to(frame.area());

        frame.render_widget(
            Text::new("Counter Example").style(Style::new().modifier(Modifier::BOLD)),
            header,
        );

        let block = Block::new()
            .borders(Borders::ALL)
            .title("Hello")
            .style(Style::new().fg(Color::Magenta))
            .title_style(Style::new().fg(Color::Green).modifier(Modifier::BOLD));

        let inner_area = block.inner(body);
        frame.render_widget(block, body);

        frame.render_widget(Text::new(format!("Count: {}", self.value)), inner_area);

        frame.render_widget(
            Text::new("Press +/-, q to quit.").style(Style::new().fg(Color::Rgb(128, 128, 128))),
            footer,
        );
    }
}

fn main() -> std::io::Result<()> {
    let counter = Counter { value: 0 };

    run(counter)
}
