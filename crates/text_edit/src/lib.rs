use egui::Event;
use egui::Key;
use egui::Modifiers;
use egui::Ui;
use std::ops::Range;

/// Plain data describing the editable text and where the cursor /
/// selection is. All fields are public — the caller manipulates them
/// directly when they need to (e.g. resyncing from an external source).
///
/// Byte positions are tracked (not chars) so mutation is O(1) and
/// UTF-8 is always valid. For painting, use the two char-space
/// helpers below.
#[derive(Debug, Clone, Default)]
pub struct State {
    /// Current text.
    text: String,
    /// Cursor position as a **byte index** into `text`. Always on a
    /// UTF-8 char boundary.
    cursor: usize,
    /// Selection anchor as a byte index. `Some(a)` with `a != cursor`
    /// means a selection is active; `None` or `a == cursor` means
    /// there is none.
    anchor: Option<usize>,
}

impl State {
    pub fn get_text(&self) -> &String {
        &self.text
    }

    /// Cursor position measured in **`char`s** — what you need for
    /// painting at `x0 + char_cursor * char_width`.
    pub fn cursor_char(&self) -> usize {
        self.text[..self.cursor].chars().count()
    }

    /// Selection measured in **`char`s** — `Some(range)` for
    /// `x0 + range.start * char_width` .. `x0 + range.end * char_width`.
    pub fn selection_char_range(&self) -> Option<Range<usize>> {
        let a = self.anchor?;
        if a == self.cursor {
            return None;
        }
        let (lo, hi) = if a < self.cursor {
            (a, self.cursor)
        } else {
            (self.cursor, a)
        };
        let lo_char = self.text[..lo].chars().count();
        let hi_char = self.text[..hi].chars().count();
        Some(lo_char..hi_char)
    }
}

impl From<String> for State {
    fn from(s: String) -> Self {
        let cursor = s.len();
        Self {
            text: s,
            cursor,
            anchor: None,
        }
    }
}

impl From<&str> for State {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}

pub fn update(ui: &mut Ui, state: &mut State) {
    ui.input_mut(|i| {
        for event in &i.events {
            process(state, event);
        }
    });
}

// ═══════════════════════════════════════════════════════════════════
// Everything below is private.
// ═══════════════════════════════════════════════════════════════════

/// Returns `true` if the event should stay in `events`.
fn process(state: &mut State, event: &Event) {
    match event {
        Event::Text(t) => {
            state.insert(t);
        }
        Event::Paste(t) => {
            state.insert(t);
        }
        Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } => handle_key(state, *key, *modifiers),
        _ => {}
    }
}

/// Returns `true` if the key event should stay in `events`.
fn handle_key(state: &mut State, key: Key, modifiers: Modifiers) {
    let shift = modifiers.shift;
    let cmd = modifiers.ctrl || modifiers.command;

    match key {
        Key::Backspace => {
            let changed = if cmd {
                state.delete_word_backward()
            } else {
                state.delete_backward()
            };
        }
        Key::Delete => {
            let changed = if cmd {
                state.delete_word_forward()
            } else {
                state.delete_forward()
            };
        }
        Key::ArrowLeft => {
            if shift {
                if cmd {
                    state.move_word_left_select();
                } else {
                    state.move_left_select();
                }
            } else if let Some(r) = state.selection_range() {
                state.cursor = r.start;
                state.anchor = None;
            } else if cmd {
                state.cursor = state.word_left(state.cursor);
                state.anchor = None;
            } else {
                state.cursor = state.prev_boundary(state.cursor);
                state.anchor = None;
            }
        }
        Key::ArrowRight => {
            if shift {
                if cmd {
                    state.move_word_right_select();
                } else {
                    state.move_right_select();
                }
            } else if let Some(r) = state.selection_range() {
                state.cursor = r.end;
                state.anchor = None;
            } else if cmd {
                state.cursor = state.word_right(state.cursor);
                state.anchor = None;
            } else {
                state.cursor = state.next_boundary(state.cursor);
                state.anchor = None;
            }
        }
        Key::Home => {
            if shift {
                state.ensure_anchor();
                state.cursor = 0;
            } else if let Some(r) = state.selection_range() {
                state.cursor = r.start;
                state.anchor = None;
            } else {
                state.cursor = 0;
                state.anchor = None;
            }
        }
        Key::End => {
            if shift {
                state.ensure_anchor();
                state.cursor = state.text.len();
            } else if let Some(r) = state.selection_range() {
                state.cursor = r.end;
                state.anchor = None;
            } else {
                state.cursor = state.text.len();
                state.anchor = None;
            }
        }
        Key::A if cmd => {
            state.anchor = Some(0);
            state.cursor = state.text.len();
        }
        Key::C if cmd => if let Some(t) = state.selection_text() {},
        Key::X if cmd => {
            if let Some(t) = state.selection_text().map(str::to_string) {
                state.delete_selection_inner();
            }
        }
        _ => {}
    }
}

