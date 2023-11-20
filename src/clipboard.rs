#[cfg(feature = "clipboard")]
use arboard::Clipboard as Arboard;
use std::borrow::Cow;
#[cfg(feature = "clipboard")]
use std::cell::RefCell;
use std::fmt;

pub enum ClipboardContent<'a> {
    Piece(Cow<'a, str>),
    Chunk(Cow<'a, [String]>),
}

impl<'a> From<ClipboardContent<'a>> for String {
    fn from(contents: ClipboardContent<'a>) -> String {
        match contents {
            ClipboardContent::Piece(s) => s.into_owned(),
            ClipboardContent::Chunk(ss) => ss.join("\n"),
        }
    }
}

pub enum Clipboard {
    Piece(String),
    Chunk(Vec<String>),
    #[cfg(feature = "clipboard")]
    Os(RefCell<Arboard>), // Use `RefCell` not to make `Clipboard::contents` mut method
}

impl Default for Clipboard {
    fn default() -> Self {
        #[cfg(feature = "clipboard")]
        if let Ok(ctx) = Arboard::new() {
            return Self::Os(RefCell::new(ctx));
        }
        Self::Piece(String::new())
    }
}

impl Clipboard {
    pub fn set_piece(&mut self, s: String) {
        debug_assert!(
            !s.contains('\n'),
            "multi-line test is passed to Clipboard::set_piece: {s:?}",
        );

        #[cfg(feature = "clipboard")]
        if let Self::Os(ctx) = self {
            if let Ok(mut ctx) = ctx.try_borrow_mut() {
                let _ = ctx.set_text(s);
                return;
            }
        }

        *self = Self::Piece(s);
    }

    pub fn set_chunk(&mut self, mut c: Vec<String>) {
        match c.len() {
            0 => self.set_piece(String::new()),
            1 => self.set_piece(c.remove(0)),
            _ => {
                #[cfg(feature = "clipboard")]
                if let Self::Os(ctx) = self {
                    if let Ok(mut ctx) = ctx.try_borrow_mut() {
                        let _ = ctx.set_text(c.join("\n"));
                        return;
                    }
                }
                *self = Self::Chunk(c);
            }
        }
    }

    pub fn contents(&self) -> ClipboardContent<'_> {
        match self {
            Self::Piece(p) => ClipboardContent::Piece(p.as_str().into()),
            Self::Chunk(c) => ClipboardContent::Chunk(c.as_slice().into()),
            #[cfg(feature = "clipboard")]
            Self::Os(ctx) => {
                if let Ok(mut ctx) = ctx.try_borrow_mut() {
                    if let Ok(contents) = ctx.get_text() {
                        let mut lines = contents
                            .split('\n')
                            .map(|s| s.strip_suffix('\r').unwrap_or(s).to_string())
                            .collect::<Vec<_>>();
                        match lines.len() {
                            0 => {}
                            1 => return ClipboardContent::Piece(lines.remove(0).into()),
                            _ => return ClipboardContent::Chunk(lines.into()),
                        }
                    }
                }
                ClipboardContent::Piece(String::new().into())
            }
        }
    }
}

impl fmt::Debug for Clipboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Piece(p) => write!(f, "Clipboard({:?})", p),
            Self::Chunk(c) => write!(f, "Clipboard({:?})", c),
            #[cfg(feature = "clipboard")]
            Self::Os(_) => write!(f, "Clipboard(OS)"),
        }
    }
}

// Note: `Clone` is necessary for `TextArea` to derive `Clone`
impl Clone for Clipboard {
    fn clone(&self) -> Self {
        match self {
            Self::Piece(p) => Self::Piece(p.clone()),
            Self::Chunk(c) => Self::Chunk(c.clone()),
            #[cfg(feature = "clipboard")]
            Self::Os(_) => {
                if let Ok(ctx) = Arboard::new() {
                    Self::Os(RefCell::new(ctx))
                } else {
                    Self::Piece(String::new())
                }
            }
        }
    }
}

#[allow(clippy::let_unit_value)]
#[cfg(test)]
mod tests {
    use super::*;

    // Aquiring the lock is necessary to make all tests in serial. When `clipboard` feature is enabled, these tests
    // access the global OS clipboard. Running tests in parallel causes races.
    #[cfg(feature = "clipboard")]
    fn guard() -> impl Drop {
        use std::sync::Mutex;
        static M: Mutex<()> = Mutex::new(());
        M.lock().unwrap()
    }
    #[cfg(not(feature = "clipboard"))]
    fn guard() {}

    #[cfg(not(feature = "clipboard"))]
    #[test]
    fn default_value() {
        let _guard = guard();
        let c = Clipboard::default();
        assert_eq!(String::from(c.contents()), "");
    }

    #[cfg(feature = "clipboard")]
    #[test]
    fn default_value() {
        let _guard = guard();
        let mut arboard = Arboard::new().unwrap();
        arboard.set_text("Hello, world").unwrap();
        let c = Clipboard::default();
        assert_eq!(String::from(c.contents()), "Hello, world");
    }

    #[test]
    fn set_get_piece() {
        let _guard = guard();
        let tests = ["", "abc", "あいうえお"];
        for test in tests {
            let mut c = Clipboard::default();
            c.set_piece(test.to_string());
            assert_eq!(String::from(c.contents()), test, "{test:?}");
        }
    }

    #[test]
    fn set_get_chunk() {
        let _guard = guard();
        let tests = [
            ("", ""),
            ("\n", "\n"),
            ("\n\n", "\n\n"),
            ("a\n", "a\n"),
            ("a\nb", "a\nb"),
            ("a\nb\nc", "a\nb\nc"),
            ("あ\nい\nう", "あ\nい\nう"),
            ("\r\n", "\n"),
            ("a\r\nb\r\nc", "a\nb\nc"),
        ];
        for test in tests {
            let (before, after) = test;
            let mut c = Clipboard::default();
            let before = before
                .replace('\r', "")
                .split('\n')
                .map(|s| s.to_string())
                .collect();
            c.set_chunk(before);
            assert_eq!(String::from(c.contents()), after, "{test:?}");
        }
    }

    #[test]
    fn clone_and_debug() {
        let _guard = guard();
        let mut c1 = Clipboard::default();

        let c2 = c1.clone();
        assert_eq!(format!("{c1:?}"), format!("{c2:?}"));

        c1.set_chunk(vec!["a".to_string(), "b".to_string()]);
        let c2 = c1.clone();
        assert_eq!(format!("{c1:?}"), format!("{c2:?}"));
    }
}
