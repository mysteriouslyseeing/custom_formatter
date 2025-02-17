use std::fmt::{Display, Write};

use custom_formatter::{custom_format, CustomFormatter, Format};

struct ColoredString {
    fragments: Vec<ColoredFragment>,
}

impl ColoredString {
    fn push_fragment(&mut self, frag: ColoredFragment) {
        self.fragments.push(frag)
    }
}

#[derive(Clone)]
struct ColoredFragment {
    fragment: String,
    color: Color,
}

#[derive(Clone)]
enum Color {
    Red,
    Blue,
    Green,
    White,
}

impl From<&str> for ColoredFragment {
    fn from(value: &str) -> Self {
        ColoredFragment {
            fragment: value.into(),
            color: Color::White,
        }
    }
}

trait ColorExt: Into<ColoredFragment> {
    fn red(self) -> ColoredFragment {
        let mut frag = self.into();
        frag.color = Color::Red;
        frag
    }
    fn blue(self) -> ColoredFragment {
        let mut frag = self.into();
        frag.color = Color::Blue;
        frag
    }
    fn green(self) -> ColoredFragment {
        let mut frag = self.into();
        frag.color = Color::Green;
        frag
    }
    fn white(self) -> ColoredFragment {
        let mut frag = self.into();
        frag.color = Color::White;
        frag
    }
}

impl<T> ColorExt for T where T: Into<ColoredFragment> {}

impl CustomFormatter for ColoredString {
    type Output = Self;
    type Error = ();
    fn from_args(args: custom_formatter::Arguments<'_, Self>) -> Result<Self::Output, Self::Error> {
        let mut string = ColoredString {
            fragments: Vec::new(),
        };

        for (piece, arg) in args {
            string.push_fragment(piece.white());
            if let Some(arg) = arg {
                arg.fmt(&mut string)?;
            }
        }

        Ok(string)
    }
}

impl Format<ColoredString> for &str {
    fn fmt(&self, f: &mut ColoredString) -> Result<(), <ColoredString as CustomFormatter>::Error> {
        f.push_fragment((*self).into());

        Ok(())
    }
}

impl Format<ColoredString> for ColoredFragment {
    fn fmt(&self, f: &mut ColoredString) -> Result<(), <ColoredString as CustomFormatter>::Error> {
        f.push_fragment(self.clone());

        Ok(())
    }
}

// These implementations are just for printing to console so we know it worked
impl Display for ColoredFragment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write! {
            f,
            "{}{}",
            // ansi codes
            match self.color {
                Color::Red => "\x1b[31m",
                Color::Blue => "\x1b[34m",
                Color::Green => "\x1b[92m",
                Color::White => "\x1b[37m",
            },
            self.fragment
        }
    }
}
impl Display for ColoredString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for frag in &self.fragments {
            f.write_fmt(format_args!("{}\x1b[0m", frag))?;
        }

        Ok(())
    }
}

fn main() {
    let string: ColoredString = custom_format!(
        "Hello {} {} {} World",
        "Red".red(),
        "Blue".blue(),
        "Green".green()
    );
    println!("{}", string);
}
