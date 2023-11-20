#[cfg(feature = "clipboard")]
use copypasta::{ClipboardContext, ClipboardProvider as _};
use std::borrow::Cow;
#[cfg(feature = "clipboard")]
use std::cell::RefCell;
use std::fmt;

pub enum ClipboardContent<'a> {
    Piece(Cow<'a, str>),
    Chunk(Cow<'a, [String]>),
}

impl<'a> Default for ClipboardContent<'a> {
    fn default() -> Self {
        Self::Piece(String::new().into())
    }
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
    Os(RefCell<ClipboardContext>), // Use `RefCell` not to make `Clipboard::contents` mut method
}

impl Default for Clipboard {
    fn default() -> Self {
        #[cfg(feature = "clipboard")]
        if let Ok(ctx) = ClipboardContext::new() {
            return Self::Os(RefCell::new(ctx));
        }
        Self::Piece(String::new())
    }
}

impl Clipboard {
    pub fn set_piece(&mut self, s: String) {
        #[cfg(feature = "clipboard")]
        if let Self::Os(ctx) = self {
            if let Ok(mut ctx) = ctx.try_borrow_mut() {
                let _ = ctx.set_contents(s);
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
                        let _ = ctx.set_contents(c.join("\n"));
                        return;
                    }
                }
                *self = Self::Chunk(c);
            }
        }
    }

    pub fn content(&self) -> ClipboardContent<'_> {
        match self {
            Self::Piece(p) => ClipboardContent::Piece(p.as_str().into()),
            Self::Chunk(c) => ClipboardContent::Chunk(c.as_slice().into()),
            #[cfg(feature = "clipboard")]
            Self::Os(ctx) => {
                if let Ok(mut ctx) = ctx.try_borrow_mut() {
                    if let Ok(contents) = ctx.get_contents() {
                        let mut lines = contents
                            .split('\n')
                            .map(|s| s.strip_suffix('\r').unwrap_or(s).to_string())
                            .collect::<Vec<_>>();
                        match lines.len() {
                            0 => {}
                            1 => return ClipboardContent::Piece(Cow::Owned(lines.remove(0))),
                            _ => return ClipboardContent::Chunk(Cow::Owned(lines)),
                        }
                    }
                }
                ClipboardContent::default()
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

impl Clone for Clipboard {
    fn clone(&self) -> Self {
        match self {
            Self::Piece(p) => Self::Piece(p.clone()),
            Self::Chunk(c) => Self::Chunk(c.clone()),
            #[cfg(feature = "clipboard")]
            Self::Os(_) => {
                if let Ok(ctx) = ClipboardContext::new() {
                    Self::Os(RefCell::new(ctx))
                } else {
                    Self::Piece(String::new())
                }
            }
        }
    }
}
