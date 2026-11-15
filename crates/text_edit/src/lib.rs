use egui::Event;
use egui::Key;
use egui::Modifiers;
use egui::Ui;
use std::ops::Range;

/// Plain data describing the editable text and where the cursor /
/// selection is. All mutation goes through [`update`] or the small
/// set of public methods below — the raw fields are kept private so
/// nothing outside this crate can put the editor in an inconsistent
/// state (e.g. cursor mid-UTF-8-codepoint).
///
/// Byte positions are tracked (not chars) so mutation is O(1) and
/// UTF-8 is always valid. For painting, use the two char-space
/// helpers [`cursor_char`][State::cursor_char] /
/// [`selection_char_range`][State::selection_char_range].
#[derive(Debug, Clone, Default)]
pub struct State {
    text: String,
    cursor: usize,
    anchor: Option<usize>,
}

impl State {
    // ── Read ──────────────────────────────────────────────────────

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

    // ── Click-driven mutations (called from the display layer) ──

    /// Move the cursor to `char_idx` and clear any selection.
    pub fn set_cursor_char(&mut self, char_idx: usize) {
        self.cursor = self.char_to_byte(char_idx);
        self.anchor = None;
    }

    /// Select the word at `char_idx` (inclusive). If `char_idx`
    /// points at whitespace immediately after a word, selects the
    /// word before it — matching VS Code's double-click.
    pub fn select_word_at(&mut self, char_idx: usize) {
        let total = self.text.chars().count();
        let byte = self.char_to_byte(char_idx.min(total));

        if let Some((s, e)) = self.word_extent_at(byte) {
            self.anchor = Some(s);
            self.cursor = e;
            return;
        }
        if byte > 0 {
            let prev = self.prev_boundary(byte);
            if let Some((s, e)) = self.word_extent_at(prev) {
                self.anchor = Some(s);
                self.cursor = e;
                return;
            }
        }
        self.cursor = byte;
        self.anchor = None;
    }

    /// Select the whole buffer.
    pub fn select_all(&mut self) {
        self.anchor = Some(0);
        self.cursor = self.text.len();
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

/// Feed keyboard events from `ui` into `state`.
///
/// * Consumes the events this crate handled (text input, arrows,
///   backspace/delete, `Ctrl+A/C/X`, `Home`/`End`) so a sibling
///   widget can't double-process them.
/// * Leaves `Enter`, `Escape`, and any unrecognised key in
///   `ui.input().events` — the caller can check them with
///   `ui.input(|i| i.key_pressed(Key::Enter))` etc.
/// * `Ctrl+C` / `Ctrl+X` write the selection to the system
///   clipboard via `ui.ctx().copy_text(...)`.
pub fn update(ui: &mut Ui, state: &mut State) {
    let mut clipboard: Option<String> = None;
    ui.input_mut(|i| {
        i.events
            .retain(|event| !process(state, event, &mut clipboard));
    });
    if let Some(text) = clipboard {
        ui.ctx().copy_text(text);
    }
}

// ═══════════════════════════════════════════════════════════════════
// Everything below is private.
// ═══════════════════════════════════════════════════════════════════

/// Returns `true` if the event was consumed (should be removed).
fn process(state: &mut State, event: &Event, clipboard: &mut Option<String>) -> bool {
    match event {
        Event::Text(t) => {
            state.insert(t);
            true
        }
        Event::Paste(t) => {
            state.insert(t);
            true
        }
        Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } => handle_key(state, *key, *modifiers, clipboard),
        _ => false,
    }
}

/// Returns `true` if the key was consumed.
fn handle_key(
    state: &mut State,
    key: Key,
    modifiers: Modifiers,
    clipboard: &mut Option<String>,
) -> bool {
    let shift = modifiers.shift;
    let cmd = modifiers.ctrl || modifiers.command;

    match key {
        Key::Backspace => {
            if cmd {
                state.delete_word_backward();
            } else {
                state.delete_backward();
            }
            true
        }
        Key::Delete => {
            if cmd {
                state.delete_word_forward();
            } else {
                state.delete_forward();
            }
            true
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
            true
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
            true
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
            true
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
            true
        }
        Key::A if cmd => {
            state.anchor = Some(0);
            state.cursor = state.text.len();
            true
        }
        Key::C if cmd => {
            if let Some(t) = state.selection_text() {
                *clipboard = Some(t.to_string());
            }
            true
        }
        Key::X if cmd => {
            if let Some(t) = state.selection_text().map(str::to_string) {
                state.delete_selection_inner();
                *clipboard = Some(t);
            }
            true
        }
        // Enter / Escape / unknown keys stay in the stream — the
        // caller decides what to do with them.
        _ => false,
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

    // ── Mutation ──────────────────────────────────────────────────

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

    // ── Boundaries ────────────────────────────────────────────────

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

    fn char_to_byte(&self, char_idx: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_idx)
            .map(|(b, _)| b)
            .unwrap_or(self.text.len())
    }

    // ── Word navigation ──────────────────────────────────────────

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

    fn word_extent_at(&self, byte: usize) -> Option<(usize, usize)> {
        if byte >= self.text.len() {
            return None;
        }
        let c = self.text[byte..].chars().next()?;
        if !is_word_char(c) {
            return None;
        }
        let mut start = byte;
        while start > 0 {
            let p = self.prev_boundary(start);
            let pc = self.text[p..start].chars().next()?;
            if !is_word_char(pc) {
                break;
            }
            start = p;
        }
        let mut end = byte;
        while end < self.text.len() {
            let n = self.next_boundary(end);
            let nc = self.text[end..n].chars().next()?;
            if !is_word_char(nc) {
                break;
            }
            end = n;
        }
        Some((start, end))
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
