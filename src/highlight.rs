//! Syntax highlighting

use crate::config::CompletionType;
use core::fmt::Display;
use std::borrow::Cow::{self, Borrowed};
use std::cell::Cell;
use std::marker::PhantomData;

/// ANSI style
pub trait Style {
    /// Produce a ansi sequences which sets the graphic mode
    fn start(&self) -> impl Display;
    /// Produce a ansi sequences which ends the graphic mode
    fn end(&self) -> impl Display;
}

/// The general trait that consume self to display
///
/// For **normal** highlight, all types that impl [`core::fmt::Display`] will auto-impl [`DisplayOnce`]
///
/// For **split-highlight**, you can use `impl Iterator<Item = 'l + StyledBlock>`
/// to get a [`StyledBlocks`], which is also impl [`DisplayOnce`]:
///
/// ```
/// use rustyline::highlight::{DisplayOnce, StyledBlock, StyledBlocks};
/// use anstyle::{Ansi256Color, Style};
/// struct Helper;
/// fn highlight<'b, 's: 'b, 'l: 'b>(
///     helper: &'s mut Helper,
///     line: &'l str,
/// ) -> impl 'b + DisplayOnce {
///     fn get_style(i: usize) -> Style {
///         Style::new().fg_color(Some(Ansi256Color((i % 16) as u8).into()))
///     }
///     let iter = (0..line.len()).map(move |i| (get_style(i), &line[i..i + 1]));
///     StyledBlocks::new(iter)
/// }
/// let mut helper = Helper;
/// highlight(&mut helper, "hello world\n").print();
/// ```
pub trait DisplayOnce {
    /// consume self to display
    fn fmt<W: core::fmt::Write>(self, f: &mut W) -> core::fmt::Result;
    /// consume self to print
    fn print(self) -> core::fmt::Result
    where
        Self: Sized,
    {
        struct StdoutWriter;
        impl core::fmt::Write for StdoutWriter {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                use std::io::Write;
                std::io::stdout()
                    .write_all(s.as_bytes())
                    .map_err(|_| core::fmt::Error)
            }
        }
        let mut stdout = StdoutWriter;
        Self::fmt(self, &mut stdout)
    }
}

impl<'l, T: Display> DisplayOnce for T {
    fn fmt<W: core::fmt::Write>(self, f: &mut W) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

/// A wrapper of `impl Iterator<Item = 'l + StyledBlock>`
/// that impl [`DisplayOnce`]:
///
/// ```
/// use rustyline::highlight::{DisplayOnce, StyledBlock, StyledBlocks};
/// use anstyle::{Ansi256Color, Style};
/// struct Helper;
/// fn highlight<'b, 's: 'b, 'l: 'b>(
///     helper: &'s mut Helper,
///     line: &'l str,
/// ) -> impl 'b + DisplayOnce {
///     fn get_style(i: usize) -> Style {
///         Style::new().fg_color(Some(Ansi256Color((i % 16) as u8).into()))
///     }
///     let iter = (0..line.len()).map(move |i| (get_style(i), &line[i..i + 1]));
///     StyledBlocks::new(iter)
/// }
/// let mut helper = Helper;
/// highlight(&mut helper, "hello world\n").print();
/// ```
pub struct StyledBlocks<'l, B, I>
where
    B: 'l + StyledBlock,
    I: Iterator<Item = B>,
{
    iter: I,
    _marker: PhantomData<&'l ()>,
}

impl<'l, B, I> StyledBlocks<'l, B, I>
where
    B: 'l + StyledBlock,
    I: Iterator<Item = B>,
{
    /// create a new [`StyledBlocks`] wrapper
    pub const fn new(iter: I) -> Self {
        Self {
            iter,
            _marker: PhantomData,
        }
    }
}

impl<'l, B, I> DisplayOnce for StyledBlocks<'l, B, I>
where
    B: 'l + StyledBlock,
    I: Iterator<Item = B>,
{
    fn fmt<W: core::fmt::Write>(self, f: &mut W) -> core::fmt::Result {
        self.iter
            .map(|block| {
                let style = block.style();
                write!(f, "{}{}{}", style.start(), block.text(), style.end())
            })
            .collect()
    }
}

