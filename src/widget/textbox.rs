use arboard::Clipboard;
use cosmic_text::{Attrs, Metrics};
use graphics::{cosmic_text::rustybuzz::ttf_parser::name::Name, *};

const KEY_CTRL: usize = 0;
const MAX_KEY: usize = 1;

use input::Key;
use winit::keyboard::NamedKey;

use crate::{logic::*, widget::*, SystemHolder};

pub struct Textbox {
    visible: bool,
    pub text: String,
    char_size: Vec<f32>,
    text_index: usize,
    bg: usize,
    limit: usize,
    pub size: Vec2,
    pub pos: Vec3,
    adjust_x: f32,
    is_selected: bool,
    caret: usize,
    caret_left: f32,
    caret_pos: usize,

    special_key_hold: [bool; MAX_KEY],
    hide_content: bool,
    z_step: (f32, i32),
    pub tooltip: Option<String>,
}

impl Textbox {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        systems: &mut SystemHolder,
        pos: Vec3,
        z_step: (f32, i32),
        size: Vec2,
        text_color: Color,
        render_layer: usize,
        limit: usize,
        selected_bg_color: Color,
        hide_content: bool,
        visible: bool,
        tooltip: Option<String>,
    ) -> Self {
        let mut rect = Rect::new(&mut systems.renderer, 0);
        rect.set_color(selected_bg_color)
            .set_position(pos)
            .set_size(size);
        let bg = systems.gfx.add_rect(rect, 0);
        systems.gfx.set_visible(bg, false);

        let text_data = create_label(
            systems,
            Vec3::new(pos.x, pos.y, pos.z.sub_f32(z_step.0, z_step.1)),
            size,
            Bounds::new(pos.x, pos.y, pos.x + size.x, pos.y + size.y),
            text_color,
        );
        let text_index = systems.gfx.add_text(text_data, render_layer);
        systems.gfx.set_visible(text_index, visible);

        let mut caret_rect = Rect::new(&mut systems.renderer, 0);
        caret_rect
            .set_size(Vec2::new(2.0, size.y))
            .set_position(Vec3::new(
                pos.x,
                pos.y,
                pos.z.sub_f32(z_step.0, z_step.1),
            ))
            .set_color(text_color);
        let caret = systems.gfx.add_rect(caret_rect, 0);
        systems.gfx.set_visible(caret, false);

        Textbox {
            visible,
            text: String::new(),
            char_size: Vec::new(),
            text_index,
            bg,
            limit,
            size,
            pos,
            z_step,
            adjust_x: 0.0,
            is_selected: false,
            special_key_hold: [false; MAX_KEY],
            hide_content,
            tooltip,
            caret,
            caret_left: 0.0,
            caret_pos: 0,
        }
    }

    pub fn set_select(&mut self, systems: &mut SystemHolder, is_select: bool) {
        if self.is_selected == is_select || !self.visible {
            return;
        }
        self.is_selected = is_select;
        systems.gfx.set_visible(self.bg, self.is_selected);
        if self.is_selected {
            systems.caret.index = Some(self.caret);
        } else {
            systems.gfx.set_visible(self.caret, false);
            if let Some(index) = systems.caret.index {
                if index == self.caret {
                    systems.caret.index = None;
                }
            }
        }
    }

    pub fn unload(&mut self, systems: &mut SystemHolder) {
        systems.gfx.remove_gfx(&mut systems.renderer, self.bg);
        systems
            .gfx
            .remove_gfx(&mut systems.renderer, self.text_index);
        systems.gfx.remove_gfx(&mut systems.renderer, self.caret);
    }

    pub fn set_visible(&mut self, systems: &mut SystemHolder, visible: bool) {
        if self.visible == visible {
            return;
        }
        self.visible = visible;
        systems
            .gfx
            .set_visible(self.bg, self.is_selected && visible);
        systems.gfx.set_visible(self.text_index, visible);
        systems.gfx.set_visible(self.caret, false);
    }

    pub fn set_z_order(&mut self, systems: &mut SystemHolder, z_order: f32) {
        self.pos.z = z_order;
        systems.gfx.set_pos(self.bg, self.pos);
        systems.gfx.set_pos(
            self.text_index,
            Vec3::new(
                self.pos.x,
                self.pos.y,
                self.pos.z.sub_f32(self.z_step.0, self.z_step.1),
            ),
        );
        systems.gfx.set_pos(
            self.caret,
            Vec3::new(
                self.pos.x,
                self.pos.y,
                self.pos.z.sub_f32(self.z_step.0, self.z_step.1),
            ),
        );
    }

    pub fn set_pos(&mut self, systems: &mut SystemHolder, new_pos: Vec2) {
        self.pos.x = new_pos.x;
        self.pos.y = new_pos.y;
        systems.gfx.set_pos(self.bg, self.pos);
        systems.gfx.set_pos(
            self.text_index,
            Vec3::new(
                self.pos.x,
                self.pos.y,
                self.pos.z.sub_f32(self.z_step.0, self.z_step.1),
            ),
        );
        systems.gfx.set_bound(
            self.text_index,
            Bounds::new(
                self.pos.x,
                self.pos.y,
                self.pos.x + self.size.x,
                self.pos.y + self.size.y,
            ),
        );
        systems.gfx.set_pos(
            self.caret,
            Vec3::new(
                self.pos.x,
                self.pos.y,
                self.pos.z.sub_f32(self.z_step.0, self.z_step.1),
            ),
        );
    }

    pub fn set_text(&mut self, systems: &mut SystemHolder, text: String) {
        self.text.clear();
        if !text.is_empty() {
            self.text.push_str(&text);
        }
        let msg = if self.hide_content {
            self.text.chars().map(|_| '*').collect()
        } else {
            self.text.clone()
        };
        systems
            .gfx
            .set_text(&mut systems.renderer, self.text_index, &msg);
        //self.adjust_text(systems);
    }

    pub fn enter_text(
        &mut self,
        systems: &mut SystemHolder,
        key: &Key,
        pressed: bool,
        numeric_only: bool,
    ) {
        if !self.visible {
            return;
        }

        if let Key::Named(NamedKey::Control) = key {
            self.special_key_hold[KEY_CTRL] = pressed;
        }

        if !pressed || !self.is_selected {
            return;
        }

        let mut did_edit = false;
        if self.special_key_hold[KEY_CTRL] {
            if !numeric_only {
                match key {
                    Key::Character('c') => {
                        set_clipboard_text(self.text.clone());
                    }
                    Key::Character('v') => {
                        self.text.push_str(&get_clipboard_text());
                        did_edit = true;
                    }
                    _ => {}
                }
            }
        } else {
            match key {
                Key::Named(NamedKey::Backspace) => {
                    if self.caret_pos == self.text.len() {
                        self.move_caret_pos(systems, true, 1, true);
                        self.text.pop();
                        self.char_size.pop();
                    } else if self.caret_pos > 0 {
                        self.move_caret_pos(systems, true, 1, true);
                        self.text.remove(self.caret_pos);
                        self.char_size.remove(self.caret_pos);
                    }
                    did_edit = true;
                }
                Key::Named(NamedKey::Delete) => {
                    self.text.clear();
                    did_edit = true;
                }
                Key::Named(NamedKey::ArrowLeft) => {
                    self.move_caret_pos(systems, true, 1, false);
                    return;
                }
                Key::Named(NamedKey::ArrowRight) => {
                    self.move_caret_pos(systems, false, 1, false);
                    return;
                }
                _ => {
                    if self.text.len() >= self.limit {
                        return;
                    }
                    if let Key::Character(char) = key {
                        let can_proceed = if numeric_only {
                            is_numeric(&char.to_string())
                        } else {
                            true
                        };
                        if can_proceed {
                            self.text.insert(self.caret_pos, *char);
                            let size =
                                measure_string(systems, char.to_string())
                                    .x
                                    .floor();
                            self.char_size.insert(self.caret_pos, size);
                            self.move_caret_pos(systems, false, 1, false);
                            did_edit = true;
                        }
                    } else if Key::Named(NamedKey::Space) == *key
                        && !numeric_only
                    {
                        self.text.insert(self.caret_pos, ' ');
                        let size =
                            measure_string(systems, ' '.to_string()).x.floor();
                        self.char_size.insert(self.caret_pos, size);
                        self.move_caret_pos(systems, false, 1, false);
                        did_edit = true;
                    }
                }
            };
        }

        if did_edit {
            let msg = if self.hide_content {
                self.text.chars().map(|_| '*').collect()
            } else {
                self.text.clone()
            };
            systems
                .gfx
                .set_text(&mut systems.renderer, self.text_index, &msg);
        }
    }

    pub fn move_caret_pos(
        &mut self,
        systems: &mut SystemHolder,
        move_left: bool,
        count: usize,
        remove_content: bool,
    ) {
        let (start, end) = if move_left {
            let end = self.caret_pos;
            self.caret_pos = self.caret_pos.saturating_sub(count);
            (self.caret_pos, end)
        } else {
            let start = self.caret_pos;
            self.caret_pos =
                self.caret_pos.saturating_add(count).min(self.text.len());
            (start, self.caret_pos)
        };
        let size = measure_string(systems, self.text[start..end].to_string()).x;

        if move_left {
            self.caret_left -= size;
        } else {
            self.caret_left += size;
        }

        if self.caret_left < 0.0 {
            self.adjust_x += (self.caret_left * -1.0).max(0.0);
            self.caret_left = 0.0;
        } else if self.caret_left > self.size.x {
            self.adjust_x -= (self.caret_left - self.size.x).max(0.0);
            self.caret_left = self.size.x;
        }

        if remove_content {
            let total_size = measure_string(systems, self.text.clone()).x;
            if total_size > self.size.x {
                let visible_size = total_size + self.adjust_x;
                let leftover = self.size.x - visible_size;
                if leftover > 0.0 {
                    self.caret_left += leftover;
                    self.adjust_x += leftover;
                }
            } else if self.adjust_x < 0.0 {
                self.caret_left += self.adjust_x * -1.0;
                self.adjust_x = 0.0;
            }
        }

        systems.gfx.set_pos(
            self.text_index,
            Vec3::new(
                self.pos.x + self.adjust_x,
                self.pos.y,
                self.pos.z.sub_f32(self.z_step.0, self.z_step.1),
            ),
        );
        systems.gfx.set_pos(
            self.caret,
            Vec3::new(
                self.pos.x + self.caret_left,
                self.pos.y,
                self.pos.z.sub_f32(self.z_step.0, self.z_step.1),
            ),
        );
    }
}

