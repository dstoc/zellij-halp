use ratatui::{
    backend::Backend,
    buffer::Cell,
    layout::Size,
    style::{Color, Modifier, Style},
    Frame, Terminal,
};
use std::collections::HashMap;
use std::error::Error;
use std::io;

/// Custom backend that captures rendered output with ANSI escape codes.
struct AnsiBackend {
    output: String,
    size: Size,
}

impl AnsiBackend {
    /// Creates a new `AnsiBackend` instance.
    pub fn new(width: u16, height: u16) -> Self {
        AnsiBackend {
            output: String::new(),
            size: Size { width, height },
        }
    }

    /// Retrieves the captured ANSI string.
    pub fn get_output(&self) -> &str {
        &self.output
    }
}

impl Backend for AnsiBackend {
    /// Draws the buffer contents into the ANSI string.
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        let cell_map: HashMap<(u16, u16), &Cell> =
            content.map(|(x, y, cell)| ((x, y), cell)).collect();

        self.output.clear();

        let mut current_style = Style::default();

        for y in 0..self.size.height {
            for x in 0..self.size.width {
                if let Some(cell) = cell_map.get(&(x, y)) {
                    if cell.style() != current_style {
                        let ansi_code = style_to_ansi(&cell.style());
                        self.output.push_str(&ansi_code);
                        current_style = cell.style();
                    }

                    let symbol = cell.symbol();
                    if symbol.is_empty() {
                        self.output.push(' ');
                    } else {
                        self.output.push_str(&symbol);
                    }
                } else {
                    self.output.push(' ');
                }
            }
            self.output.push_str("\x1b[0m\n");
            current_style = Style::default();
        }

        Ok(())
    }

    /// Hides the cursor. No-op for ANSI backend.
    fn hide_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }

    /// Shows the cursor. No-op for ANSI backend.
    fn show_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }

    /// Retrieves the current cursor position. Returns default position.
    fn get_cursor_position(&mut self) -> io::Result<ratatui::layout::Position> {
        Ok(ratatui::layout::Position { x: 0, y: 0 })
    }

    /// Sets the cursor position. No-op for ANSI backend.
    fn set_cursor_position<P: Into<ratatui::layout::Position>>(
        &mut self,
        _pos: P,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Clears the screen by resetting the output string.
    fn clear(&mut self) -> io::Result<()> {
        self.output.clear();
        Ok(())
    }

    /// Returns the size of the terminal.
    fn size(&self) -> io::Result<Size> {
        Ok(self.size)
    }

    /// Retrieves the window size, mapping `columns_rows` correctly.
    fn window_size(&mut self) -> io::Result<ratatui::backend::WindowSize> {
        Ok(ratatui::backend::WindowSize {
            columns_rows: self.size,
            pixels: Size {
                width: 0,
                height: 0,
            },
        })
    }

    /// Flushes the output. No-op for ANSI backend.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Converts a `Style` to its corresponding ANSI escape codes.
fn style_to_ansi(style: &Style) -> String {
    let mut ansi = String::new();
    let esc = '\x1b';

    if style.add_modifier.contains(Modifier::BOLD) {
        ansi.push_str(&format!("{}[1m", esc));
    }
    if style.add_modifier.contains(Modifier::UNDERLINED) {
        ansi.push_str(&format!("{}[4m", esc));
    }
    if style.add_modifier.contains(Modifier::DIM) {
        ansi.push_str(&format!("{}[2m", esc));
    }

    if let Some(fg) = style.fg {
        let fg_code = color_to_ansi(fg, false);
        if !fg_code.is_empty() {
            ansi.push_str(&format!("{}[{}m", esc, fg_code));
        }
    }

    if let Some(bg) = style.bg {
        let bg_code = color_to_ansi(bg, true);
        if !bg_code.is_empty() {
            ansi.push_str(&format!("{}[{}m", esc, bg_code));
        }
    }

    ansi
}

/// Maps a `Color` to its ANSI color code.
fn color_to_ansi(color: Color, is_bg: bool) -> String {
    let base = if is_bg { 40 } else { 30 };
    match color {
        Color::Black => format!("{}", base + 0),
        Color::Red => format!("{}", base + 1),
        Color::Green => format!("{}", base + 2),
        Color::Yellow => format!("{}", base + 3),
        Color::Blue => format!("{}", base + 4),
        Color::Magenta => format!("{}", base + 5),
        Color::Cyan => format!("{}", base + 6),
        Color::Gray => format!("{}", base + 7),
        Color::DarkGray => format!("{}", base + 60 + 0),
        Color::LightRed => format!("{}", base + 60 + 1),
        Color::LightGreen => format!("{}", base + 60 + 2),
        Color::LightYellow => format!("{}", base + 60 + 3),
        Color::LightBlue => format!("{}", base + 60 + 4),
        Color::LightMagenta => format!("{}", base + 60 + 5),
        Color::LightCyan => format!("{}", base + 60 + 6),
        Color::White => format!("{}", base + 60 + 7),
        _ => String::new(),
    }
}

/// Draws onto a string buffer with ANSI escape codes using a custom `AnsiBackend`.
///
/// # Arguments
///
/// * `width` - The width of the rendering area.
/// * `height` - The height of the rendering area.
/// * `draw_fn` - A callback function that handles drawing on the terminal frame.
///
/// # Returns
///
/// A `Result` containing the rendered string with ANSI codes or an error.
pub fn draw_to_string<F>(width: u16, height: u16, draw_fn: F) -> Result<String, Box<dyn Error>>
where
    F: FnOnce(&mut Frame),
{
    let backend = AnsiBackend::new(width, height);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(draw_fn)?;
    let backend = terminal.backend();
    let rendered = backend.get_output().to_string();
    Ok(rendered)
}