impl Style for () {
    fn start(&self) -> impl Display {
        ""
    }

    fn end(&self) -> impl Display {
        ""
    }
}

/*#[cfg(feature = "ansi-str")]
#[cfg_attr(docsrs, doc(cfg(feature = "ansi-str")))]
impl Style for ansi_str::Style {
    fn start(&self) -> impl Display {
        self.start()
    }

    fn end(&self) -> impl Display {
        self.end()
    }
}*/
// #[cfg(feature = "anstyle")]
// #[cfg_attr(docsrs, doc(cfg(feature = "anstyle")))]
impl Style for anstyle::Style {
    fn start(&self) -> impl Display {
        self.render()
    }

    fn end(&self) -> impl Display {
        self.render_reset()
    }
}

/// Styled text
pub trait StyledBlock {
    /// Style impl
    type Style: Style
    where
        Self: Sized;
    /// Raw text to be styled
    fn text(&self) -> &str;
    /// `Style` to be applied on `text`
    fn style(&self) -> &Self::Style
    where
        Self: Sized;
}
/*#[cfg(feature = "ansi-str")]
#[cfg_attr(docsrs, doc(cfg(feature = "ansi-str")))]
impl StyledBlock for ansi_str::AnsiBlock<'_> {
    type Style = ansi_str::Style;

    fn text(&self) -> &str {
        self.text()
    }

    fn style(&self) -> &Self::Style {
        self.style()
    }
}*/

impl<S: Style, T: AsRef<str>> StyledBlock for (S, T) {
    type Style = S;

    fn text(&self) -> &str {
        self.1.as_ref()
    }

    fn style(&self) -> &Self::Style {
        &self.0
    }
}

/// Syntax highlighter with [ANSI color](https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_(Select_Graphic_Rendition)_parameters).
///
/// Currently, the highlighted version *must* have the same display width as
/// the original input.
pub trait Highlighter {
    /// Takes the currently edited `line` with the cursor `pos`ition and
    /// returns the highlighted version (with ANSI color).
    ///
    /// For example, you can implement
    /// [blink-matching-paren](https://www.gnu.org/software/bash/manual/html_node/Readline-Init-File-Syntax.html).
    fn highlight<'b, 's: 'b, 'l: 'b>(
        &'s mut self,
        line: &'l str,
        pos: usize,
    ) -> impl 'b + DisplayOnce {
        let _ = pos;
        line
    }

    /// Takes the `prompt` and
    /// returns the highlighted version (with ANSI color).
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s mut self,
        prompt: &'p str,
        default: bool,
    ) -> impl 'b + DisplayOnce {
        let _ = default;
        prompt
    }
    /// Takes the `hint` and
    /// returns the highlighted version (with ANSI color).
    fn highlight_hint<'b, 's: 'b, 'h: 'b>(&'s mut self, hint: &'h str) -> impl 'b + DisplayOnce {
        hint
    }
    /// Takes the completion `candidate` and
    /// returns the highlighted version (with ANSI color).
    ///
    /// Currently, used only with `CompletionType::List`.
    fn highlight_candidate<'b, 's: 'b, 'c: 'b>(
        &'s mut self,
        candidate: &'c str, // FIXME should be Completer::Candidate
        completion: CompletionType,
    ) -> impl 'b + DisplayOnce {
        let _ = completion;
        candidate
    }
    /// Tells if `line` needs to be highlighted when a specific char is typed or
    /// when cursor is moved under a specific char.
    /// `forced` flag is `true` mainly when user presses Enter (i.e. transient
    /// vs final highlight).
    ///
    /// Used to optimize refresh when a character is inserted or the cursor is
    /// moved.
    fn highlight_char(&mut self, line: &str, pos: usize, forced: bool) -> bool {
        let _ = (line, pos, forced);
        false
    }
}

impl Highlighter for () {}

