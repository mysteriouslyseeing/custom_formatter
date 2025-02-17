use super::*;
use std::{fmt::Write as FmtWrite, io::Write as IoWrite};

impl CustomFormatter for Vec<u8> {
    type Output = Self;
    type Error = std::io::Error;
    fn from_args(args: Arguments<'_, Self>) -> Result<Self, std::io::Error> {
        let mut self_ = Vec::with_capacity(args.estimated_total_capacity());

        for (piece, arg) in args {
            self_.write(piece.as_bytes())?;
            if let Some(arg) = arg {
                arg.fmt(&mut self_)?;
            }
        }

        Ok(self_)
    }
}
impl Format<Vec<u8>> for &str {
    fn fmt(&self, f: &mut Vec<u8>) -> Result<(), std::io::Error> {
        f.write(self.as_bytes()).map(|_| ())
    }
    fn estimated_capacity(&self) -> usize {
        self.len()
    }
}
impl Format<Vec<u8>> for u8 {
    fn fmt(&self, f: &mut Vec<u8>) -> Result<(), std::io::Error> {
        Ok(f.push(*self))
    }
    fn estimated_capacity(&self) -> usize {
        1
    }
}
impl Format<Vec<u8>> for &[u8] {
    fn fmt(&self, f: &mut Vec<u8>) -> Result<(), <Vec<u8> as CustomFormatter>::Error> {
        f.extend_from_slice(self);

        Ok(())
    }
    fn estimated_capacity(&self) -> usize {
        self.len()
    }
}
impl<T: Format<Vec<u8>>> Format<Vec<u8>> for &T {
    fn fmt(&self, f: &mut Vec<u8>) -> Result<(), <Vec<u8> as CustomFormatter>::Error> {
        T::fmt(self, f)
    }
}

impl CustomFormatter for DebugFormatter {
    type Output = String;
    type Error = std::fmt::Error;
    fn from_args(args: Arguments<'_, Self>) -> Result<Self::Output, Self::Error> {
        let mut self_ = Self(String::with_capacity(args.estimated_total_capacity()));

        for (piece, arg) in args {
            self_.0.write_str(piece)?;
            if let Some(arg) = arg {
                arg.fmt(&mut self_)?;
            }
        }

        Ok(self_.0)
    }
}

impl<T> Format<DebugFormatter> for T
where
    T: core::fmt::Debug,
{
    fn fmt(
        &self,
        f: &mut DebugFormatter,
    ) -> Result<(), <DebugFormatter as CustomFormatter>::Error> {
        f.0.write_fmt(format_args!("{self:?}"))
    }
}

impl CustomFormatter for DisplayFormatter {
    type Output = String;
    type Error = std::fmt::Error;
    fn from_args(args: Arguments<'_, Self>) -> Result<Self::Output, Self::Error> {
        let mut self_ = Self(String::with_capacity(args.estimated_total_capacity()));

        for (piece, arg) in args {
            self_.0.write_str(piece)?;
            if let Some(arg) = arg {
                arg.fmt(&mut self_)?;
            }
        }

        Ok(self_.0)
    }
}

impl<T> Format<DisplayFormatter> for T
where
    T: core::fmt::Display,
{
    fn fmt(
        &self,
        f: &mut DisplayFormatter,
    ) -> Result<(), <DebugFormatter as CustomFormatter>::Error> {
        f.0.write_fmt(format_args!("{self}"))
    }
}