// ─── Selection helpers ─────────────────────────────────────────────

impl State {
    fn selection_range(&self) -> Option<Range<usize>> {
        let a = self.anchor?;
        if a == self.cursor {
            return None;
        }
        Some(if a < self.cursor {
            a..self.cursor
        } else {
            self.cursor..a
        })
    }

    fn selection_text(&self) -> Option<&str> {
        let r = self.selection_range()?;
        Some(&self.text[r])
    }

    fn delete_selection_inner(&mut self) {
        let Some(r) = self.selection_range() else {
            return;
        };
        self.text.replace_range(r.clone(), "");
        self.cursor = r.start;
        self.anchor = None;
    }

    fn ensure_anchor(&mut self) {
        if self.anchor.is_none() {
            self.anchor = Some(self.cursor);
        }
    }

    fn insert(&mut self, s: &str) {
        self.delete_selection_inner();
        let at = self.cursor;
        self.text.insert_str(at, s);
        self.cursor = at + s.len();
        self.anchor = None;
    }

    fn delete_backward(&mut self) {
        if self.cursor == 0 {
            return;
        }

        let prev = self.prev_boundary(self.cursor);

        self.text.replace_range(prev..self.cursor, "");
        self.cursor = prev;
    }

    fn delete_forward(&mut self) {
        if self.cursor >= self.text.len() {
            return;
        }

        let next = self.next_boundary(self.cursor);
        self.text.replace_range(self.cursor..next, "");
    }

    fn delete_word_backward(&mut self) {
        if self.cursor == 0 {
            return;
        }

        let start = self.word_left(self.cursor);

        if start == self.cursor {
            return;
        }

        self.text.replace_range(start..self.cursor, "");
        self.cursor = start;
    }

    fn delete_word_forward(&mut self) {
        if self.cursor >= self.text.len() {
            return;
        }
        let end = self.word_right(self.cursor);

        if end == self.cursor {
            return;
        }

        self.text.replace_range(self.cursor..end, "");
    }

    fn move_left_select(&mut self) {
        self.ensure_anchor();
        self.cursor = self.prev_boundary(self.cursor);
    }

    fn move_right_select(&mut self) {
        self.ensure_anchor();
        self.cursor = self.next_boundary(self.cursor);
    }

    fn move_word_left_select(&mut self) {
        self.ensure_anchor();
        self.cursor = self.word_left(self.cursor);
    }

    fn move_word_right_select(&mut self) {
        self.ensure_anchor();
        self.cursor = self.word_right(self.cursor);
    }

    fn prev_boundary(&self, pos: usize) -> usize {
        let mut i = pos.min(self.text.len());
        while i > 0 {
            i -= 1;
            if self.text.is_char_boundary(i) {
                return i;
            }
        }
        0
    }

    fn next_boundary(&self, pos: usize) -> usize {
        let mut i = pos.saturating_add(1).min(self.text.len());
        while i < self.text.len() && !self.text.is_char_boundary(i) {
            i += 1;
        }
        i
    }

    /// Move left to the start of the previous "word group":
    /// skip non-word chars, then word chars.
    fn word_left(&self, from: usize) -> usize {
        let mut i = from;
        while i > 0 {
            let prev = self.prev_boundary(i);
            let c = self.text[prev..i].chars().next().unwrap_or(' ');
            if is_word_char(c) {
                break;
            }
            i = prev;
        }
        while i > 0 {
            let prev = self.prev_boundary(i);
            let c = self.text[prev..i].chars().next().unwrap_or(' ');
            if !is_word_char(c) {
                break;
            }
            i = prev;
        }
        i
    }

    /// Move right to the end of the next "word group":
    /// skip word chars, then non-word chars.
    fn word_right(&self, from: usize) -> usize {
        let mut i = from;
        while i < self.text.len() {
            let next = self.next_boundary(i);
            let c = self.text[i..next].chars().next().unwrap_or(' ');
            if !is_word_char(c) {
                break;
            }
            i = next;
        }
        while i < self.text.len() {
            let next = self.next_boundary(i);
            let c = self.text[i..next].chars().next().unwrap_or(' ');
            if is_word_char(c) {
                break;
            }
            i = next;
        }
        i
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