impl<'r, H: Highlighter> Highlighter for &'r mut H {
    fn highlight<'b, 's: 'b, 'l: 'b>(
        &'s mut self,
        line: &'l str,
        pos: usize,
    ) -> impl 'b + DisplayOnce {
        (**self).highlight(line, pos)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s mut self,
        prompt: &'p str,
        default: bool,
    ) -> impl 'b + DisplayOnce {
        (**self).highlight_prompt(prompt, default)
    }

    fn highlight_hint<'b, 's: 'b, 'h: 'b>(&'s mut self, hint: &'h str) -> impl 'b + DisplayOnce {
        (**self).highlight_hint(hint)
    }

    fn highlight_candidate<'b, 's: 'b, 'c: 'b>(
        &'s mut self,
        candidate: &'c str,
        completion: CompletionType,
    ) -> impl 'b + DisplayOnce {
        (**self).highlight_candidate(candidate, completion)
    }

    fn highlight_char(&mut self, line: &str, pos: usize, forced: bool) -> bool {
        (**self).highlight_char(line, pos, forced)
    }
}

// TODO versus https://python-prompt-toolkit.readthedocs.io/en/master/pages/reference.html?highlight=HighlightMatchingBracketProcessor#prompt_toolkit.layout.processors.HighlightMatchingBracketProcessor

/// Highlight matching bracket when typed or cursor moved on.
#[derive(Default)]
pub struct MatchingBracketHighlighter {
    // #[cfg(feature = "anstyle")]
    style: anstyle::Style,
    bracket: Cell<Option<(u8, usize)>>, // memorize the character to search...
}

impl MatchingBracketHighlighter {
    /// Constructor
    #[must_use]
    pub fn new() -> Self {
        Self {
            // #[cfg(feature = "anstyle")]
            style: anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::AnsiColor::Blue.into())),
            bracket: Cell::new(None),
        }
    }
}

impl Highlighter for MatchingBracketHighlighter {
    fn highlight<'b, 's: 'b, 'l: 'b>(
        &'s mut self,
        line: &'l str,
        _pos: usize,
    ) -> impl 'b + DisplayOnce {
        if line.len() <= 1 {
            return Borrowed(line);
        }
        // highlight matching brace/bracket/parenthesis if it exists
        if let Some((bracket, pos)) = self.bracket.get() {
            if let Some((matching, idx)) = find_matching_bracket(line, pos, bracket) {
                let mut copy = line.to_owned();
                copy.replace_range(idx..=idx, &format!("\x1b[1;34m{}\x1b[0m", matching as char));
                return Cow::Owned(copy);
            }
        }
        Borrowed(line)
    }

    fn highlight_char(&mut self, line: &str, pos: usize, forced: bool) -> bool {
        if forced {
            self.bracket.set(None);
            return false;
        }
        // will highlight matching brace/bracket/parenthesis if it exists
        self.bracket.set(check_bracket(line, pos));
        self.bracket.get().is_some()
    }
}

fn find_matching_bracket(line: &str, pos: usize, bracket: u8) -> Option<(u8, usize)> {
    let matching = matching_bracket(bracket);
    let mut idx;
    let mut unmatched = 1;
    if is_open_bracket(bracket) {
        // forward search
        idx = pos + 1;
        let bytes = &line.as_bytes()[idx..];
        for b in bytes {
            if *b == matching {
                unmatched -= 1;
                if unmatched == 0 {
                    debug_assert_eq!(matching, line.as_bytes()[idx]);
                    return Some((matching, idx));
                }
            } else if *b == bracket {
                unmatched += 1;
            }
            idx += 1;
        }
        debug_assert_eq!(idx, line.len());
    } else {
        // backward search
        idx = pos;
        let bytes = &line.as_bytes()[..idx];
        for b in bytes.iter().rev() {
            if *b == matching {
                unmatched -= 1;
                if unmatched == 0 {
                    debug_assert_eq!(matching, line.as_bytes()[idx - 1]);
                    return Some((matching, idx - 1));
                }
            } else if *b == bracket {
                unmatched += 1;
            }
            idx -= 1;
        }
        debug_assert_eq!(idx, 0);
    }
    None
}

