use crate::context::PropertyHandlerContext;
use crate::declaration::DeclarationList;
use crate::error::{ParserError, PrinterError};
use crate::printer::Printer;
use crate::properties::Property;
use crate::stylesheet::PrinterOptions;
use crate::targets::Browsers;
use cssparser::*;

/// Trait for things that can be parsed from CSS syntax.
pub trait Parse<'i>: Sized {
  /// Parse a value of this type using an existing parser.
  fn parse<'t>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i, ParserError<'i>>>;

  /// Parse a value from a string.
  ///
  /// (This is a convenience wrapper for `parse` and probably should not be overridden.)
  fn parse_string(input: &'i str) -> Result<Self, ParseError<'i, ParserError<'i>>> {
    let mut input = ParserInput::new(input);
    let mut parser = Parser::new(&mut input);
    let result = Self::parse(&mut parser)?;
    parser.expect_exhausted()?;
    Ok(result)
  }
}

/// Trait for things the can serialize themselves in CSS syntax.
pub trait ToCss {
  /// Serialize `self` in CSS syntax, writing to `dest`.
  fn to_css<W>(&self, dest: &mut Printer<W>) -> Result<(), PrinterError>
  where
    W: std::fmt::Write;

  /// Serialize `self` in CSS syntax and return a string.
  ///
  /// (This is a convenience wrapper for `to_css` and probably should not be overridden.)
  #[inline]
  fn to_css_string(&self, options: PrinterOptions) -> Result<String, PrinterError> {
    let mut s = String::new();
    let mut printer = Printer::new(&mut s, options);
    self.to_css(&mut printer)?;
    Ok(s)
  }
}

impl<'a, T> ToCss for &'a T
where
  T: ToCss + ?Sized,
{
  fn to_css<W>(&self, dest: &mut Printer<W>) -> Result<(), PrinterError>
  where
    W: std::fmt::Write,
  {
    (*self).to_css(dest)
  }
}

pub(crate) trait PropertyHandler<'i>: Sized {
  fn handle_property(
    &mut self,
    property: &Property<'i>,
    dest: &mut DeclarationList<'i>,
    context: &mut PropertyHandlerContext<'i>,
  ) -> bool;
  fn finalize(&mut self, dest: &mut DeclarationList<'i>, context: &mut PropertyHandlerContext<'i>);
}

pub(crate) mod private {
  pub trait TryAdd<T> {
    fn try_add(&self, other: &T) -> Option<T>;
  }
}

pub(crate) trait FromStandard<T>: Sized {
  fn from_standard(val: &T) -> Option<Self>;
}

pub(crate) trait FallbackValues: Sized {
  fn get_fallbacks(&mut self, targets: Browsers) -> Vec<Self>;
}
