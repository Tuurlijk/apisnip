use crate::Mode;
use ratatui::style::{Color, Modifier, Style};
use supports_color::ColorLevel;
use terminal_light;

// Helper to calculate a gradient color based on distance from selected row
pub fn gradient_color(
    distance: usize,
    selected: bool,
    is_selected_item: bool,
    color_level: Option<ColorLevel>,
    default_foreground: (u8, u8, u8),
    color_mode: Mode,
) -> Style {
    // If this is the selected row, use reversed style
    if selected {
        return Style::default().add_modifier(Modifier::REVERSED | Modifier::ITALIC);
    }

    // If this is a selected item (✂️), use green/bold regardless of distance
    if is_selected_item {
        return Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD);
    }

    // For terminals with no color support, just return default style
    if color_level.is_none() {
        return Style::default();
    }

    // No effect for selected row and immediate neighbors
    if distance <= 0 {
        return Style::default();
    }

    // Maximum distance for gradient effect
    let max_distance = 20;

    // Calculate progress using a sine wave function instead of linear progression
    // Map distance to a value between 0 and PI/2 (0 to 90 degrees)
    let normalized_distance = (distance as f32).min(max_distance as f32) / max_distance as f32;
    let progress = (normalized_distance * std::f32::consts::PI / 2.0).sin();

    // Apply sine wave gradient based on terminal capabilities
    let foreground = default_foreground;

    // Calculate dimmed foreground color based on color mode
    let dimmed = calculate_dimmed_color(foreground, color_mode);

    // Calculate interpolated color with proper clamping based on color mode
    let color = interpolate_color(foreground, dimmed, progress, color_mode);

    // Create style with the calculated color
    match color_level {
        Some(level) if level.has_16m => {
            // For truecolor terminals, use RGB directly
            Style::default().fg(Color::Rgb(color.0, color.1, color.2))
        }
        Some(level) if level.has_256 => {
            // For 256-color terminals, convert to indexed color
            Style::default().fg(Color::Indexed(rgb_to_indexed(color.0, color.1, color.2)))
        }
        _ => {
            // For basic terminals, use simple dimming
            if progress > 0.5 {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            }
        }
    }
}

// Calculate dimmed foreground color based on color mode
pub fn calculate_dimmed_color(foreground: (u8, u8, u8), color_mode: Mode) -> (u8, u8, u8) {
    match color_mode {
        Mode::Dark => (
            (foreground.0 as f32 * 0.75).clamp(0.0, 255.0) as u8,
            (foreground.1 as f32 * 0.75).clamp(0.0, 255.0) as u8,
            (foreground.2 as f32 * 0.75).clamp(0.0, 255.0) as u8,
        ),
        Mode::Light => (
            (foreground.0 as f32 * 2.0).clamp(0.0, 255.0) as u8,
            (foreground.1 as f32 * 2.0).clamp(0.0, 255.0) as u8,
            (foreground.2 as f32 * 2.0).clamp(0.0, 255.0) as u8,
        ),
        _ => (
            (foreground.0 as f32 * 0.75).clamp(0.0, 255.0) as u8,
            (foreground.1 as f32 * 0.75).clamp(0.0, 255.0) as u8,
            (foreground.2 as f32 * 0.75).clamp(0.0, 255.0) as u8,
        ),
    }
}

// Interpolate between foreground and dimmed colors based on progress
pub fn interpolate_color(
    foreground: (u8, u8, u8),
    dimmed: (u8, u8, u8),
    progress: f32,
    color_mode: Mode,
) -> (u8, u8, u8) {
    let r = interpolate_component(foreground.0, dimmed.0, progress, color_mode);
    let g = interpolate_component(foreground.1, dimmed.1, progress, color_mode);
    let b = interpolate_component(foreground.2, dimmed.2, progress, color_mode);
    (r, g, b)
}

// Interpolate a single color component with proper clamping based on color mode
pub fn interpolate_component(fg: u8, dimmed: u8, progress: f32, color_mode: Mode) -> u8 {
    let value = fg as f32 + ((dimmed as f32 - fg as f32) * progress);

    // Clamp the value based on color mode
    let clamped = match color_mode {
        Mode::Dark => value.clamp(dimmed as f32, fg as f32),
        _ => value.clamp(fg as f32, dimmed as f32),
    };

    clamped as u8
}

// Helper function to convert hex color to (r,g,b) tuple
pub fn hex_to_rgb(hex: u32) -> (u8, u8, u8) {
    let r = ((hex >> 16) & 0xFF) as u8;
    let g = ((hex >> 8) & 0xFF) as u8;
    let b = (hex & 0xFF) as u8;
    (r, g, b)
}

// Convert RGB values to an indexed color (16-231)
pub fn rgb_to_indexed(r: u8, g: u8, b: u8) -> u8 {
    // Convert RGB to the 6x6x6 color cube (0-5 for each component)
    let r_index = (r as f32 / 256.0 * 6.0) as u8;
    let g_index = (g as f32 / 256.0 * 6.0) as u8;
    let b_index = (b as f32 / 256.0 * 6.0) as u8;

    // Ensure indices are in 0-5 range
    let r_idx = r_index.min(5);
    let g_idx = g_index.min(5);
    let b_idx = b_index.min(5);

    // Calculate the indexed color (16-231)
    16 + 36 * r_idx + 6 * g_idx + b_idx
}

// Set color preferences based on terminal background
pub fn set_color_preferences(color_mode: &mut Mode, default_foreground_color: &mut (u8, u8, u8)) {
    match terminal_light::luma() {
        Ok(luma) if luma > 0.85 => {
            // Light mode: use a dark gray (#333333)
            *default_foreground_color = hex_to_rgb(0x333333);
            *color_mode = Mode::Light;
        }
        Ok(luma) if luma < 0.2 => {
            // Dark mode: use a light gray (#C0C0C0)
            *default_foreground_color = hex_to_rgb(0xC0C0C0);
            *color_mode = Mode::Dark;
        }
        _ => {
            // Default to dark mode
            *default_foreground_color = hex_to_rgb(0xC0C0C0);
            *color_mode = Mode::Dark;
        }
    }
}
