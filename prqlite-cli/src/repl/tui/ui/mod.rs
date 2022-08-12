mod components;
mod traits;
use components::*;

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

    f.render_widget(HelpMessage::widget(app), chunks[0]);
    f.render_stateful_widget(
        History::widget(app, chunks[1]),
        chunks[1],
        &mut app.state.history.state,
    );
    f.render_widget(Input::widget(app), chunks[2]);

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
