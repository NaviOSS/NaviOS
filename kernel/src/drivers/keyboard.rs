use core::fmt::{Display, LowerHex, UpperHex};

use crate::utils::{Locked, Optional};
use alloc::vec::Vec;
use bitflags::bitflags;
use int_enum::IntEnum;
use macros::EncodeKey;
use spin::MutexGuard;

static mut CURRENT_UNENCODED_KEY: [u8; 8] = [0; 8]; // multibyte key
static mut LATEST_UNENCODED_BYTE: usize = 0; // pointer in ^^^

static CURRENT_KEYS: Locked<Vec<Key>> = Locked::new(Vec::new());
fn current_keys() -> MutexGuard<'static, Vec<Key>> {
    CURRENT_KEYS.inner.lock()
}

#[no_mangle]
pub extern "C" fn __navi_keyboard_get_pressed_key_flags(code: KeyCode) -> Optional<KeyFlags> {
    for key in &*current_keys() {
        if key.code == code {
            let key = key.clone();
            return Optional::Some(key.flags);
        }
    }
    Optional::None
}

#[no_mangle]
pub extern "C" fn __navi_keyboard_key_is_pressed(code: KeyCode, flags: KeyFlags) -> bool {
    let key = Key::new(code, flags);

    for pressed_key in &*current_keys() {
        if *pressed_key == key {
            return true;
        }
    }

    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    pub code: KeyCode, // each code has lower 5 bits as column while the highest 3 are row
    pub flags: KeyFlags,
}

impl Key {
    const CTRL_KEY: Key = Self::new(KeyCode::Ctrl, KeyFlags::empty());
    const SHIFT_KEY: Key = Self::new(KeyCode::Shift, KeyFlags::empty());
    const ALT_KEY: Key = Self::new(KeyCode::Alt, KeyFlags::empty());
    const CAPSLOCK_KEY: Key = Self::new(KeyCode::CapsLock, KeyFlags::empty());

    #[inline]
    pub fn is_pressed(&self) -> bool {
        __navi_keyboard_key_is_pressed(self.code, self.flags)
    }

    // returns a Key with flags from keycode
    pub fn process_keycode(keycode: KeyCode) -> Self {
        let mut flags = KeyFlags::empty();

        if Self::CTRL_KEY.is_pressed() && keycode != KeyCode::Ctrl {
            flags |= KeyFlags::CTRL;
        }

        if Self::SHIFT_KEY.is_pressed() && keycode != KeyCode::Shift {
            flags |= KeyFlags::SHIFT;
        }

        if Self::ALT_KEY.is_pressed() && keycode != KeyCode::Alt {
            flags |= KeyFlags::ALT;
        }

        if Self::CAPSLOCK_KEY.is_pressed() && keycode != KeyCode::CapsLock {
            flags |= KeyFlags::CAPS_LOCK;
        }

        Self::new(keycode, flags)
    }

    pub const fn new(code: KeyCode, flags: KeyFlags) -> Self {
        Self { code, flags }
    }

    pub const fn default() -> Self {
        Self {
            code: KeyCode::NULL,
            flags: KeyFlags::empty(),
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(C)]
    pub struct KeyFlags: u8 {
        const CTRL = 1 << 0;
        const ALT = 1 << 1;
        const SHIFT = 1 << 2;
        const CAPS_LOCK = 1 << 3;
    }
}

macro_rules! row {
    ($row: expr) => {
        $row << 5
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum KeyCode {
    // set the first key at index N row to row!(N), then put the other keys in order
    NULL = row!(0),
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    PrintScr,

    Esc = row!(1),
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Key0,
    Minus,
    Equals,
    Backspace,

    KeyQ = row!(2),
    KeyW,
    KeyE,
    KeyR,
    KeyT,
    KeyY,
    KeyU,
    KeyI,
    KeyO,
    KeyP,
    LeftBrace,
    RightBrace,
    BackSlash,

    KeyA = row!(3),
    KeyS,
    KeyD,
    KeyF,
    KeyG,
    KeyH,
    KeyJ,
    KeyK,
    KeyL,
    Semicolon,
    DoubleQuote,
    Return,

    KeyZ = row!(4),
    KeyX,
    KeyC,
    KeyV,
    KeyB,
    KeyN,
    KeyM,
    BackQuote,
    Comma,
    Dot,
    Slash,

    Tab = row!(5),
    CapsLock,
    Ctrl,
    Shift,
    Alt,
    Super,
    Space,
    Up,
    Down,
    Left,
    Right,

    PageUp = row!(6),
    PageDown,
    Insert,
    Delete,
    Home,
    End,
}

impl LowerHex for KeyCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        LowerHex::fmt(&(*self as u8), f)
    }
}

impl UpperHex for KeyCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        UpperHex::fmt(&(*self as u8), f)
    }
}

impl Display for KeyCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        UpperHex::fmt(&self, f)
    }
}

pub trait EncodeKey: Sized {
    fn encode(self) -> KeyCode;
}

// you need to add the keycode as a variant below, give it the same name as the key in KeyCode enum
#[repr(u64)]
#[derive(IntEnum, Clone, Copy, EncodeKey)]
pub enum Set1Key {
    NULL = 0,
    // row 1
    Esc = 0x1,
    Key1 = 0x2,
    Key2 = 0x3,
    Key3 = 0x4,
    Key4 = 0x5,
    Key5 = 0x6,
    Key6 = 0x7,
    Key7 = 0x8,
    Key8 = 0x9,
    Key9 = 0xA,
    Key0 = 0xB,
    Minus = 0xC,
    Equals = 0xD,
    Backspace = 0xE,

    // row 2
    KeyQ = 0x10,

    // row 6
    PageUp = 0x49e0,
    PageDown = 0x51e0,
}
#[inline]
fn reset_unencoded_buffer() {
    unsafe {
        CURRENT_UNENCODED_KEY = [0u8; 8];
        LATEST_UNENCODED_BYTE = 0;
    }
}

fn add_pressed_keycode(code: KeyCode) {
    if code == KeyCode::NULL {
        return;
    }

    let key = Key::process_keycode(code);
    current_keys().push(key);
}

fn remove_pressed_keycode(code: KeyCode) {
    if code == KeyCode::NULL {
        return;
    }

    let mut current_keys = current_keys();
    let key = current_keys
        .iter()
        .enumerate()
        .find(|(_, key)| key.code == code);

    if let Some((index, _)) = key {
        current_keys.remove(index);
    }
}

pub fn encode_ps2_set_1(code: u8) {
    // for any prefix we add it to CURRENT_UNENCODED_KEY and we dont start encoding
    unsafe {
        CURRENT_UNENCODED_KEY[LATEST_UNENCODED_BYTE] = code;
        if code == 0xE0 {
            LATEST_UNENCODED_BYTE += 1;
            return;
        }
    };

    let break_code;

    unsafe {
        if CURRENT_UNENCODED_KEY[LATEST_UNENCODED_BYTE] & 128 == 128 {
            CURRENT_UNENCODED_KEY[LATEST_UNENCODED_BYTE] -= 0x80;
            break_code = true;
        } else {
            break_code = false;
        }
    }

    let key: u64 = unsafe { core::mem::transmute(CURRENT_UNENCODED_KEY) };
    let key = Set1Key::try_from(key).unwrap_or(Set1Key::NULL);
    let encoded = key.encode();

    if break_code {
        remove_pressed_keycode(encoded)
    } else {
        add_pressed_keycode(encoded)
    }

    reset_unencoded_buffer()
}