// check under or before the cursor
fn check_bracket(line: &str, pos: usize) -> Option<(u8, usize)> {
    if line.is_empty() {
        return None;
    }
    let mut pos = pos;
    if pos >= line.len() {
        pos = line.len() - 1; // before cursor
        let b = line.as_bytes()[pos]; // previous byte
        if is_close_bracket(b) {
            Some((b, pos))
        } else {
            None
        }
    } else {
        let mut under_cursor = true;
        loop {
            let b = line.as_bytes()[pos];
            if is_close_bracket(b) {
                return if pos == 0 { None } else { Some((b, pos)) };
            } else if is_open_bracket(b) {
                return if pos + 1 == line.len() {
                    None
                } else {
                    Some((b, pos))
                };
            } else if under_cursor && pos > 0 {
                under_cursor = false;
                pos -= 1; // or before cursor
            } else {
                return None;
            }
        }
    }
}

const fn matching_bracket(bracket: u8) -> u8 {
    match bracket {
        b'{' => b'}',
        b'}' => b'{',
        b'[' => b']',
        b']' => b'[',
        b'(' => b')',
        b')' => b'(',
        b => b,
    }
}
const fn is_open_bracket(bracket: u8) -> bool {
    matches!(bracket, b'{' | b'[' | b'(')
}
const fn is_close_bracket(bracket: u8) -> bool {
    matches!(bracket, b'}' | b']' | b')')
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn find_matching_bracket() {
        use super::find_matching_bracket;
        assert_eq!(find_matching_bracket("(...", 0, b'('), None);
        assert_eq!(find_matching_bracket("...)", 3, b')'), None);

        assert_eq!(find_matching_bracket("()..", 0, b'('), Some((b')', 1)));
        assert_eq!(find_matching_bracket("(..)", 0, b'('), Some((b')', 3)));

        assert_eq!(find_matching_bracket("..()", 3, b')'), Some((b'(', 2)));
        assert_eq!(find_matching_bracket("(..)", 3, b')'), Some((b'(', 0)));

        assert_eq!(find_matching_bracket("(())", 0, b'('), Some((b')', 3)));
        assert_eq!(find_matching_bracket("(())", 3, b')'), Some((b'(', 0)));
    }
    #[test]
    pub fn check_bracket() {
        use super::check_bracket;
        assert_eq!(check_bracket(")...", 0), None);
        assert_eq!(check_bracket("(...", 2), None);
        assert_eq!(check_bracket("...(", 3), None);
        assert_eq!(check_bracket("...(", 4), None);
        assert_eq!(check_bracket("..).", 4), None);

        assert_eq!(check_bracket("(...", 0), Some((b'(', 0)));
        assert_eq!(check_bracket("(...", 1), Some((b'(', 0)));
        assert_eq!(check_bracket("...)", 3), Some((b')', 3)));
        assert_eq!(check_bracket("...)", 4), Some((b')', 3)));
    }
    #[test]
    pub fn matching_bracket() {
        use super::matching_bracket;
        assert_eq!(matching_bracket(b'('), b')');
        assert_eq!(matching_bracket(b')'), b'(');
    }

    #[test]
    pub fn is_open_bracket() {
        use super::is_close_bracket;
        use super::is_open_bracket;
        assert!(is_open_bracket(b'('));
        assert!(is_close_bracket(b')'));
    }

    #[test]
    #[cfg(feature = "ansi-str")]
    pub fn styled_text() {
        use ansi_str::get_blocks;

        let mut blocks = get_blocks("\x1b[1;32mHello \x1b[3mworld\x1b[23m!\x1b[0m");
        assert_eq!(blocks.next(), get_blocks("\x1b[1;32mHello ").next());
        assert_eq!(blocks.next(), get_blocks("\x1b[1;32m\x1b[3mworld").next());
        assert_eq!(blocks.next(), get_blocks("\x1b[1;32m!").next());
        assert!(blocks.next().is_none())
    }
}
