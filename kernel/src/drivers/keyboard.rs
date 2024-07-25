use core::fmt::{Display, LowerHex, UpperHex};

use crate::println;
use int_enum::IntEnum;
use macros::EncodeKey;

#[derive(Debug, Clone, Copy)]
pub struct Key {
    pub code: KeyCode, // each code has lower 5 bits as column while the highest 3 are row
}

// NOTE the ranges are based on the US-QWERTY layout
// NULL, F1 .. F12, print screen - row 0 - null is an invaild key it is 0
// esc, 1 .. 0, -, =, backspace - num row 1

// q .. p, [, ], \ - first alphabet row 2
// a .. l, :, ", enter - second alphabet row 3
// z .. m, '`', ',', '.', / - third alphabet row 4

// tab, caps lock, shift, ctrl, alt, super(windows), space, up, down, left, right - control row 5
// pg up, pg down, insert, delete, home, end - prob more numpad keys which i dont have row 6
/* empty row 7 */

macro_rules! row {
    ($row: expr) => {
        $row << 5
    };
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum KeyCode {
    // set the first key at N row to row!(N), then put the other keys in order
    NULL = row!(0),

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

impl Key {
    pub const fn default() -> Self {
        Self {
            code: KeyCode::NULL,
        }
    }
}

static mut CURRENT_UNENCODED_KEY: [u8; 8] = [0; 8]; // multibyte key
static mut LATEST_UNENCODED_BYTE: usize = 0; // pointer in ^^^

static mut CURRENT_KEYS: [Key; 256] = [Key::default(); 256];

pub trait EncodeKey: Sized {
    fn encode(self) -> KeyCode;
}

// EncodeKey macro defined in macros crate, basically you just
// need to add the keycode as a variant below, give it the same name as the key in KeyCode enum
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

    KeyQ = 0x10,
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

    let key: u64 = unsafe { core::mem::transmute(CURRENT_UNENCODED_KEY) };
    let key = Set1Key::try_from(key).unwrap_or(Set1Key::NULL);
    let encoded = key.encode();

    println!("0x{}", encoded);

    unsafe {
        CURRENT_UNENCODED_KEY = [0u8; 8];
        LATEST_UNENCODED_BYTE = 0;
    }
}
