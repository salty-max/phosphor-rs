use briks::{
    Application, Color, Command, Constraint, Direction, Event, Frame, KeyCode, Layout, Modifier,
    Style, run,
    widgets::{Block, Borders, Text},
};

struct State;

#[derive(PartialEq)]
enum Action {
    Quit,
}

impl Application for State {
    type Action = Action;

    fn on_event(&self, event: Event) -> Option<Self::Action> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => Some(Action::Quit),
                _ => None,
            },
            _ => None,
        }
    }

    fn update(&mut self, msg: Self::Action) -> Command {
        if msg == Action::Quit {
            return Command::Quit;
        }
        Command::None
    }

    fn draw(&self, frame: &mut Frame) {
        let [header_area, body_area, footer_area] = Layout::new(
            Direction::Vertical,
            vec![
                Constraint::Length(3),
                Constraint::Fill,
                Constraint::Length(1),
            ],
        )
        .split_to(frame.area());

        // --- HEADER ---
        let header_block = Block::new()
            .borders(Borders::BOTTOM)
            .padding(0)
            .style(Style::new().fg(Color::Rgb(0, 122, 204)));

        let header_inner = header_block.inner(header_area);
        frame.render_widget(header_block, header_area);

        let header_title = Text::new(" 🚀 BRIKS DASHBOARD v0.1.0 ").style(
            Style::new()
                .fg(Color::White)
                .bg(Color::Rgb(0, 122, 204))
                .modifier(Modifier::BOLD),
        );
        frame.render_widget(header_title, header_inner);

        // --- BODY ---
        let [sidebar_area, _, content_area] = Layout::new(
            Direction::Horizontal,
            vec![
                Constraint::Ratio(1, 4),
                Constraint::Length(1),
                Constraint::Fill,
            ],
        )
        .split_to(body_area);

        // Sidebar
        let sidebar_block = Block::new()
            .borders(Borders::ALL)
            .title(" Navigation ")
            .title_style(Style::new().fg(Color::Yellow).modifier(Modifier::BOLD))
            .style(Style::new().fg(Color::Rgb(100, 100, 100)));

        let sidebar_inner = sidebar_block.inner(sidebar_area);
        frame.render_widget(sidebar_block, sidebar_area);

        frame.render_area(sidebar_inner, |f| {
            f.write_str(0, 0, "1. Overview");
            f.write_str(0, 1, "2. Analytics");
            f.write_str(0, 2, "3. Settings");
        });

        // Main Content
        let content_block = Block::new()
            .borders(Borders::ALL)
            .title(" System Status ")
            .title_style(Style::new().fg(Color::Cyan).modifier(Modifier::BOLD))
            .style(Style::new().fg(Color::Rgb(0, 122, 204)));

        let content_inner = content_block.inner(content_area);
        frame.render_widget(content_block, content_area);

        frame.render_area(content_inner, |f| {
            f.with_style(Style::new().fg(Color::Green), |f2| {
                f2.write_str(0, 0, "CPU Usage: [||||------] 42%");
            });
            f.write_str(0, 2, "Memory:    [||||||----] 64%");
            f.write_str(0, 4, "Disk I/O:  Stable");
        });

        // --- FOOTER ---
        let footer_text = Text::new(" Q: Quit | S: Save | R: Refresh ")
            .style(Style::new().fg(Color::Rgb(100, 100, 100)));
        frame.render_widget(footer_text, footer_area);
    }
}

fn main() -> std::io::Result<()> {
    run(State)
}
