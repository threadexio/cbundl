use std::fmt::{self, Display};

use owo_colors::OwoColorize;

pub mod style {
    use super::*;

    pub type StyleFn = fn(&dyn Display, f: &mut fmt::Formatter) -> fmt::Result;

    pub fn path(x: &dyn Display, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "“{}”", x.bright_magenta())
    }
}

#[macro_export]
macro_rules! display {
    ($x:expr) => {{
        $crate::display::_display_type(&$x)
    }};
    ($x:expr; as $style:ident) => {{
        $crate::display::_display_styled(&$x, $crate::display::style::$style)
    }};
}

#[allow(non_camel_case_types)]
pub struct _display_type<'a, T>(pub &'a T)
where
    T: ?Sized;

impl<'a, 'b, T> Display for _display_type<'a, &'b T>
where
    _display_type<'b, T>: Display,
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        _display_type(*self.0).fmt(f)
    }
}

impl Display for _display_type<'_, std::path::Path> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        style::path(&self.0.display(), f)
    }
}

impl Display for _display_type<'_, std::path::PathBuf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        style::path(&self.0.display(), f)
    }
}

#[allow(non_camel_case_types)]
pub struct _display_styled<'a, T>(pub &'a T, pub style::StyleFn)
where
    T: ?Sized;

impl<'a, T> Display for _display_styled<'a, T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.1)(&self.0, f)
    }
}
