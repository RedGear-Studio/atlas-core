use core::fmt;

/// Represents a position in bytes within a source file.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Default)]
pub struct BytePos(usize);

impl BytePos {
    /// Shifts the byte position by the length of a character.
    #[inline(always)]
    pub fn shift(self, ch: char) -> Self {
        BytePos(self.0 + ch.len_utf8())
    }

    /// Shifts the byte position by a specified number of bytes.
    #[inline(always)]
    pub fn shift_by(self, n: usize) -> Self {
        BytePos(self.0 + n)
    }
}

impl fmt::Display for BytePos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for BytePos {
    fn from(value: usize) -> Self {
        BytePos(value)
    }
}

impl From<BytePos> for usize {
    fn from(value: BytePos) -> Self {
        value.0
    }
}

/// Represents a span in a source file, defined by a start and end byte position.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Default)]
pub struct Span {
    /// The position of character at the start of the span
    pub start: BytePos,
    /// The position of character at the end of the span
    pub end: BytePos,
    /// The source file path
    pub path: &'static str,
}

impl Span {
    /// Creates a new `Span` without bounds checking.
    /// # Safety
    /// It's the caller's responsibility to ensure that `start` and `end` are valid
    pub unsafe fn new_unchecked(start: usize, end: usize, path: &'static str) -> Self {
        Span {
            start: BytePos(start),
            end: BytePos(end),
            path,
        }
    }

    /// Creates an empty `Span` with both start and end positions at zero.
    #[inline]
    pub const fn empty() -> Self {
        Span {
            start: BytePos(0),
            end: BytePos(0),
            path: "",
        }
    }

    /// Combines two spans to create a new span that encompasses both.
    pub fn union_span(self, other: Self) -> Self {
        use std::cmp;
        if self.path != other.path {
            panic!(
                "Cannot union spans from different files: {} and {}",
                self.path, other.path
            );
        }
        Span {
            start: cmp::min(self.start, other.start),
            end: cmp::max(self.end, other.end),
            path: self.path,
        }
    }

    /// Retrieves line information associated with the span.
    pub fn get_line_info(&self) -> LineInformation {
        let start_byte = self.start.0;
        let end_byte = self.end.0;
        let content = std::fs::read_to_string(self.path).expect("Unable to read file");
        // Find the start and end of the line containing the span's start position
        let line_start = content[..start_byte]
            .rfind('\n')
            .map(|idx| idx + 1)
            .unwrap_or(0);
        let line_end = content[end_byte..]
            .find('\n')
            .map(|idx| end_byte + idx)
            .unwrap_or(end_byte);

        let line_text = content[line_start..=(line_end - 1)].to_owned();
        let line_number = content[..start_byte].chars().filter(|&c| c == '\n').count() + 1;
        let column_number = start_byte - line_start + 1;

        LineInformation::new(line_number, column_number, line_text)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}:{} in \"{}\"]", self.start, self.end, self.path)
    }
}

/// WARNING: LineInformation need a huge rework as it's not working well at all
pub struct LineInformation {
    pub line_number: usize,
    pub column_number: usize,
    pub line_text: String,
}

impl LineInformation {
    pub fn new(line_number: usize, column_number: usize, line_text: String) -> Self {
        LineInformation {
            line_number,
            column_number,
            line_text,
        }
    }
}

impl fmt::Display for LineInformation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:\n {}",
            self.line_number, self.column_number, self.line_text
        )
    }
}

/// A trait representing an object that has an associated span in the source code.
///
/// This trait is used to obtain the span (range of positions) for a token or other
/// syntactic element. It provides methods to get the start and end positions
/// of the span.
///
/// # Required Methods
///
/// - `span(&self) -> Span`: Returns the `Span` representing the start and end positions of the object.
///
/// # Provided Methods
///
/// - `start(&self) -> usize`: Returns the starting position of the span.
///   This method is implemented using the `span` method and returns the start position.
/// - `end(&self) -> usize`: Returns the ending position of the span.
///   This method is implemented using the `span` method and returns the end position.
///
/// # Example
///
/// ```
/// use atlas_core::utils::span::{Spanned, Span, BytePos};
///
/// struct Token {
///     span: Span,
/// }
///
/// # impl Spanned for Token {
/// #     fn span(&self) -> Span {
/// #         self.span
/// #     }
/// # }
/// # unsafe {
/// //NB: "new_unchecked(usize, usize, char)" is an unsafe function, it's only used here as an example
/// let token = Token { span: Span::new_unchecked(5, 10, "[")};
/// assert_eq!(token.start(), 5);
/// assert_eq!(token.end(), 10);
/// # }
/// ```
pub trait Spanned {
    /// Returns the `Span` representing the start and end positions of the object.
    fn span(&self) -> Span;
    /// Returns the starting position of the span.
    fn start(&self) -> usize {
        self.span().start.0
    }
    /// Returns the end position of the span.
    fn end(&self) -> usize {
        self.span().end.0
    }
}

impl Spanned for Span {
    #[inline(always)]
    fn span(&self) -> Span {
        *self
    }
}