/*
pub fn is_text(event: &KeyEvent) -> bool {
    matches!(
        event.physical_key,
        PhysicalKey::Code(
            KeyCode::KeyA
                | KeyCode::KeyB
                | KeyCode::KeyC
                | KeyCode::KeyD
                | KeyCode::KeyE
                | KeyCode::KeyF
                | KeyCode::KeyG
                | KeyCode::KeyH
                | KeyCode::KeyI
                | KeyCode::KeyJ
                | KeyCode::KeyK
                | KeyCode::KeyL
                | KeyCode::KeyM
                | KeyCode::KeyN
                | KeyCode::KeyO
                | KeyCode::KeyP
                | KeyCode::KeyQ
                | KeyCode::KeyR
                | KeyCode::KeyS
                | KeyCode::KeyT
                | KeyCode::KeyU
                | KeyCode::KeyV
                | KeyCode::KeyW
                | KeyCode::KeyX
                | KeyCode::KeyY
                | KeyCode::KeyZ
                | KeyCode::Digit1
                | KeyCode::Digit2
                | KeyCode::Digit3
                | KeyCode::Digit4
                | KeyCode::Digit5
                | KeyCode::Digit6
                | KeyCode::Digit7
                | KeyCode::Digit8
                | KeyCode::Digit9
                | KeyCode::Digit0
                | KeyCode::Comma
                | KeyCode::Period
                | KeyCode::BracketLeft
                | KeyCode::BracketRight
                | KeyCode::Backquote
                | KeyCode::Minus
                | KeyCode::Equal
                | KeyCode::Quote
                | KeyCode::Backslash
                | KeyCode::Semicolon
                | KeyCode::Slash
                | KeyCode::Space,
        )
    )
}
 */

pub fn is_numeric(char: &str) -> bool {
    char.trim().parse::<i64>().is_ok()
}

pub fn get_clipboard_text() -> String {
    let mut clipboard = Clipboard::new().unwrap();
    match clipboard.get_text() {
        Ok(data) => data,
        Err(_) => String::new(),
    }
}

pub fn set_clipboard_text(message: String) {
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text(message).unwrap();
}
