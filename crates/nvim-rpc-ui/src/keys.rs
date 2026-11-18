//! Portable logical keys, committed text, and Neovim key notation.
//!
//! Backends translate platform key events into logical characters and modifiers.
//! Do not apply a physical Shift a second time to a character already translated
//! by the keyboard layout. IME commits use [`KeyInput::Text`]; clipboard blocks
//! use the paste API. [`KeyInput::Sequence`] interprets Neovim key notation.

use std::borrow::Cow;
use std::fmt;

/// One logical key, committed text, or an already encoded Neovim key sequence.
///
/// Text and sequences borrow their source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyInput<'a> {
    AsciiKey {
        key: AsciiKey,
        modifiers: Modifiers,
    },
    SpecialKey {
        key: SpecialKey,
        modifiers: Modifiers,
    },
    /// A Unicode scalar. ASCII and control characters are normalized too.
    Character {
        key: char,
        modifiers: Modifiers,
    },
    /// Committed text: literal `<` is escaped, without interpreting key notation.
    Text {
        text: &'a str,
    },
    /// A sequence in Neovim key notation, for example `"ihello<Esc>"`.
    Sequence {
        keys: &'a str,
    },
}

impl<'a> KeyInput<'a> {
    /// Converts an ASCII byte into logical key input.
    ///
    /// Returns `None` for bytes above 127. C0 controls map to named keys or
    /// their control-key equivalents. A byte
    /// alone cannot distinguish Backspace from Ctrl-H, or Tab from Ctrl-I;
    /// backends with that information should construct the logical key directly.
    pub fn from_ascii(key: u8, modifiers: Modifiers) -> Option<Self> {
        key.is_ascii()
            .then(|| Self::from_char(char::from(key), modifiers))
    }

    /// Converts a layout-translated character, including non-ASCII Unicode.
    pub fn from_char(key: char, modifiers: Modifiers) -> Self {
        if !key.is_ascii() {
            return Self::Character { key, modifiers };
        }
        let (key, modifiers) = normalize(key, modifiers);
        match key {
            KeyName::Special(key) => Self::SpecialKey { key, modifiers },
            KeyName::Character(key) => Self::AsciiKey {
                key: AsciiKey(key as u8),
                modifiers,
            },
        }
    }

    /// Returns this input in Neovim key notation.
    ///
    /// Literal `<` in [`Self::Text`] becomes `<LT>`; [`Self::Sequence`] is
    /// already encoded and is passed through unchanged. The result borrows the
    /// source text and can outlive this input. Escaping text allocates only
    /// when it contains `<`.
    pub fn to_nvim(self) -> EncodedKey<'a> {
        let data = match self {
            Self::AsciiKey { key, modifiers } => Encoding::Inline(InlineKey::new(
                KeyName::Character(char::from(key.0)),
                modifiers,
            )),
            Self::SpecialKey { key, modifiers } => {
                Encoding::Inline(InlineKey::new(KeyName::Special(key), modifiers))
            }
            Self::Character { key, modifiers } => {
                Encoding::Inline(InlineKey::character(key, modifiers))
            }
            Self::Text { text } => Encoding::Text(if text.contains('<') {
                Cow::Owned(text.replace('<', "<LT>"))
            } else {
                Cow::Borrowed(text)
            }),
            Self::Sequence { keys } => Encoding::Text(Cow::Borrowed(keys)),
        };
        EncodedKey(data)
    }
}

/// Logical keyboard modifiers applied to a key.
///
/// Translate toolkit flags according to the platform's keyboard mapping. Do
/// not apply Shift a second time to a character already translated by that
/// mapping.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    /// Alt or Meta, encoded as Neovim's `M-` modifier.
    pub alt: bool,
    /// Command or Super, encoded as Neovim's `D-` modifier.
    pub command: bool,
}
impl Modifiers {
    fn any(self) -> bool {
        self.shift || self.control || self.alt || self.command
    }
}

