use chrono::{DateTime, Local};
use ratatui::buffer::Buffer;
use ratatui::layout::{Margin, Rect};
use ratatui::prelude::{Line, Span, StatefulWidget};
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, BorderType, Clear, List, ListState, Widget};

/// logs widget
pub struct LogsWidget<'a> {
    logs: Vec<Line<'a>>,
}

impl<'a> LogsWidget<'a> {
    pub fn from(logs: &'a [(DateTime<Local>, &str)]) -> Self {
        Self {
            logs: logs.iter()
                .map(|(dt, log)| {
                    Line::from(vec![
                        Span::from(dt.time().format("%H:%M:%S").to_string()).style(Style::default()),
                        Span::from(" "),
                        Span::from(*log).style(Style::default()),
                    ])
                })
                .collect()
        }
    }
}

impl StatefulWidget for LogsWidget<'_> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Clear.render(area, buf);

        Block::new()
            .title(" internal logs ")
            .title_style(Style::default())
            .borders(Borders::ALL)
            .border_style(Style::default())
            .border_type(BorderType::Plain)
            .render(area, buf);

        let content_area = area.inner(Margin::new(2, 1));
        let logs = List::from_iter(self.logs);

        StatefulWidget::render(logs, content_area, buf, state);
    }
}
