//! Subset of X11 keysyms used by the engine.

pub const BACKSPACE: u32 = 0xff08;
pub const RETURN: u32 = 0xff0d;
pub const TAB: u32 = 0xff09;
pub const ESCAPE: u32 = 0xff1b;
pub const SPACE: u32 = 0x0020;
pub const SHIFT_L: u32 = 0xffe1;
pub const SHIFT_R: u32 = 0xffe2;
pub const CONTROL_L: u32 = 0xffe3;
pub const CONTROL_R: u32 = 0xffe4;
pub const ALT_L: u32 = 0xffe9;
pub const ALT_R: u32 = 0xffea;
pub const SUPER_L: u32 = 0xffeb;
pub const SUPER_R: u32 = 0xffec;

/// IBus / GDK: bit 30 set on key-release events.
pub const RELEASE_MASK: u32 = 1 << 30;
pub const CONTROL_MASK: u32 = 1 << 2;
pub const MOD1_MASK: u32 = 1 << 3; // Alt
pub const MOD4_MASK: u32 = 1 << 6; // Super
pub const IGNORED_MASK: u32 = RELEASE_MASK | CONTROL_MASK | MOD1_MASK | MOD4_MASK;

pub fn is_modifier(keyval: u32) -> bool {
    matches!(
        keyval,
        SHIFT_L
            | SHIFT_R
            | CONTROL_L
            | CONTROL_R
            | ALT_L
            | ALT_R
            | SUPER_L
            | SUPER_R
            | 0xffe5 // Caps_Lock
            | 0xff7e // Mode_switch
            | 0xfe03 // ISO_Level3_Shift
    )
}

/// Convert keyval to a Unicode char when it is a printable BMP character.
pub fn keyval_to_char(keyval: u32) -> Option<char> {
    if keyval == SPACE {
        return Some(' ');
    }
    if keyval == TAB {
        return Some('\t');
    }
    if keyval == RETURN {
        return Some('\n');
    }
    // Latin-1 printable and beyond via direct keyval==unicode (common for a-z, 0-9)
    if (0x20..=0xff).contains(&keyval) {
        return char::from_u32(keyval);
    }
    None
}