/// A printable ASCII key that has no named-key escaping requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsciiKey(u8);
impl AsciiKey {
    /// Creates a printable ASCII key that needs no named-key escaping.
    ///
    /// Returns `None` for bytes outside `b'!'..=b'~'`, and for `<`, `\\`, or `|`.
    /// Use [`KeyInput::from_ascii()`] for those characters, spaces, and controls.
    pub fn new(key: u8) -> Option<Self> {
        match key {
            b'<' | b'\\' | b'|' => None,
            b'!'..=b'~' => Some(Self(key)),
            _ => None,
        }
    }

    /// Returns the printable ASCII byte.
    pub const fn as_byte(self) -> u8 {
        self.0
    }
}

/// Named keys and control characters encoded with Neovim's `<...>` notation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialKey {
    /// The NUL character (U+0000), encoded as `<Nul>`.
    Null,
    Bar,
    Backspace,
    Backslash,
    /// Enter or Return, encoded as `<CR>`.
    Enter,
    Delete,
    Down,
    End,
    Escape,
    Home,
    Insert,
    Left,
    LessThan,
    LineFeed,
    PageDown,
    PageUp,
    Right,
    Space,
    Tab,
    Up,
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
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
    F32,
    F33,
    F34,
    F35,
}
impl SpecialKey {
    /// Returns the Neovim key name, excluding modifiers and brackets.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Null => "Nul",
            Self::Bar => "Bar",
            Self::Backspace => "BS",
            Self::Backslash => "Bslash",
            Self::Enter => "CR",
            Self::Delete => "Del",
            Self::Down => "Down",
            Self::End => "End",
            Self::Escape => "Esc",
            Self::Home => "Home",
            Self::Insert => "Insert",
            Self::Left => "Left",
            Self::LessThan => "LT",
            Self::LineFeed => "NL",
            Self::PageDown => "PageDown",
            Self::PageUp => "PageUp",
            Self::Right => "Right",
            Self::Space => "Space",
            Self::Tab => "Tab",
            Self::Up => "Up",
            Self::F1 => "F1",
            Self::F2 => "F2",
            Self::F3 => "F3",
            Self::F4 => "F4",
            Self::F5 => "F5",
            Self::F6 => "F6",
            Self::F7 => "F7",
            Self::F8 => "F8",
            Self::F9 => "F9",
            Self::F10 => "F10",
            Self::F11 => "F11",
            Self::F12 => "F12",
            Self::F13 => "F13",
            Self::F14 => "F14",
            Self::F15 => "F15",
            Self::F16 => "F16",
            Self::F17 => "F17",
            Self::F18 => "F18",
            Self::F19 => "F19",
            Self::F20 => "F20",
            Self::F21 => "F21",
            Self::F22 => "F22",
            Self::F23 => "F23",
            Self::F24 => "F24",
            Self::F25 => "F25",
            Self::F26 => "F26",
            Self::F27 => "F27",
            Self::F28 => "F28",
            Self::F29 => "F29",
            Self::F30 => "F30",
            Self::F31 => "F31",
            Self::F32 => "F32",
            Self::F33 => "F33",
            Self::F34 => "F34",
            Self::F35 => "F35",
        }
    }
}

