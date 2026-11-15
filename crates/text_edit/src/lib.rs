use egui::Event;
use egui::Key;
use egui::Modifiers;
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
    pub text: String,
    /// Cursor position as a **byte index** into `text`. Always on a
    /// UTF-8 char boundary.
    pub cursor: usize,
    /// Selection anchor as a byte index. `Some(a)` with `a != cursor`
    /// means a selection is active; `None` or `a == cursor` means
    /// there is none.
    pub anchor: Option<usize>,
}

impl State {
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

/// What happened this frame. All fields are read-only for the caller.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Outcome {
    /// `true` if `state.text` differs from before the call — the
    /// caller should sync their `&mut String` from `state.text`.
    pub changed: bool,
    /// Enter was pressed.
    pub submit: bool,
    /// Escape was pressed.
    pub cancel: bool,
    /// Non-`None` when Ctrl+C or Ctrl+X was pressed with a selection.
    /// Contains the text to write to the system clipboard.
    pub clipboard: Option<String>,
}

/// Process `events` against `state`, mutating both:
///
/// * `state.text` / `state.cursor` / `state.anchor` are updated.
/// * Editing events (text input, key presses we understand) are
///   **removed** from `events` so other widgets won't re-handle them.
///   Non-editing events (pointer, non-editing keys, Enter for the
///   caller's form, …) are **left in place**.
///
/// This is the **only** function this crate exposes. Everything else
/// the caller needs is plain data on [`State`] / [`Outcome`].
///
/// ## Behaviour (VS Code)
///
/// * `←/→` move by char; `Ctrl+←/→` move by word.
/// * `Shift+←/→` extend selection; `Ctrl+Shift+←/→` extend by word.
/// * `←/→` with an active selection collapse to that edge first.
/// * `Home` / `End` jump to line bounds.
/// * `Backspace` / `Delete` remove one char; `Ctrl+Backspace` /
///   `Ctrl+Delete` remove a word. Both are selection-aware.
/// * `Ctrl+A` selects all; `Ctrl+C` / `Ctrl+X` set
///   [`Outcome::clipboard`].
/// * `Enter` sets [`Outcome::submit`] and stays in `events`.
/// * `Escape` sets [`Outcome::cancel`] and is consumed.
///
/// ## Example
///
/// ```ignore
/// // `state` lives across frames, e.g. in `ui.data`.
/// let mut state = text_edit::State::from(value.as_str());
///
/// let outcome = ui.input_mut(|i| text_edit::update(&mut state, &mut i.events));
///
/// if outcome.changed {
///     *value = state.text.clone();
/// }
/// if let Some(text) = outcome.clipboard {
///     ui.ctx().copy_text(text);
/// }
/// if outcome.cancel {
///     ui.memory_mut(|m| m.surrender_focus(id));
/// }
/// // outcome.submit → caller decides what to do
/// ```
pub fn update(state: &mut State, events: &mut Vec<Event>) -> Outcome {
    let mut outcome = Outcome::default();

    let mut i = 0;
    while i < events.len() {
        let keep = process(state, &events[i], &mut outcome);
        if keep {
            i += 1;
        } else {
            events.remove(i);
        }
    }

    outcome
}

// ═══════════════════════════════════════════════════════════════════
// Everything below is private.
// ═══════════════════════════════════════════════════════════════════

/// Returns `true` if the event should stay in `events`.
fn process(state: &mut State, event: &Event, outcome: &mut Outcome) -> bool {
    match event {
        Event::Text(t) => {
            insert(state, t);
            outcome.changed = true;
            false
        }
        Event::Paste(t) => {
            insert(state, t);
            outcome.changed = true;
            false
        }
        Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } => handle_key(state, *key, *modifiers, outcome),
        _ => true,
    }
}

/// Returns `true` if the key event should stay in `events`.
fn handle_key(state: &mut State, key: Key, modifiers: Modifiers, outcome: &mut Outcome) -> bool {
    let shift = modifiers.shift;
    let cmd = modifiers.ctrl || modifiers.command;

    match key {
        Key::Backspace => {
            let changed = if cmd {
                delete_word_backward(state)
            } else {
                delete_backward(state)
            };
            if changed {
                outcome.changed = true;
            }
            false
        }
        Key::Delete => {
            let changed = if cmd {
                delete_word_forward(state)
            } else {
                delete_forward(state)
            };
            if changed {
                outcome.changed = true;
            }
            false
        }
        Key::ArrowLeft => {
            if shift {
                if cmd {
                    move_word_left_select(state);
                } else {
                    move_left_select(state);
                }
            } else if let Some(r) = selection_range(state) {
                state.cursor = r.start;
                state.anchor = None;
            } else if cmd {
                state.cursor = word_left(state, state.cursor);
                state.anchor = None;
            } else {
                state.cursor = prev_boundary(state, state.cursor);
                state.anchor = None;
            }
            false
        }
        Key::ArrowRight => {
            if shift {
                if cmd {
                    move_word_right_select(state);
                } else {
                    move_right_select(state);
                }
            } else if let Some(r) = selection_range(state) {
                state.cursor = r.end;
                state.anchor = None;
            } else if cmd {
                state.cursor = word_right(state, state.cursor);
                state.anchor = None;
            } else {
                state.cursor = next_boundary(state, state.cursor);
                state.anchor = None;
            }
            false
        }
        Key::Home => {
            if shift {
                ensure_anchor(state);
                state.cursor = 0;
            } else if let Some(r) = selection_range(state) {
                state.cursor = r.start;
                state.anchor = None;
            } else {
                state.cursor = 0;
                state.anchor = None;
            }
            false
        }
        Key::End => {
            if shift {
                ensure_anchor(state);
                state.cursor = state.text.len();
            } else if let Some(r) = selection_range(state) {
                state.cursor = r.end;
                state.anchor = None;
            } else {
                state.cursor = state.text.len();
                state.anchor = None;
            }
            false
        }
        Key::A if cmd => {
            state.anchor = Some(0);
            state.cursor = state.text.len();
            false
        }
        Key::C if cmd => {
            if let Some(t) = selection_text(state) {
                outcome.clipboard = Some(t.to_string());
            }
            false
        }
        Key::X if cmd => {
            if let Some(t) = selection_text(state).map(str::to_string) {
                delete_selection_inner(state);
                outcome.clipboard = Some(t);
                outcome.changed = true;
            }
            false
        }
        Key::Enter => {
            outcome.submit = true;
            true // keep in the stream — caller may want it too
        }
        Key::Escape => {
            outcome.cancel = true;
            false
        }
        _ => true,
    }
}

