use rustyline::config::Configurer;
use rustyline::highlight::Highlighter;
use rustyline::{ColorMode, Editor, Result};
use rustyline::{Completer, Helper, Hinter, Validator};

#[derive(Completer, Helper, Hinter, Validator)]
struct MaskingHighlighter {
    masking: bool,
}

impl Highlighter for MaskingHighlighter {
    fn highlight<'b, 's: 'b, 'l: 'b>(
        &'s mut self,
        line: &'l str,
        pos: usize,
    ) -> std::borrow::Cow<'b, str> {
        use unicode_width::UnicodeWidthStr;
        if self.masking {
            std::borrow::Cow::Owned(" ".repeat(line.width()))
        } else {
            std::borrow::Cow::Borrowed(line)
        }
    }

    fn highlight_char(&mut self, _line: &str, _pos: usize, _forced: bool) -> bool {
        self.masking
    }
}

fn main() -> Result<()> {
    println!("This is just a hack. Reading passwords securely requires more than that.");
    let h = MaskingHighlighter { masking: false };
    let mut rl = Editor::new(h)?;

    let username = rl.readline("Username:")?;
    println!("Username: {username}");

    rl.helper_mut().masking = true;
    rl.set_color_mode(ColorMode::Forced); // force masking
    rl.set_auto_add_history(false); // make sure password is not added to history
    let mut guard = rl.set_cursor_visibility(false)?;
    let passwd = rl.readline("Password:")?;
    guard.take();
    println!("Secret: {passwd}");
    Ok(())
}
