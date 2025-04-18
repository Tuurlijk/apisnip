#![allow(dead_code)]
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Offset, Rect};
use ratatui::prelude::{Line, Modifier, Span, Style, Widget};
use ratatui::style::Color;
use ratatui::widgets::Clear;

/// Enum to define shortcut structures for the constructor
pub enum Shortcut<'a> {
    Pair(&'a str, &'a str),
    Trio(&'a str, &'a str, &'a str),
}

/// A widget to display keyboard shortcuts in the UI
#[derive(Clone, Default)]
pub struct Shortcuts {
    shortcuts: Vec<(String, String, Option<String>)>,
    separator: String,
    shortcut_label_style: Style,
    shortcut_key_style: Style,
    alignment: Alignment,
    padding_start: String,
    padding_end: String,
}

impl Shortcuts {
    /// Create a new shortcuts widget from a vector of ShortcutDef enums
    pub fn new(values: Vec<Shortcut>) -> Self {
        Self {
            shortcuts: values
                .into_iter()
                .map(|def| match def {
                    Shortcut::Pair(k, l) => (k.to_string(), l.to_string(), None),
                    Shortcut::Trio(k1, l, k2) => {
                        (k1.to_string(), l.to_string(), Some(k2.to_string()))
                    }
                })
                .collect(),
            separator: " | ".to_string(),
            shortcut_label_style: Style::default().add_modifier(Modifier::BOLD),
            shortcut_key_style: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            alignment: Alignment::Right,
            padding_start: " ".to_string(),
            padding_end: " ".to_string(),
        }
    }

    /// Get the line representation of all shortcuts
    pub fn as_line(&self) -> Line {
        if self.shortcuts.is_empty() {
            return Line::default().alignment(self.alignment);
        }

        let mut spans = Vec::with_capacity(self.shortcuts.len() * 5 + 2); // Adjusted capacity estimate

        // Add start padding if configured
        if !self.padding_start.is_empty() {
            spans.push(Span::raw(&self.padding_start));
        }

        // Process each shortcut
        for (i, (key1, label, key2_opt)) in self.shortcuts.iter().enumerate() {
            // Add separator before shortcut (except for the first one)
            if i > 0 {
                spans.push(Span::raw(&self.separator));
            }

            match key2_opt {
                Some(key2) => {
                    // Three-part shortcut: key1 label key2
                    spans.push(Span::styled(key1, self.shortcut_key_style));
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(label, self.shortcut_label_style));
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(key2, self.shortcut_key_style));
                }
                None => {
                    // Two-part shortcut: Try mnemonic highlighting first
                    if label.contains(key1) {
                        // Create mnemonic spans (key is part of the label)
                        let first_char = key1.chars().next().unwrap_or('?'); // Use key1

                        if let Some(idx) = label.find(first_char) {
                            // Ensure the key is not empty before slicing
                            let key_len = key1.chars().count();
                            if key_len > 0 && idx + key_len <= label.chars().count() {
                                // Split the label around the key
                                let before = label.chars().take(idx).collect::<String>();
                                let highlight =
                                    label.chars().skip(idx).take(key_len).collect::<String>();
                                let after = label.chars().skip(idx + key_len).collect::<String>();

                                spans.push(Span::styled(before, self.shortcut_label_style));
                                spans.push(Span::styled(highlight, self.shortcut_key_style));
                                spans.push(Span::styled(after, self.shortcut_label_style));
                            } else {
                                // Fallback if slicing indices are invalid (should be rare)
                                spans.push(Span::styled(key1, self.shortcut_key_style));
                                spans.push(Span::raw(" "));
                                spans.push(Span::styled(label, self.shortcut_label_style));
                            }
                        } else {
                            // Fallback to regular key + label if char not found
                            spans.push(Span::styled(key1, self.shortcut_key_style));
                            spans.push(Span::raw(" "));
                            spans.push(Span::styled(label, self.shortcut_label_style));
                        }
                    } else {
                        // Regular shortcut (key + label)
                        spans.push(Span::styled(key1, self.shortcut_key_style));
                        spans.push(Span::raw(" "));
                        spans.push(Span::styled(label, self.shortcut_label_style));
                    }
                }
            }
        }

        // Add end padding if configured
        if !self.padding_end.is_empty() {
            spans.push(Span::raw(&self.padding_end));
        }

        Line::from(spans).alignment(self.alignment)
    }

    /// Set a custom separator between shortcuts
    pub fn with_separator(mut self, separator: &str) -> Self {
        self.separator = separator.to_string();
        self
    }

    /// Set the style for shortcut keys
    pub fn with_key_style(mut self, style: Style) -> Self {
        self.shortcut_key_style = style;
        self
    }

    /// Set the style for shortcut labels
    pub fn with_label_style(mut self, style: Style) -> Self {
        self.shortcut_label_style = style;
        self
    }

    /// Set the alignment for the shortcuts line
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Set padding at the start of the shortcuts
    pub fn with_start_padding(mut self, padding: &str) -> Self {
        self.padding_start = padding.to_string();
        self
    }

    /// Set padding at the end of the shortcuts
    pub fn with_end_padding(mut self, padding: &str) -> Self {
        self.padding_end = padding.to_string();
        self
    }
}

impl Widget for Shortcuts {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line = self.as_line();
        let line_width = line.width() as i32;
        let delta = area.width as i32 - line_width;

        // Clear the area where we'll render the shortcuts
        if delta > 0 {
            let area_to_clear = area.offset(Offset { x: delta, y: 0 }).clamp(area);
            Clear.render(area_to_clear, buf);
        }

        // Render the line with shortcuts
        line.render(area, buf);
    }
}
