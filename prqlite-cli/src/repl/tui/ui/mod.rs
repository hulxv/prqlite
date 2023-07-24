mod components;
mod traits;
use components::*;

use super::{App, InputMode};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
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
    f.render_widget(Input::widget(app, chunks[2]), chunks[2]);

    if let InputMode::Insert = app.input_mode {
        let lines = app.input.width() as u16 / (chunks[2].width);

        // !Fix: cursor coords isn't correct.
        app.state
            .coords
            .insert
            .set_max((chunks[2].width, chunks[2].height))
            .set_min((chunks[1].x, chunks[1].y))
            .set(
                chunks[2].x
                    + ((app.input.width() as u16 + lines) % chunks[2].width)
                    + 2 * lines
                    + 1,
                chunks[2].y + lines + 1,
            );

        f.set_cursor(app.state.coords.insert.x, app.state.coords.insert.y);
    }
}
