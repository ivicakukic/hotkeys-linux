/// Linux virtual key mapping for HotKeys
/// Maps Windows VK codes to Linux KEY_* constants from linux/input-event-codes.h
///
/// References:
/// - https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h
/// - Windows VK codes: https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes

use paste::paste;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualKey<'a> {
    pub vkey: u16,        // Windows VK code (for compatibility)
    pub linux_key: u16,   // Linux KEY_* code
    pub title: &'a str,
}

impl<'a> VirtualKey<'a> {
    const fn new(vkey: u16, linux_key: u16, title: &'a str) -> Self {
        Self { vkey, linux_key, title }
    }

    pub fn matches(&self, text: &str) -> bool {
        self.title.eq_ignore_ascii_case(text)
    }
}

macro_rules! virtual_keys {
    ($($name:tt, $vkey:tt, $linux_key:tt, $text:tt;)*) => {
        $(
            paste! {
                pub const [<VK_ $name:upper>]: VirtualKey = VirtualKey::new($vkey, $linux_key, $text);
            }
        )*
        pub const ALL_KEYS: &'static [&'static VirtualKey] = &[$(
            &paste!{[<VK_ $name:upper>]}
        ),*];
    }
}

// Linux KEY_* constants from input-event-codes.h
const KEY_BACKSPACE: u16 = 14;
const KEY_TAB: u16 = 15;
const KEY_ENTER: u16 = 28;
const KEY_LEFTCTRL: u16 = 29;
const KEY_LEFTALT: u16 = 56;
const KEY_RIGHTALT: u16 = 100;
const KEY_RIGHTCTRL: u16 = 97;
const KEY_LEFTSHIFT: u16 = 42;
const KEY_RIGHTSHIFT: u16 = 54;
const KEY_CAPSLOCK: u16 = 58;
const KEY_ESC: u16 = 1;
const KEY_SPACE: u16 = 57;
const KEY_PAGEUP: u16 = 104;
const KEY_PAGEDOWN: u16 = 109;
const KEY_END: u16 = 107;
const KEY_HOME: u16 = 102;
const KEY_LEFT: u16 = 105;
const KEY_UP: u16 = 103;
const KEY_RIGHT: u16 = 106;
const KEY_DOWN: u16 = 108;
const KEY_PRINTSCREEN: u16 = 99;
const KEY_INSERT: u16 = 110;
const KEY_DELETE: u16 = 111;

// Number keys
const KEY_0: u16 = 11;
const KEY_1: u16 = 2;
const KEY_2: u16 = 3;
const KEY_3: u16 = 4;
const KEY_4: u16 = 5;
const KEY_5: u16 = 6;
const KEY_6: u16 = 7;
const KEY_7: u16 = 8;
const KEY_8: u16 = 9;
const KEY_9: u16 = 10;

// Letter keys
const KEY_A: u16 = 30;
const KEY_B: u16 = 48;
const KEY_C: u16 = 46;
const KEY_D: u16 = 32;
const KEY_E: u16 = 18;
const KEY_F: u16 = 33;
const KEY_G: u16 = 34;
const KEY_H: u16 = 35;
const KEY_I: u16 = 23;
const KEY_J: u16 = 36;
const KEY_K: u16 = 37;
const KEY_L: u16 = 38;
const KEY_M: u16 = 50;
const KEY_N: u16 = 49;
const KEY_O: u16 = 24;
const KEY_P: u16 = 25;
const KEY_Q: u16 = 16;
const KEY_R: u16 = 19;
const KEY_S: u16 = 31;
const KEY_T: u16 = 20;
const KEY_U: u16 = 22;
const KEY_V: u16 = 47;
const KEY_W: u16 = 17;
const KEY_X: u16 = 45;
const KEY_Y: u16 = 21;
const KEY_Z: u16 = 44;

// Function keys
const KEY_F1: u16 = 59;
const KEY_F2: u16 = 60;
const KEY_F3: u16 = 61;
const KEY_F4: u16 = 62;
const KEY_F5: u16 = 63;
const KEY_F6: u16 = 64;
const KEY_F7: u16 = 65;
const KEY_F8: u16 = 66;
const KEY_F9: u16 = 67;
const KEY_F10: u16 = 68;
const KEY_F11: u16 = 87;
const KEY_F12: u16 = 88;

// Numpad
const KEY_KP0: u16 = 82;
const KEY_KP1: u16 = 79;
const KEY_KP2: u16 = 80;
const KEY_KP3: u16 = 81;
const KEY_KP4: u16 = 75;
const KEY_KP5: u16 = 76;
const KEY_KP6: u16 = 77;
const KEY_KP7: u16 = 71;
const KEY_KP8: u16 = 72;
const KEY_KP9: u16 = 73;
const KEY_KPASTERISK: u16 = 55;
const KEY_KPPLUS: u16 = 78;
const KEY_KPMINUS: u16 = 74;
const KEY_KPDOT: u16 = 83;
const KEY_KPSLASH: u16 = 98;
// const KEY_KPENTER: u16 = 104;
// const KEY_KPDELETE: u16 = 91;

