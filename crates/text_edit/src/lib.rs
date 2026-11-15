use egui::Event;
use egui::Key;
use egui::Modifiers;
use egui::Ui;
use std::ops::Range;

/// Snapshot of editable state used for undo / redo.
#[derive(Debug, Clone)]
struct Snapshot {
    text: String,
    cursor: usize,
    anchor: Option<usize>,
}

/// Plain data describing the editable text and where the cursor /
/// selection is.
///
/// **`cursor` and `anchor` are character indices**, not byte indices.
/// For `"hashim"`, `cursor == 5` means between `"hashi"` and `"m"`.
///
/// Mutation goes through [`update`] or the small set of public
/// methods on `State`.
#[derive(Debug, Clone, Default)]
pub struct State {
    text: String,
    cursor: usize,
    anchor: Option<usize>,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
}

impl State {
    // ── Read ──────────────────────────────────────────────────────

    pub fn get_text(&self) -> &String {
        &self.text
    }

    pub fn cursor_char(&self) -> usize {
        self.cursor
    }

    pub fn selection_char_range(&self) -> Option<Range<usize>> {
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

    // ── Click / drag mutations ────────────────────────────────────

    pub fn set_cursor_char(&mut self, char_idx: usize) {
        self.cursor = char_idx.min(self.char_count());
        self.anchor = None;
    }

    pub fn begin_selection_at(&mut self, char_idx: usize) {
        let idx = char_idx.min(self.char_count());
        self.cursor = idx;
        self.anchor = Some(idx);
    }

    pub fn extend_selection_to(&mut self, char_idx: usize) {
        self.cursor = char_idx.min(self.char_count());
        // anchor intentionally untouched
    }

    pub fn select_word_at(&mut self, char_idx: usize) {
        let total = self.char_count();
        let idx = char_idx.min(total);

        if let Some((s, e)) = word_extent_at(&self.text, idx) {
            self.anchor = Some(s);
            self.cursor = e;
            return;
        }
        if idx > 0 {
            if let Some((s, e)) = word_extent_at(&self.text, idx - 1) {
                self.anchor = Some(s);
                self.cursor = e;
                return;
            }
        }
        self.cursor = idx;
        self.anchor = None;
    }

    pub fn select_all(&mut self) {
        self.anchor = Some(0);
        self.cursor = self.char_count();
    }

    // ── Undo / redo ───────────────────────────────────────────────

    fn push_undo(&mut self) {
        self.undo.push(Snapshot {
            text: self.text.clone(),
            cursor: self.cursor,
            anchor: self.anchor,
        });
        self.redo.clear();
    }

    fn undo(&mut self) -> bool {
        let Some(prev) = self.undo.pop() else {
            return false;
        };
        self.redo.push(Snapshot {
            text: std::mem::take(&mut self.text),
            cursor: self.cursor,
            anchor: self.anchor,
        });
        self.text = prev.text;
        self.cursor = prev.cursor;
        self.anchor = prev.anchor;
        true
    }

    fn redo(&mut self) -> bool {
        let Some(next) = self.redo.pop() else {
            return false;
        };
        self.undo.push(Snapshot {
            text: std::mem::take(&mut self.text),
            cursor: self.cursor,
            anchor: self.anchor,
        });
        self.text = next.text;
        self.cursor = next.cursor;
        self.anchor = next.anchor;
        true
    }

    // ── Internal helpers ─────────────────────────────────────────

    fn char_count(&self) -> usize {
        self.text.chars().count()
    }

    fn selection_range(&self) -> Option<Range<usize>> {
        self.selection_char_range()
    }

    fn selection_text(&self) -> Option<&str> {
        let r = self.selection_range()?;
        let s = byte_of_char(&self.text, r.start);
        let e = byte_of_char(&self.text, r.end);
        Some(&self.text[s..e])
    }

    fn delete_selection_inner(&mut self) -> bool {
        let Some(r) = self.selection_range() else {
            return false;
        };
        let s = byte_of_char(&self.text, r.start);
        let e = byte_of_char(&self.text, r.end);
        self.text.replace_range(s..e, "");
        self.cursor = r.start;
        self.anchor = None;
        true
    }

    fn ensure_anchor(&mut self) {
        if self.anchor.is_none() {
            self.anchor = Some(self.cursor);
        }
    }

    // ── Mutation ─────────────────────────────────────────────────

    fn insert(&mut self, s: &str) {
        self.push_undo();
        self.delete_selection_inner();
        let byte = byte_of_char(&self.text, self.cursor);
        self.text.insert_str(byte, s);
        self.cursor += s.chars().count();
        self.anchor = None;
    }

    fn delete_backward(&mut self) {
        let had_selection = self.selection_range().is_some();
        if !had_selection && self.cursor == 0 {
            return;
        }
        self.push_undo();
        if self.delete_selection_inner() {
            return;
        }
        let s = byte_of_char(&self.text, self.cursor - 1);
        let e = byte_of_char(&self.text, self.cursor);
        self.text.replace_range(s..e, "");
        self.cursor -= 1;
    }

    fn delete_forward(&mut self) {
        let had_selection = self.selection_range().is_some();
        let total = self.char_count();
        if !had_selection && self.cursor >= total {
            return;
        }
        self.push_undo();
        if self.delete_selection_inner() {
            return;
        }
        let s = byte_of_char(&self.text, self.cursor);
        let e = byte_of_char(&self.text, self.cursor + 1);
        self.text.replace_range(s..e, "");
    }

    fn delete_word_backward(&mut self) {
        let had_selection = self.selection_range().is_some();
        if !had_selection {
            if self.cursor == 0 {
                return;
            }
            let start = word_left(&self.text, self.cursor);
            if start == self.cursor {
                return;
            }
        }
        self.push_undo();
        if self.delete_selection_inner() {
            return;
        }
        let start = word_left(&self.text, self.cursor);
        let s = byte_of_char(&self.text, start);
        let e = byte_of_char(&self.text, self.cursor);
        self.text.replace_range(s..e, "");
        self.cursor = start;
    }

    fn delete_word_forward(&mut self) {
        let had_selection = self.selection_range().is_some();
        let total = self.char_count();
        if !had_selection {
            if self.cursor >= total {
                return;
            }
            let end = word_right(&self.text, self.cursor);
            if end == self.cursor {
                return;
            }
        }
        self.push_undo();
        if self.delete_selection_inner() {
            return;
        }
        let end = word_right(&self.text, self.cursor);
        let s = byte_of_char(&self.text, self.cursor);
        let e = byte_of_char(&self.text, end);
        self.text.replace_range(s..e, "");
    }

    fn move_left_select(&mut self) {
        self.ensure_anchor();
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    fn move_right_select(&mut self) {
        self.ensure_anchor();
        let total = self.char_count();
        if self.cursor < total {
            self.cursor += 1;
        }
    }

    fn move_word_left_select(&mut self) {
        self.ensure_anchor();
        self.cursor = word_left(&self.text, self.cursor);
    }

    fn move_word_right_select(&mut self) {
        self.ensure_anchor();
        self.cursor = word_right(&self.text, self.cursor);
    }
}

impl From<String> for State {
    fn from(s: String) -> Self {
        let cursor = s.chars().count();
        Self {
            text: s,
            cursor,
            anchor: None,
            undo: Vec::new(),
            redo: Vec::new(),
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
/// Handles: typing, paste, **copy**, **cut**, arrows, home/end,
/// backspace/delete (with word variants), Ctrl+A, **Ctrl+Z**,
/// **Ctrl+Y / Ctrl+Shift+Z**. Copy / cut results are written to the
/// system clipboard via `ui.ctx().copy_text(...)`.
///
/// `Enter`, `Escape`, and unknown keys stay in `ui.input().events`.
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
// Event processing
// ═══════════════════════════════════════════════════════════════════

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
        // egui routes OS Ctrl+C / Ctrl+X as these events — NOT as
        // `Event::Key { key: Key::C, modifiers: ctrl }`. Without these
        // arms, copy / cut silently do nothing.
        Event::Copy => {
            if let Some(t) = state.selection_text() {
                *clipboard = Some(t.to_string());
            }
            true
        }
        Event::Cut => {
            if let Some(t) = state.selection_text().map(str::to_string) {
                state.push_undo();
                state.delete_selection_inner();
                *clipboard = Some(t);
            }
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
                state.cursor = word_left(&state.text, state.cursor);
                state.anchor = None;
            } else if state.cursor > 0 {
                state.cursor -= 1;
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
                state.cursor = word_right(&state.text, state.cursor);
                state.anchor = None;
            } else {
                let total = state.char_count();
                if state.cursor < total {
                    state.cursor += 1;
                    state.anchor = None;
                }
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
                state.cursor = state.char_count();
            } else if let Some(r) = state.selection_range() {
                state.cursor = r.end;
                state.anchor = None;
            } else {
                state.cursor = state.char_count();
                state.anchor = None;
            }
            true
        }
        Key::A if cmd => {
            state.select_all();
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
                state.push_undo();
                state.delete_selection_inner();
                *clipboard = Some(t);
            }
            true
        }
        Key::Z if cmd => {
            if shift {
                state.redo();
            } else {
                state.undo();
            }
            true
        }
        Key::Y if cmd => {
            state.redo();
            true
        }
        // Enter / Escape / unknown keys stay in the stream.
        _ => false,
    }
}

// ═══════════════════════════════════════════════════════════════════
// UTF-8 + word helpers (unchanged)
// ═══════════════════════════════════════════════════════════════════

fn byte_of_char(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(text.len())
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn word_left(text: &str, from: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let total = chars.len();
    let mut i = from.min(total);
    while i > 0 && !is_word_char(chars[i - 1]) {
        i -= 1;
    }
    while i > 0 && is_word_char(chars[i - 1]) {
        i -= 1;
    }
    i
}

fn word_right(text: &str, from: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let total = chars.len();
    let mut i = from.min(total);
    while i < total && !is_word_char(chars[i]) {
        i += 1;
    }
    while i < total && is_word_char(chars[i]) {
        i += 1;
    }
    i
}

fn word_extent_at(text: &str, char_idx: usize) -> Option<(usize, usize)> {
    let chars: Vec<char> = text.chars().collect();
    if char_idx >= chars.len() {
        return None;
    }
    if !is_word_char(chars[char_idx]) {
        return None;
    }
    let mut start = char_idx;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }
    let mut end = char_idx + 1;
    while end < chars.len() && is_word_char(chars[end]) {
        end += 1;
    }
    Some((start, end))
}
