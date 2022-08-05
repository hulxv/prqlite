use super::{App, CursorCoords, InputMode};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;
pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(5),
                Constraint::Percentage(75),
                Constraint::Percentage(15),
            ]
            .as_ref(),
        )
        .split(f.size());

    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("i", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to inserting commands and queries."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Insert => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to reading output, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to execute command or query"),
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    // List of output messages
    let output: Vec<ListItem> = app
        .state
        .history
        .items
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let mut content = vec![Spans::from(Span::styled(
                m.time.to_rfc2822(),
                Style::default().fg(Color::Yellow),
            ))];
            if i != 0 {
                content.insert(0, Spans::from(Span::raw("")))
            }
            (app.prompt.clone() + " " + &m.command.clone())
                .chars()
                .collect::<Vec<char>>()
                .chunks(app.prompt.len() + chunks[1].width as usize + 5)
                .map(|c| c.iter().collect::<String>())
                .for_each(|line| {
                    content.push(Spans::from(Span::styled(
                        line,
                        Style::default().fg(Color::Blue),
                    )))
                });
            match &m.output {
                Ok(o) => o.clone(),
                Err(e) => e.to_string(),
            }
            .chars()
            .collect::<Vec<char>>()
            .chunks(chunks[1].width as usize + 5)
            .map(|c| c.iter().collect::<String>())
            .for_each(|chunk| {
                chunk.split("\n").for_each(|line| {
                    content.push(Spans::from(Span::styled(
                        line.to_owned(),
                        Style::default().fg(if m.output.is_ok() {
                            Color::LightCyan
                        } else {
                            Color::Red
                        }),
                    )));
                })
            });
            ListItem::new(content)
        })
        .collect();

    // Output widget
    let output_list = List::new(output)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Output")
                .style(match app.input_mode {
                    InputMode::Insert => Style::default(),
                    InputMode::Normal => Style::default().fg(Color::Yellow),
                }),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_stateful_widget(output_list, chunks[1], &mut app.state.history.state);

    // Input widget

    let input = Paragraph::new(vec![Spans::from(Span::raw(app.input.as_str()))])
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Insert => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"))
        .wrap(Wrap { trim: true });
    f.render_widget(input, chunks[2]);

    let coords = match app.input_mode {
        InputMode::Normal => {
            let coords = &mut app.state.coords.normal;

            coords
                .set_max((chunks[1].width + 1, chunks[1].height + 1))
                .set_min((chunks[1].x + 1, chunks[1].y + 1));

            if coords.get() == CursorCoords::default().get() {
                coords.set(chunks[1].x + 1, chunks[1].y);
            }
            coords
        }

        InputMode::Insert => {
            let coords = &mut app.state.coords.insert;
            let lines = app.input.width() as u16 / chunks[2].width;

            coords
                .set_max((chunks[2].width, chunks[2].height))
                .set_min((chunks[1].x, chunks[1].y))
                .set(
                    chunks[2].x + ((app.input.width() as u16 + lines) % chunks[2].width) + 1,
                    chunks[2].y + lines + 1,
                )
        }
    };
    f.set_cursor(coords.x, coords.y)
}