// Punctuation
const KEY_SEMICOLON: u16 = 39;
const KEY_EQUAL: u16 = 13;
const KEY_COMMA: u16 = 51;
const KEY_MINUS: u16 = 12;
const KEY_DOT: u16 = 52;
const KEY_SLASH: u16 = 53;
const KEY_GRAVE: u16 = 41;
const KEY_LEFTBRACE: u16 = 26;
const KEY_BACKSLASH: u16 = 43;
const KEY_RIGHTBRACE: u16 = 27;
const KEY_APOSTROPHE: u16 = 40;

// Special keys
const KEY_LEFTMETA: u16 = 125;  // Windows key
const KEY_RIGHTMETA: u16 = 126; // Windows key
const KEY_NUMLOCK: u16 = 69;
const KEY_SCROLLLOCK: u16 = 70;
const KEY_PAUSE: u16 = 119;

virtual_keys! {
    "back",         0x08,   KEY_BACKSPACE,   "back";
    "tab",          0x09,   KEY_TAB,         "tab";
    "enter",        0x0D,   KEY_ENTER,       "enter";
    "shift",        0x10,   KEY_LEFTSHIFT,   "shift";
    "ctrl",         0x11,   KEY_LEFTCTRL,    "ctrl";
    "alt",          0x12,   KEY_LEFTALT,     "alt";
    "pause",        0x13,   KEY_PAUSE,       "pause";
    "capslock",     0x14,   KEY_CAPSLOCK,    "capslock";
    "esc",          0x1B,   KEY_ESC,         "esc";
    "space",        0x20,   KEY_SPACE,       "space";
    "pgup",         0x21,   KEY_PAGEUP,      "pgup";
    "pgdown",       0x22,   KEY_PAGEDOWN,    "pgdown";
    "end",          0x23,   KEY_END,         "end";
    "home",         0x24,   KEY_HOME,        "home";
    "larrow",       0x25,   KEY_LEFT,        "larrow";
    "uarrow",       0x26,   KEY_UP,          "uarrow";
    "rarrow",       0x27,   KEY_RIGHT,       "rarrow";
    "darrow",       0x28,   KEY_DOWN,        "darrow";
    "prtscrn",      0x2C,   KEY_PRINTSCREEN, "prtscrn";
    "ins",          0x2D,   KEY_INSERT,      "ins";
    "del",          0x2E,   KEY_DELETE,      "del";
    "0",            0x30,   KEY_0,           "0";
    "1",            0x31,   KEY_1,           "1";
    "2",            0x32,   KEY_2,           "2";
    "3",            0x33,   KEY_3,           "3";
    "4",            0x34,   KEY_4,           "4";
    "5",            0x35,   KEY_5,           "5";
    "6",            0x36,   KEY_6,           "6";
    "7",            0x37,   KEY_7,           "7";
    "8",            0x38,   KEY_8,           "8";
    "9",            0x39,   KEY_9,           "9";
    "a",            0x41,   KEY_A,           "a";
    "b",            0x42,   KEY_B,           "b";
    "c",            0x43,   KEY_C,           "c";
    "d",            0x44,   KEY_D,           "d";
    "e",            0x45,   KEY_E,           "e";
    "f",            0x46,   KEY_F,           "f";
    "g",            0x47,   KEY_G,           "g";
    "h",            0x48,   KEY_H,           "h";
    "i",            0x49,   KEY_I,           "i";
    "j",            0x4A,   KEY_J,           "j";
    "k",            0x4B,   KEY_K,           "k";
    "l",            0x4C,   KEY_L,           "l";
    "m",            0x4D,   KEY_M,           "m";
    "n",            0x4E,   KEY_N,           "n";
    "o",            0x4F,   KEY_O,           "o";
    "p",            0x50,   KEY_P,           "p";
    "q",            0x51,   KEY_Q,           "q";
    "r",            0x52,   KEY_R,           "r";
    "s",            0x53,   KEY_S,           "s";
    "t",            0x54,   KEY_T,           "t";
    "u",            0x55,   KEY_U,           "u";
    "v",            0x56,   KEY_V,           "v";
    "w",            0x57,   KEY_W,           "w";
    "x",            0x58,   KEY_X,           "x";
    "y",            0x59,   KEY_Y,           "y";
    "z",            0x5A,   KEY_Z,           "z";
    "lwin",         0x5B,   KEY_LEFTMETA,    "lwin";
    "rwin",         0x5C,   KEY_RIGHTMETA,   "rwin";
    "numpad0",      0x60,   KEY_KP0,         "numpad0";
    "numpad1",      0x61,   KEY_KP1,         "numpad1";
    "numpad2",      0x62,   KEY_KP2,         "numpad2";
    "numpad3",      0x63,   KEY_KP3,         "numpad3";
    "numpad4",      0x64,   KEY_KP4,         "numpad4";
    "numpad5",      0x65,   KEY_KP5,         "numpad5";
    "numpad6",      0x66,   KEY_KP6,         "numpad6";
    "numpad7",      0x67,   KEY_KP7,         "numpad7";
    "numpad8",      0x68,   KEY_KP8,         "numpad8";
    "numpad9",      0x69,   KEY_KP9,         "numpad9";
    "multiply",     0x6A,   KEY_KPASTERISK,  "multiply";
    "add",          0x6B,   KEY_KPPLUS,      "add";
    "subtract",     0x6D,   KEY_KPMINUS,     "subtract";
    "decimal",      0x6E,   KEY_KPDOT,       "decimal";
    "divide",       0x6F,   KEY_KPSLASH,     "divide";
//    "numpaddelete", 0xDF,   KEY_KPDELETE,    "numpaddelete"; // DF is not real, there is no windows Virtual Key for Numpad Delete
//    "numpadenter",  0xDF,   KEY_KPENTER,     "numpadenter";  // DF is not real, there is no windows Virtual Key for Numpad Enter  -> TODO: check if these (+others) dont work only on Wayland
    "f1",           0x70,   KEY_F1,          "f1";
    "f2",           0x71,   KEY_F2,          "f2";
    "f3",           0x72,   KEY_F3,          "f3";
    "f4",           0x73,   KEY_F4,          "f4";
    "f5",           0x74,   KEY_F5,          "f5";
    "f6",           0x75,   KEY_F6,          "f6";
    "f7",           0x76,   KEY_F7,          "f7";
    "f8",           0x77,   KEY_F8,          "f8";
    "f9",           0x78,   KEY_F9,          "f9";
    "f10",          0x79,   KEY_F10,         "f10";
    "f11",          0x7A,   KEY_F11,         "f11";
    "f12",          0x7B,   KEY_F12,         "f12";
    "numlock",      0x90,   KEY_NUMLOCK,     "numlock";
    "scrllock",     0x91,   KEY_SCROLLLOCK,  "scrllock";
    "lshift",       0xA0,   KEY_LEFTSHIFT,   "lshift";
    "rshift",       0xA1,   KEY_RIGHTSHIFT,  "rshift";
    "lctrl",        0xA2,   KEY_LEFTCTRL,    "lctrl";
    "rctrl",        0xA3,   KEY_RIGHTCTRL,   "rctrl";
    "lalt",         0xA4,   KEY_LEFTALT,     "lalt";
    "ralt",         0xA5,   KEY_RIGHTALT,    "ralt";
    "semicol",      0xBA,   KEY_SEMICOLON,   ";";
    "plus",         0xBB,   KEY_EQUAL,       "=";
    "comma",        0xBC,   KEY_COMMA,       ",";
    "minus",        0xBD,   KEY_MINUS,       "-";
    "dot",          0xBE,   KEY_DOT,         ".";
    "slash",        0xBF,   KEY_SLASH,       "/";
    "tick",         0xC0,   KEY_GRAVE,       "`";
    "lsbrck",       0xDB,   KEY_LEFTBRACE,   "[";
    "backslash",    0xDC,   KEY_BACKSLASH,   "\\";
    "rsbrck",       0xDD,   KEY_RIGHTBRACE,  "]";
    "sqote",        0xDE,   KEY_APOSTROPHE,  "'";
}