// ─── Selection helpers ─────────────────────────────────────────────

fn selection_range(state: &State) -> Option<Range<usize>> {
    let a = state.anchor?;
    if a == state.cursor {
        return None;
    }
    Some(if a < state.cursor {
        a..state.cursor
    } else {
        state.cursor..a
    })
}

fn selection_text(state: &State) -> Option<&str> {
    let r = selection_range(state)?;
    Some(&state.text[r])
}

fn delete_selection_inner(state: &mut State) -> bool {
    let Some(r) = selection_range(state) else {
        return false;
    };
    state.text.replace_range(r.clone(), "");
    state.cursor = r.start;
    state.anchor = None;
    true
}

fn ensure_anchor(state: &mut State) {
    if state.anchor.is_none() {
        state.anchor = Some(state.cursor);
    }
}

// ─── Mutation ──────────────────────────────────────────────────────

fn insert(state: &mut State, s: &str) {
    delete_selection_inner(state);
    let at = state.cursor;
    state.text.insert_str(at, s);
    state.cursor = at + s.len();
    state.anchor = None;
}

fn delete_backward(state: &mut State) -> bool {
    if delete_selection_inner(state) {
        return true;
    }
    if state.cursor == 0 {
        return false;
    }
    let prev = prev_boundary(state, state.cursor);
    state.text.replace_range(prev..state.cursor, "");
    state.cursor = prev;
    true
}

fn delete_forward(state: &mut State) -> bool {
    if delete_selection_inner(state) {
        return true;
    }
    if state.cursor >= state.text.len() {
        return false;
    }
    let next = next_boundary(state, state.cursor);
    state.text.replace_range(state.cursor..next, "");
    true
}

fn delete_word_backward(state: &mut State) -> bool {
    if delete_selection_inner(state) {
        return true;
    }
    if state.cursor == 0 {
        return false;
    }
    let start = word_left(state, state.cursor);
    if start == state.cursor {
        return false;
    }
    state.text.replace_range(start..state.cursor, "");
    state.cursor = start;
    true
}

fn delete_word_forward(state: &mut State) -> bool {
    if delete_selection_inner(state) {
        return true;
    }
    if state.cursor >= state.text.len() {
        return false;
    }
    let end = word_right(state, state.cursor);
    if end == state.cursor {
        return false;
    }
    state.text.replace_range(state.cursor..end, "");
    true
}

// ─── Cursor movement ──────────────────────────────────────────────

fn move_left_select(state: &mut State) {
    ensure_anchor(state);
    state.cursor = prev_boundary(state, state.cursor);
}

fn move_right_select(state: &mut State) {
    ensure_anchor(state);
    state.cursor = next_boundary(state, state.cursor);
}

fn move_word_left_select(state: &mut State) {
    ensure_anchor(state);
    state.cursor = word_left(state, state.cursor);
}

fn move_word_right_select(state: &mut State) {
    ensure_anchor(state);
    state.cursor = word_right(state, state.cursor);
}

// ─── Boundary navigation ──────────────────────────────────────────

fn prev_boundary(state: &State, pos: usize) -> usize {
    let mut i = pos.min(state.text.len());
    while i > 0 {
        i -= 1;
        if state.text.is_char_boundary(i) {
            return i;
        }
    }
    0
}

fn next_boundary(state: &State, pos: usize) -> usize {
    let mut i = pos.saturating_add(1).min(state.text.len());
    while i < state.text.len() && !state.text.is_char_boundary(i) {
        i += 1;
    }
    i
}

/// Move left to the start of the previous "word group":
/// skip non-word chars, then word chars.
fn word_left(state: &State, from: usize) -> usize {
    let mut i = from;
    while i > 0 {
        let prev = prev_boundary(state, i);
        let c = state.text[prev..i].chars().next().unwrap_or(' ');
        if is_word_char(c) {
            break;
        }
        i = prev;
    }
    while i > 0 {
        let prev = prev_boundary(state, i);
        let c = state.text[prev..i].chars().next().unwrap_or(' ');
        if !is_word_char(c) {
            break;
        }
        i = prev;
    }
    i
}

/// Move right to the end of the next "word group":
/// skip word chars, then non-word chars.
fn word_right(state: &State, from: usize) -> usize {
    let mut i = from;
    while i < state.text.len() {
        let next = next_boundary(state, i);
        let c = state.text[i..next].chars().next().unwrap_or(' ');
        if !is_word_char(c) {
            break;
        }
        i = next;
    }
    while i < state.text.len() {
        let next = next_boundary(state, i);
        let c = state.text[i..next].chars().next().unwrap_or(' ');
        if is_word_char(c) {
            break;
        }
        i = next;
    }
    i
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
