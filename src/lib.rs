//! A generalized version of format_args, that allows you to format into arbitrary types, instead of
//! just strings.
//!
//! For examples, see the examples subdirectory. `color.rs` provides a simple motivation.
//!
//! To get started, implement [`CustomFormatter`]. For types with a single canonical way to format,
//! implement [`CustomFormatter`] directly on the type with `Output = Self`. For types with multiple
//! different formatting strategies (such as Debug and Display for strings), implement
//! [`CustomFormatter`] on a marker type instead.
//!
//! Then, implement `Format<F>` on types that can be formatted into your custom formatter.
//!
//! Finally, you can use the provided macro, [`custom_format!`], in a similar manner to `format`, to
//! create instances of the type.

use bevy_ptr::Ptr;

/// A custom formatting strategy.
pub trait CustomFormatter: Sized {
    /// The type this formatting strategy produces. If this is Self, the trait implementation
    /// describes the canonical formatting strategy.
    type Output;
    /// The type this formatting strategy produces.
    type Error;
    /// Create a Self from the given [`Arguments`]. Generally, the implementation will look
    /// something like the following:
    /// ```rust,ignore
    /// fn from_args(args: Arguments<'_, Self>) -> Result<Self::Output, Self::Error> {
    /// let mut self_ = Self::new();
    ///
    /// for (piece, arg) in args {
    ///     self_.push_str(piece);
    ///     if let Some(arg) = arg {
    ///         arg.fmt(&mut self_);
    ///     }
    /// }
    ///
    /// Ok(self_.into_output())
    /// }
    /// ```
    /// Obviously, specifics may differ.
    fn from_args(args: Arguments<'_, Self>) -> Result<Self::Output, Self::Error>;
}

pub trait Format<F: CustomFormatter> {
    /// Format into the given formatter. This should use associated methods on the formatter.
    fn fmt(&self, f: &mut F) -> Result<(), F::Error>;
    /// A size hint. Exactly what this size refers to is up to the custom formatter, although the
    /// formatter may not rely on the implementation being correct.
    fn estimated_capacity(&self) -> usize {
        0
    }
}

/// A collection of format arguments. Implements `Iterator<Item = (&'static str, Option<Argument<'a,
/// F>>)`. Construct this using [`custom_format_args`].
pub struct Arguments<'a, F: CustomFormatter> {
    pieces: &'a [&'static str],
    args: &'a [Argument<'a, F>],
}

impl<'a, F: CustomFormatter> Iterator for Arguments<'a, F> {
    type Item = (&'static str, Option<Argument<'a, F>>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((first_piece, rest)) = self.pieces.split_first() {
            self.pieces = rest;
            if let Some((first_arg, rest)) = self.args.split_first() {
                self.args = rest;
                Some((*first_piece, Some(*first_arg)))
            } else {
                Some((*first_piece, None))
            }
        } else {
            None
        }
    }
}

impl<'a, F: CustomFormatter> Arguments<'a, F> {
    /// Create a new arguments from the given slices of strings and arguments. `pieces` and `args`
    /// will end up interleaved. `pieces` should be either the same length as `args`, or one longer.
    pub fn new(pieces: &'a [&'static str], args: &'a [Argument<'_, F>]) -> Self {
        Self { pieces, args }
    }
}

impl<F: CustomFormatter> Arguments<'_, F> {
    /// Access the static string slices
    pub fn pieces(&self) -> &[&'static str] {
        self.pieces
    }
    /// Access the format arguments
    pub fn args(&self) -> &[Argument<'_, F>] {
        self.args
    }
    /// Get the estimated size of the format arguments, not including the static string size.
    pub fn estimated_total_capacity(&self) -> usize {
        self.args.iter().map(|arg| arg.estimated_capacity).sum()
    }
}

/// Represents a single formatting argument.
pub struct Argument<'a, F: CustomFormatter> {
    ptr: Ptr<'a>,
    // INVARIANT: this has to be a transmuted Format::fmt function pointer, and ptr has to be a
    // pointer to the type it is from.
    formatter: unsafe fn(Ptr<'_>, &mut F) -> Result<(), F::Error>,
    estimated_capacity: usize,
}

impl<F: CustomFormatter> Clone for Argument<'_, F> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            formatter: self.formatter,
            estimated_capacity: self.estimated_capacity,
        }
    }
}

impl<F: CustomFormatter> Copy for Argument<'_, F> {}

impl<F: CustomFormatter> Format<F> for Argument<'_, F> {
    fn fmt(&self, f: &mut F) -> Result<(), <F as CustomFormatter>::Error> {
        // Safety: if the invariant is upheld, this is safe
        unsafe { (self.formatter)(self.ptr, f) }
    }
}

impl<'a, F: CustomFormatter> Argument<'a, F> {
    /// Create an [`Argument`] from a reference to a type that implements [`Format`].
    pub fn from_ref<T: Format<F>>(value: &'a T) -> Self {
        value.into()
    }
}

impl<'a, F: CustomFormatter, T: Format<F>> From<&'a T> for Argument<'a, F> {
    fn from(value: &'a T) -> Self {
        Self {
            ptr: value.into(),
            // Safety: layouts are the same
            formatter: unsafe {
                std::mem::transmute(T::fmt as fn(&T, &mut F) -> Result<(), F::Error>)
            },
            estimated_capacity: value.estimated_capacity(),
        }
    }
}

// #[doc(hidden)]
pub use custom_formatter_macro::custom_format_args as __custom_format_args_internal;

/// Create an [`Arguments`] from a formatting string.
///
/// Note: formatting specifiers are not supported.
#[macro_export]
macro_rules! custom_format_args {
    ($($args:tt)*) => {
        $crate::__custom_format_args_internal!(in $crate, $($args)*)
    }
}

/// Create any type that is the output of a [`CustomFormatter`], from a formatting string.
///
/// If the type you are formatting into has a [`CustomFormatter`] implementation with `Output =
/// Self`, then the type can probably be inferred:
/// ```rust,ignore
/// let res: Vec<u8> = custom_format!("hello world");
/// ```
///
/// If not, or if you want to specify the type within the macro, use `with`:
/// ```rust,ignore
/// let res = custom_format!(with Vec<u8>, "hello world");
/// ```
///
/// Note: formatting specifiers are not supported.
///
/// # Panics
/// Panics if the formatter encounters an error.
#[macro_export]
macro_rules! custom_format {
    (with $ty:ty, $($args:tt)*) => {
        $crate::custom_format::<$ty>($crate::custom_format_args!($($args)*))
    };
    ($($args:tt)*) => {
        $crate::custom_format_infer($crate::custom_format_args!($($args)*))
    };
}

/// Format into a type, using the given formatting strategy.
///
/// # Panics
/// Panics if the formatter encounters an error.
pub fn custom_format<F: CustomFormatter>(args: Arguments<'_, F>) -> F::Output {
    if let Ok(inner) = F::from_args(args) {
        inner
    } else {
        panic!("the formatter returned an error")
    }
}

/// Format into a T, possibly inferring type from surroundings.
///
/// # Panics
/// Panics if the formatter encounters an error.
pub fn custom_format_infer<T>(args: Arguments<'_, T>) -> T
where
    T: CustomFormatter<Output = T>,
{
    if let Ok(inner) = T::from_args(args) {
        inner
    } else {
        panic!("the formatter returned an error")
    }
}

/// An example formatter. Can format anything with a Debug implementation. Formats into a String.
pub struct DebugFormatter(String);
/// An example formatter. Can format anything with a Display implementation. Formats into a String.
pub struct DisplayFormatter(String);

mod impls;
