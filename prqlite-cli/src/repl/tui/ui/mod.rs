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
        let lines = (app.input.width() as f32 / (chunks[2].width as f32 - 3.0)).floor() as u16;

        let mut coords = app.state.coords.insert;

        coords
            .set_max((chunks[2].width, chunks[2].height))
            .set_min((chunks[1].x, chunks[1].y));

        // !Fix: x out of range
        let x =
            chunks[2].x + ((app.input.width() as u16 + lines) % chunks[2].width) + 2 * lines + 1;

        let y = chunks[2].y + lines + 1;

        coords.set(if x > coords.max_x.unwrap() { 2 } else { x }, y);

        // !DEBUGGING
        // app.push_msg(
        //     "",
        //     &format!(
        //         "{:?}, lines: {lines}, max:{:?} , min: {:?}",
        //         coords.get(),
        //         coords.get_max(),
        //         coords.get_min()
        //     ),
        //     OutputType::Success,
        // );
        f.set_cursor(coords.x, coords.y);
    }
}
