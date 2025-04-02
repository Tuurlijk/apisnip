#![allow(dead_code)]
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Offset, Rect};
use ratatui::prelude::{Line, Modifier, Span, Style, Widget};
use ratatui::style::Color;
use ratatui::widgets::Clear;

/// A widget to display keyboard shortcuts in the UI
#[derive(Clone, Default)]
pub struct Shortcuts {
    shortcuts: Vec<(String, String)>,
    separator: String,
    shortcut_label_style: Style,
    shortcut_key_style: Style,
    alignment: Alignment,
    padding_start: String,
    padding_end: String,
}

impl Shortcuts {
    /// Create a new shortcuts widget from a vector of (key, label) pairs
    pub fn from(values: Vec<(&str, &str)>) -> Self {
        Self {
            shortcuts: values.into_iter()
                .map(|(k, l)| (k.to_string(), l.to_string()))
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
        
        let mut spans = Vec::with_capacity(self.shortcuts.len() * 5 + 2);
        
        // Add start padding if configured
        if !self.padding_start.is_empty() {
            spans.push(Span::raw(&self.padding_start));
        }
        
        // Process each shortcut
        for (i, (key, label)) in self.shortcuts.iter().enumerate() {
            // Add separator before shortcut (except for the first one)
            if i > 0 {
                spans.push(Span::raw(&self.separator));
            }
            
            // Render the key-label pair
            if label.contains(key) {
                // Create mnemonic spans (key is part of the label)
                let first_char = key.chars().next().unwrap_or('?');
                
                if let Some(idx) = label.find(first_char) {
                    // Split the label around the key character
                    let before = &label[..idx];
                    let highlight = &label[idx..idx+1];
                    let after = &label[idx+1..];
                    
                    spans.push(Span::styled(before, self.shortcut_label_style));
                    spans.push(Span::styled(highlight, self.shortcut_key_style));
                    spans.push(Span::styled(after, self.shortcut_label_style));
                } else {
                    // Fallback to regular key + label
                    spans.push(Span::styled(key, self.shortcut_key_style));
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(label, self.shortcut_label_style));
                }
            } else {
                // Regular shortcut (key + label)
                spans.push(Span::styled(key, self.shortcut_key_style));
                spans.push(Span::raw(" "));
                spans.push(Span::styled(label, self.shortcut_label_style));
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
