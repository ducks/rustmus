use crate::app::App;
use crate::browser::BrowserItem;
use ratatui::{prelude::*, widgets::*};

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = app
        .browser
        .list
        .entries
        .iter()
        .map(|item| match item {
            BrowserItem::UpDirectory => ListItem::new("[DIR] .."),
            BrowserItem::Entry(path) => {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("???");
                let prefix = if path.is_dir() { "[DIR] " } else { "      " };
                ListItem::new(format!("{prefix}{name}"))
            }
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Browser").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
        .highlight_symbol("➤ ");

    frame.render_stateful_widget(list, area, &mut app.browser.list.state);
}
