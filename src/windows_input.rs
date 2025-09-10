use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::io;
use winapi::um::winuser::{
    GetKeyState, GetKeyboardState, ToUnicode, VK_CONTROL, VK_MENU, VK_SHIFT,
};

pub fn read_key_events() -> io::Result<Vec<Event>> {
    let mut events = Vec::new();
    let mut key_states = [0u8; 256];
    let mut old_key_states = [0u8; 256];

    unsafe {
        if GetKeyboardState(key_states.as_mut_ptr()) == 0 {
            return Err(io::Error::last_os_error());
        }
    }

    for (vk_code, (current_state, old_state)) in
        key_states.iter().zip(old_key_states.iter()).enumerate()
    {
        if (current_state & 0x80) != (old_state & 0x80) {
            let key_code = vk_to_keycode(vk_code as i32);
            let kind = if (current_state & 0x80) != 0 {
                crossterm::event::KeyEventKind::Press
            } else {
                crossterm::event::KeyEventKind::Release
            };

            let modifiers = get_key_modifiers();
            let state = crossterm::event::KeyEventState::empty();

            if let Some(key_code) = key_code {
                events.push(Event::Key(KeyEvent {
                    code: key_code,
                    modifiers,
                    kind,
                    state,
                }));
            }
        }
    }

    old_key_states.copy_from_slice(&key_states);
    Ok(events)
}

fn get_key_modifiers() -> KeyModifiers {
    let mut modifiers = KeyModifiers::empty();
    unsafe {
        if GetKeyState(VK_SHIFT) & 0x8000u16 as i16 != 0 {
            modifiers |= KeyModifiers::SHIFT;
        }
        if GetKeyState(VK_CONTROL) & 0x8000u16 as i16 != 0 {
            modifiers |= KeyModifiers::CONTROL;
        }
        if GetKeyState(VK_MENU) & 0x8000u16 as i16 != 0 {
            modifiers |= KeyModifiers::ALT;
        }
    }
    modifiers
}

fn vk_to_keycode(vk_code: i32) -> Option<KeyCode> {
    use winapi::um::winuser::*;
    match vk_code {
        VK_BACK => Some(KeyCode::Backspace),
        VK_RETURN => Some(KeyCode::Enter),
        VK_LEFT => Some(KeyCode::Left),
        VK_RIGHT => Some(KeyCode::Right),
        VK_UP => Some(KeyCode::Up),
        VK_DOWN => Some(KeyCode::Down),
        VK_HOME => Some(KeyCode::Home),
        VK_END => Some(KeyCode::End),
        VK_PRIOR => Some(KeyCode::PageUp),
        VK_NEXT => Some(KeyCode::PageDown),
        VK_TAB => Some(KeyCode::Tab),
        VK_DELETE => Some(KeyCode::Delete),
        VK_INSERT => Some(KeyCode::Insert),
        VK_ESCAPE => Some(KeyCode::Esc),
        VK_F1 => Some(KeyCode::F(1)),
        VK_F2 => Some(KeyCode::F(2)),
        VK_F3 => Some(KeyCode::F(3)),
        VK_F4 => Some(KeyCode::F(4)),
        VK_F5 => Some(KeyCode::F(5)),
        VK_F6 => Some(KeyCode::F(6)),
        VK_F7 => Some(KeyCode::F(7)),
        VK_F8 => Some(KeyCode::F(8)),
        VK_F9 => Some(KeyCode::F(9)),
        VK_F10 => Some(KeyCode::F(10)),
        VK_F11 => Some(KeyCode::F(11)),
        VK_F12 => Some(KeyCode::F(12)),
        _ => {
            let mut buffer = [0u16; 1];
            let mut keyboard_state = [0u8; 256];
            unsafe {
                if GetKeyboardState(keyboard_state.as_mut_ptr()) == 0 {
                    return None;
                }
                let result = ToUnicode(
                    vk_code as u32,
                    0,
                    keyboard_state.as_ptr(),
                    buffer.as_mut_ptr(),
                    1,
                    0,
                );
                if result == 1 {
                    let c = std::char::from_u32(buffer[0] as u32).unwrap_or_default();
                    if c.is_alphanumeric() || c.is_ascii_punctuation() {
                        return Some(KeyCode::Char(c));
                    }
                }
            }
            None
        }
    }
}