pub fn find_vkey(text: &str) -> Result<&'static VirtualKey<'static>, &'static str> {
    ALL_KEYS.iter()
        .find(|vk| vk.matches(text))
        .copied()
        .ok_or("Unknown virtual key")
}

pub fn get_vkey(vk_code: u16) -> Result<&'static VirtualKey<'static>, &'static str> {
    ALL_KEYS.iter()
        .find(|vk| vk.vkey == vk_code)
        .copied()
        .ok_or("Unknown virtual key code")
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_keys() {
        assert_eq!(Ok(&VK_F1), find_vkey("f1"));
        assert_eq!(Ok(&VK_F1), find_vkey("F1")); // Case insensitive
        assert_eq!(Ok(&VK_NUMLOCK), find_vkey("numlock"));
        assert_eq!(Ok(&VK_P), find_vkey("p"));
        assert_eq!(Ok(&VK_CTRL), find_vkey("ctrl"));
        assert_eq!(Ok(&VK_ENTER), find_vkey("enter"));
        assert_eq!(Err("Unknown virtual key"), find_vkey("nonexistent"));
    }

    #[test]
    fn test_get_vkey() {
        assert_eq!(Ok(&VK_F1), get_vkey(0x70));
        assert_eq!(Ok(&VK_NUMLOCK), get_vkey(0x90));
        assert_eq!(Err("Unknown virtual key code"), get_vkey(0xFFFF));
    }

    #[test]
    fn test_linux_key_mapping() {
        assert_eq!(VK_A.linux_key, KEY_A);
        assert_eq!(VK_ENTER.linux_key, KEY_ENTER);
        assert_eq!(VK_F1.linux_key, KEY_F1);
        assert_eq!(VK_CTRL.linux_key, KEY_LEFTCTRL);
    }

    #[test]
    fn test_compatibility_vkeys() {
        // Windows VK codes should be preserved for compatibility
        assert_eq!(VK_A.vkey, 0x41);
        assert_eq!(VK_ENTER.vkey, 0x0D);
        assert_eq!(VK_F1.vkey, 0x70);
    }
}