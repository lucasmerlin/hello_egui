//! Key codes CEF expects for an [`egui::Key`].
//!
//! The codes of physical keys come from Chromium's own table, through the `keycode` crate.
//! Only the Windows virtual-key code, which CEF reads on every platform, is mapped here.

use std::str::FromStr as _;

use egui::Key;
use keycode::{KeyMap, KeyMappingCode as Code};

/// The codes of one key, as [`cef::KeyEvent`] takes them.
#[derive(Clone, Copy, Debug)]
pub struct KeyCodes {
    /// The Windows virtual-key code. CEF uses it on every platform.
    pub windows: i32,
    /// The platform's code for the physical key: a macOS `kVK_` code, an X11 key code, or a
    /// Windows `WM_KEYDOWN` `lParam`. Chromium takes `KeyboardEvent.code` from it.
    pub native: i32,
}

/// The codes of `key`, or `None` for keys CEF has no use for.
///
/// `physical` is the key at the same place on a US keyboard, if known. The native code
/// follows it, since native codes name a physical key.
pub fn key_codes(key: Key, physical: Option<Key>) -> Option<KeyCodes> {
    let windows = windows_key_code(dom_code(key)?)?;
    let native = physical
        .and_then(dom_code)
        .or_else(|| dom_code(key))
        .map_or(0, native_key_code);
    Some(KeyCodes { windows, native })
}

/// The `KeyboardEvent.code` of the key at `key`'s place on a US keyboard.
fn dom_code(key: Key) -> Option<Code> {
    let name = key.name();
    let code = match key {
        // egui names these after what they type, the DOM after where they are.
        Key::ArrowDown => "ArrowDown",
        Key::ArrowLeft => "ArrowLeft",
        Key::ArrowRight => "ArrowRight",
        Key::ArrowUp => "ArrowUp",
        Key::Colon => "Semicolon",
        Key::Plus | Key::Equals => "Equal",
        Key::Pipe => "Backslash",
        Key::Questionmark => "Slash",
        Key::Exclamationmark => "Digit1",
        Key::OpenBracket | Key::OpenCurlyBracket => "BracketLeft",
        Key::CloseBracket | Key::CloseCurlyBracket => "BracketRight",
        Key::Backtick => "Backquote",
        Key::SuperLeft => "MetaLeft",
        Key::SuperRight => "MetaRight",
        // egui names letters and digits `A` and `1`, the DOM `KeyA` and `Digit1`.
        _ if name.len() == 1 && name.as_bytes()[0].is_ascii_alphabetic() => {
            return Code::from_str(&format!("Key{name}")).ok();
        }
        _ if name.len() == 1 && name.as_bytes()[0].is_ascii_digit() => {
            return Code::from_str(&format!("Digit{name}")).ok();
        }
        // The rest have the same name in both.
        _ => name,
    };
    Code::from_str(code).ok()
}

fn native_key_code(code: Code) -> i32 {
    let map = KeyMap::from(code);
    if cfg!(target_os = "macos") {
        // `0xffff` marks a key macOS doesn't have.
        if map.mac == 0xFFFF {
            0
        } else {
            i32::from(map.mac)
        }
    } else if cfg!(target_os = "windows") {
        // Like the `lParam` of `WM_KEYDOWN`: a repeat count of 1, the scan code in bits 16 to
        // 23, and bit 24 for extended keys, whose scan codes start with `0xe0`.
        let scan_code = i32::from(map.win & 0xFF);
        let extended = i32::from(map.win >> 8 == 0xE0);
        1 | scan_code << 16 | extended << 24
    } else {
        i32::from(map.xkb)
    }
}

/// The Windows virtual-key code of the key at `code` on a US keyboard.
fn windows_key_code(code: Code) -> Option<i32> {
    let name = code.to_string();
    // `VK_A` to `VK_Z` and `VK_0` to `VK_9` are the ASCII codes of the key's letter or digit.
    if let Some(c) = name.strip_prefix("Key").or_else(|| name.strip_prefix("Digit")) {
        return c.bytes().next().map(i32::from);
    }
    // `VK_F1` is `0x70`, up to `VK_F24`.
    if let Some(n) = name.strip_prefix('F').and_then(|n| n.parse::<i32>().ok()) {
        return Some(0x6F + n);
    }
    Some(match code {
        Code::Backspace => 0x08,
        Code::Tab => 0x09,
        Code::Enter => 0x0D,
        Code::Escape => 0x1B,
        Code::Space => 0x20,
        Code::PageUp => 0x21,
        Code::PageDown => 0x22,
        Code::End => 0x23,
        Code::Home => 0x24,
        Code::ArrowLeft => 0x25,
        Code::ArrowUp => 0x26,
        Code::ArrowRight => 0x27,
        Code::ArrowDown => 0x28,
        Code::Insert => 0x2D,
        Code::Delete => 0x2E,
        Code::MetaLeft => 0x5B,
        Code::MetaRight => 0x5C,
        Code::ShiftLeft => 0xA0,
        Code::ShiftRight => 0xA1,
        Code::ControlLeft => 0xA2,
        Code::ControlRight => 0xA3,
        Code::AltLeft => 0xA4,
        Code::AltRight => 0xA5,
        Code::BrowserBack => 0xA6,
        Code::Semicolon => 0xBA,
        Code::Equal => 0xBB,
        Code::Comma => 0xBC,
        Code::Minus => 0xBD,
        Code::Period => 0xBE,
        Code::Slash => 0xBF,
        Code::Backquote => 0xC0,
        Code::BracketLeft => 0xDB,
        Code::Backslash => 0xDC,
        Code::BracketRight => 0xDD,
        Code::Quote => 0xDE,
        Code::IntlBackslash => 0xE2,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_egui_key_with_a_dom_code_has_a_virtual_key() {
        for key in Key::ALL {
            if let Some(code) = dom_code(*key) {
                assert!(
                    windows_key_code(code).is_some() || matches!(key, Key::Copy | Key::Cut | Key::Paste),
                    "{key:?} ({code}) has no virtual-key code"
                );
            }
        }
    }

    #[test]
    fn known_codes() {
        let codes = |key| key_codes(key, None).map(|c| c.windows);
        assert_eq!(codes(Key::A), Some(0x41));
        assert_eq!(codes(Key::Num7), Some(0x37));
        assert_eq!(codes(Key::F12), Some(0x7B));
        assert_eq!(codes(Key::Questionmark), Some(0xBF));
        assert_eq!(codes(Key::F25), None);

        let a = KeyMap::from(Code::KeyA);
        assert_eq!((a.mac, a.xkb), (0x00, 0x26));
    }
}
