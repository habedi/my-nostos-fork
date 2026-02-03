//! NostosPanel - A Cursive view that renders content from Nostos code
//!
//! This allows parts of the TUI to be written in Nostos itself.

use cursive::event::{Event, EventResult, Key};
use cursive::view::{View, CannotFocus};
use cursive::direction::Direction;
use cursive::{Printer, Vec2, Rect};
use nostos_repl::ReplEngine;
use std::cell::RefCell;
use std::rc::Rc;

/// A panel whose content and behavior is defined in Nostos code
pub struct NostosPanel {
    /// Reference to the REPL engine for evaluating Nostos code
    engine: Rc<RefCell<ReplEngine>>,
    /// Name of the Nostos function that returns the view
    view_fn: String,
    /// Name of the Nostos function that handles key events (receives key name as string)
    key_handler_fn: String,
    /// Cached rendered content
    cached_content: String,
    /// Whether we need to re-render
    needs_refresh: bool,
}

impl NostosPanel {
    /// Create a new NostosPanel
    ///
    /// # Arguments
    /// * `engine` - Reference to the ReplEngine
    /// * `view_fn` - Name of the Nostos function that returns view content
    /// * `key_handler_fn` - Name of the Nostos function that handles keys (receives key name)
    /// * `_title` - Panel title (unused, kept for API compatibility)
    pub fn new(engine: Rc<RefCell<ReplEngine>>, view_fn: &str, key_handler_fn: &str, _title: &str) -> Self {
        let mut panel = Self {
            engine,
            view_fn: view_fn.to_string(),
            key_handler_fn: key_handler_fn.to_string(),
            cached_content: String::new(),
            needs_refresh: true,
        };
        // Initial render
        panel.refresh();
        panel
    }

    /// Refresh the view by re-evaluating the Nostos view function
    pub fn refresh(&mut self) {
        let result = self.engine.borrow_mut().eval(&format!("{}()", self.view_fn));
        match result {
            Ok(content) => {
                // ReplEngine.eval returns a formatted string directly
                // Strip quotes if it's a string literal result
                self.cached_content = content.trim_matches('"').to_string();
            }
            Err(e) => {
                self.cached_content = format!("Error: {}", e);
            }
        }
        self.needs_refresh = false;
    }

    /// Convert a key event to our string representation
    fn event_to_key_string(event: &Event) -> Option<String> {
        match event {
            Event::Char(c) => Some(c.to_string()),
            Event::Key(Key::Up) => Some("up".to_string()),
            Event::Key(Key::Down) => Some("down".to_string()),
            Event::Key(Key::Left) => Some("left".to_string()),
            Event::Key(Key::Right) => Some("right".to_string()),
            Event::Key(Key::Enter) => Some("enter".to_string()),
            Event::Key(Key::Tab) => Some("tab".to_string()),
            Event::Key(Key::Backspace) => Some("backspace".to_string()),
            Event::Key(Key::Del) => Some("delete".to_string()),
            Event::Key(Key::Esc) => Some("esc".to_string()),
            Event::Key(Key::Home) => Some("home".to_string()),
            Event::Key(Key::End) => Some("end".to_string()),
            Event::Key(Key::PageUp) => Some("pageup".to_string()),
            Event::Key(Key::PageDown) => Some("pagedown".to_string()),
            Event::CtrlChar(c) => Some(format!("ctrl+{}", c)),
            Event::AltChar(c) => Some(format!("alt+{}", c)),
            _ => None,
        }
    }
}

impl View for NostosPanel {
    fn draw(&self, printer: &Printer) {
        // Draw content directly (no border - ActiveWindow handles that)
        for (i, line) in self.cached_content.lines().enumerate() {
            if i >= printer.size.y {
                break;
            }
            printer.print((0, i), line);
        }
    }

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        let lines = self.cached_content.lines().count().max(1);
        let max_width = self.cached_content.lines()
            .map(|l| l.len())
            .max()
            .unwrap_or(10);

        Vec2::new(
            max_width.min(constraint.x),
            lines.min(constraint.y)
        )
    }

    fn take_focus(&mut self, _source: Direction) -> Result<EventResult, CannotFocus> {
        Ok(EventResult::Consumed(None))
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        // Let these events bubble up to OnEventView wrapper for close/navigation
        match &event {
            Event::Shift(Key::Tab) => return EventResult::Ignored,  // Window cycling
            Event::Key(Key::Esc) => return EventResult::Ignored,     // Close panel
            Event::CtrlChar('w') => return EventResult::Ignored,     // Close panel
            _ => {}
        }

        // Convert event to key string and pass to Nostos handler
        if let Some(key_str) = Self::event_to_key_string(&event) {
            // Call the Nostos key handler with the key name
            let call = format!("{}(\"{}\")", self.key_handler_fn, key_str);
            let _ = self.engine.borrow_mut().eval(&call);
            self.refresh();
            return EventResult::Consumed(None);
        }

        EventResult::Ignored
    }

    fn important_area(&self, size: Vec2) -> Rect {
        Rect::from_size((0, 0), size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_to_key_string() {
        assert_eq!(NostosPanel::event_to_key_string(&Event::Char('a')), Some("a".to_string()));
        assert_eq!(NostosPanel::event_to_key_string(&Event::Key(Key::Up)), Some("up".to_string()));
        assert_eq!(NostosPanel::event_to_key_string(&Event::CtrlChar('k')), Some("ctrl+k".to_string()));
        assert_eq!(NostosPanel::event_to_key_string(&Event::AltChar('x')), Some("alt+x".to_string()));
    }
}
