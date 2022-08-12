use super::{traits::*, App, InputMode};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget as TuiStatefulWidget,
        Widget as TuiWidget, Wrap,
    },
    Frame,
};
pub struct History;
impl History {
    pub fn widget(app: &App, chunk: Rect) -> impl TuiStatefulWidget<State = ListState> {
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
                    .chunks(app.prompt.len() + chunk.width as usize + 5)
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
                .chunks(chunk.width as usize + 5)
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

        List::new(output)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Output")
                    .style(match app.input_mode {
                        InputMode::Insert => Style::default(),
                        InputMode::Normal => Style::default().fg(Color::Yellow),
                    }),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
    }
}

pub struct Input;
impl Input {
    pub fn widget(app: &App) -> impl TuiWidget + '_ {
        Paragraph::new(vec![Spans::from(Span::raw(app.input.as_str()))])
            .style(match app.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Insert => Style::default().fg(Color::Yellow),
            })
            .block(Block::default().borders(Borders::ALL).title("Input"))
            .wrap(Wrap { trim: true })
    }
}

pub struct HelpMessage;
impl HelpMessage {
    pub fn widget(app: &App) -> impl TuiWidget + '_ {
        let (msg, style) = match app.input_mode {
            InputMode::Normal => (
                vec![
                    Span::raw("Press "),
                    Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to exit, "),
                    Span::styled("i | Tab", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to inserting commands and queries."),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Insert => (
                vec![
                    Span::raw("Press "),
                    Span::styled("Esc | Tab", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to reading output, "),
                    Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to execute command or query"),
                ],
                Style::default(),
            ),
        };
        let mut text = Text::from(Spans::from(msg));
        text.patch_style(style);
        Paragraph::new(text)
    }
}