/// Encoded key bytes that borrow unchanged source text.
///
/// Borrows source text independently of the original [`KeyInput`]. Inline keys
/// and escaped text are stored in this value.
/// Implements `AsRef<str>` and `Display` without materializing an extra string.
pub struct EncodedKey<'a>(Encoding<'a>);
enum Encoding<'a> {
    Inline(InlineKey),
    Text(Cow<'a, str>),
}
impl EncodedKey<'_> {
    /// Borrows the encoded UTF-8 key notation without copying it.
    pub fn as_str(&self) -> &str {
        match &self.0 {
            Encoding::Inline(key) => key.as_str(),
            Encoding::Text(text) => text,
        }
    }
}
impl AsRef<str> for EncodedKey<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl fmt::Display for EncodedKey<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy)]
enum KeyName {
    Character(char),
    Special(SpecialKey),
}
fn normalize(key: char, mut modifiers: Modifiers) -> (KeyName, Modifiers) {
    let special = match key {
        '\0' => SpecialKey::Null,
        '\x08' => SpecialKey::Backspace,
        '\t' => SpecialKey::Tab,
        '\n' => SpecialKey::LineFeed,
        '\r' => SpecialKey::Enter,
        '\x1b' => SpecialKey::Escape,
        ' ' => SpecialKey::Space,
        '<' => SpecialKey::LessThan,
        '\\' => SpecialKey::Backslash,
        '|' => SpecialKey::Bar,
        '\x7f' => SpecialKey::Delete,
        '\x01'..='\x1f' => {
            modifiers.control = true;
            let key = match key as u8 {
                0x1c => KeyName::Special(SpecialKey::Backslash),
                0x1d => KeyName::Character(']'),
                0x1e => KeyName::Character('^'),
                0x1f => KeyName::Character('_'),
                // This arm has 1 <= n <= 27; 97 + n <= 124 fits u8.
                n => KeyName::Character(char::from(b'a' + n - 1)),
            };
            return (key, modifiers);
        }
        _ => return (KeyName::Character(key), modifiers),
    };
    (KeyName::Special(special), modifiers)
}

// new() writes at most four two-byte prefixes, a SpecialKey::name() (at
// most PageDown.len() = 8 bytes) or one UTF-8 scalar (at most 4), and two
// brackets. Keep the SpecialKey names and this bound together when extending it.
const KEY_CAPACITY: usize = 4 * 2 + "PageDown".len() + 2;
struct InlineKey {
    bytes: [u8; KEY_CAPACITY],
    len: u8,
}
impl InlineKey {
    fn character(key: char, modifiers: Modifiers) -> Self {
        let (key, modifiers) = normalize(key, modifiers);
        Self::new(key, modifiers)
    }
    fn new(key: KeyName, modifiers: Modifiers) -> Self {
        let mut ret = Self {
            bytes: [0; KEY_CAPACITY],
            len: 0,
        };
        let enclosed = modifiers.any() || matches!(key, KeyName::Special(_));
        if enclosed {
            ret.push("<");
            for (enabled, name) in [
                (modifiers.shift, "S-"),
                (modifiers.control, "C-"),
                (modifiers.alt, "M-"),
                (modifiers.command, "D-"),
            ] {
                if enabled {
                    ret.push(name);
                }
            }
        }
        match key {
            KeyName::Character(key) => {
                let mut buf = [0; 4];
                ret.push(key.encode_utf8(&mut buf));
            }
            KeyName::Special(key) => ret.push(key.name()),
        }
        if enclosed {
            ret.push(">");
        }
        ret
    }
    fn push(&mut self, text: &str) {
        // Only new() calls push(), with the fragments bounded at KEY_CAPACITY.
        // Every prefix sum therefore fits the array and u8 (18 < 256).
        let start = usize::from(self.len);
        let end = start + text.len();
        self.bytes[start..end].copy_from_slice(text.as_bytes());
        self.len = end as u8;
    }
    fn as_str(&self) -> &str {
        // new() appends only complete &str fragments, so their concatenation
        // is UTF-8. push() keeps len <= KEY_CAPACITY, proving the slice bound.
        std::str::from_utf8(&self.bytes[..usize::from(self.len)])
            .expect("key encoding preserves UTF-8")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn modifiers(bits: u8) -> Modifiers {
        Modifiers {
            shift: bits & 1 != 0,
            control: bits & 2 != 0,
            alt: bits & 4 != 0,
            command: bits & 8 != 0,
        }
    }

    #[test]
    fn ascii_controls_and_reserved_keys_have_explicit_notation() {
        for (byte, expected) in [
            (0, "<Nul>"),
            (1, "<C-a>"),
            (7, "<C-g>"),
            (8, "<BS>"),
            (9, "<Tab>"),
            (10, "<NL>"),
            (11, "<C-k>"),
            (12, "<C-l>"),
            (13, "<CR>"),
            (14, "<C-n>"),
            (26, "<C-z>"),
            (27, "<Esc>"),
            (28, "<C-Bslash>"),
            (29, "<C-]>"),
            (30, "<C-^>"),
            (31, "<C-_>"),
            (32, "<Space>"),
            (b'<', "<LT>"),
            (b'\\', "<Bslash>"),
            (b'|', "<Bar>"),
            (127, "<Del>"),
        ] {
            let input = KeyInput::from_ascii(byte, Modifiers::default()).unwrap();
            assert_eq!(input.to_nvim().as_str(), expected, "byte {byte}");
        }
        for byte in 128..=255 {
            assert_eq!(KeyInput::from_ascii(byte, Modifiers::default()), None);
        }
    }

    #[test]
    fn all_ascii_modifier_combinations_fit_and_preserve_utf8() {
        for byte in 0..128 {
            for bits in 0..16 {
                let input = KeyInput::from_ascii(byte, modifiers(bits)).unwrap();
                let encoded = input.to_nvim();
                assert!(
                    encoded.as_str().len() <= KEY_CAPACITY,
                    "byte {byte}, modifiers {bits}"
                );
                assert!(matches!(encoded.0, Encoding::Inline(_)));
            }
        }
        assert_eq!(
            KeyInput::SpecialKey {
                key: SpecialKey::PageDown,
                modifiers: modifiers(15)
            }
            .to_nvim()
            .as_str(),
            "<S-C-M-D-PageDown>"
        );
        assert_eq!(
            KeyInput::Character {
                key: '😀',
                modifiers: modifiers(15)
            }
            .to_nvim()
            .as_str(),
            "<S-C-M-D-😀>"
        );
    }

    #[test]
    fn logical_characters_keep_case_and_unicode() {
        for (key, modifiers, expected) in [
            ('a', Modifiers::default(), "a"),
            ('A', Modifiers::default(), "A"),
            ('é', Modifiers::default(), "é"),
            ('界', Modifiers::default(), "界"),
            ('ä', modifiers(4), "<M-ä>"),
            ('a', modifiers(3), "<S-C-a>"),
            ('>', modifiers(2), "<C->>"),
            ('<', modifiers(2), "<C-LT>"),
        ] {
            let input = KeyInput::from_char(key, modifiers);
            assert_eq!(input.to_nvim().as_str(), expected);
        }
    }

    #[test]
    fn committed_text_escapes_notation_but_sequences_are_borrowed_without_length_cap() {
        let input = KeyInput::Text {
            text: "日本語<CR>\\|>e\u{301}",
        };
        assert_eq!(input.to_nvim().as_str(), "日本語<LT>CR>\\|>e\u{301}");
        let text = String::from("日本語");
        let encoded = KeyInput::Text { text: &text }.to_nvim();
        assert_eq!(encoded.as_str(), text);
        assert_eq!(encoded.as_str().as_ptr(), text.as_ptr());
        let keys = "a<Esc>".repeat(100);
        let encoded = KeyInput::Sequence { keys: &keys }.to_nvim();
        assert_eq!(encoded.as_str(), keys);
        assert_eq!(encoded.as_str().as_ptr(), keys.as_ptr());
        assert_eq!(KeyInput::Sequence { keys: "" }.to_nvim().as_str(), "");
    }

    #[test]
    fn ascii_keys_validate_their_printable_invariant() {
        for byte in 0..=255 {
            let key = AsciiKey::new(byte);
            let expected = (b'!'..=b'~').contains(&byte) && !b"<\\|".contains(&byte);
            assert_eq!(key.is_some(), expected, "byte {byte}");
            if let Some(key) = key {
                assert_eq!(key.as_byte(), byte);
            }
        }
    }
}
