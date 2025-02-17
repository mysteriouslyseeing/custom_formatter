use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, Expr, Ident, LitStr, Path, Token,
};

#[proc_macro]
pub fn custom_format_args(stream: TokenStream) -> TokenStream {
    let macro_args = parse_macro_input!(stream as FormatArgs);
    let custom_formatter = macro_args.custom_format_crate;

    let args: Vec<Expr> = macro_args
        .args
        .pairs()
        .filter_map(|pair| pair.punct().copied().cloned())
        .collect();

    let pieces: Vec<LitStr> = macro_args.format_str.args.iter().cloned().collect();

    let mut args_iter = args.iter();
    //TODO: this will actually reevalute arguments multiple times if they are specified multiple
    // times, unlike format!("{0}{0}", func()), which evaluates func() only once.
    let args_reordered: Vec<Expr> = match macro_args
        .format_str
        .args
        .pairs()
        .filter_map(|p| p.punct().copied().cloned())
        .map(|arg| match arg {
            FormatArgument::Positional => args_iter.next().cloned().ok_or_else(|| {
                syn::Error::new_spanned(
                    &macro_args.format_str.lit,
                    "format string missing positional argument",
                )
            }),
            FormatArgument::Numbered(n) => args.get(n).cloned().ok_or_else(|| {
                syn::Error::new_spanned(
                    &macro_args.format_str.lit,
                    "numbered argument out of range",
                )
            }),
            FormatArgument::Named(name) => Ok(syn::Expr::Verbatim(name.into_token_stream())),
        })
        .collect::<Result<Vec<Expr>, syn::Error>>()
    {
        Ok(a) => a,
        Err(e) => return e.into_compile_error().into(),
    };

    quote::quote! {
        #custom_formatter::Arguments::new(&[#(#pieces),*], &[#(#custom_formatter::Argument::from_ref(&#args_reordered)),*])
    }
    .into()
}

struct FormatArgs {
    _in: Token![in],
    custom_format_crate: Path,
    _comma: Token![,],
    format_str: FormatString,
    args: Punctuated<Token![,], Expr>,
}

impl Parse for FormatArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _in: input.parse()?,
            custom_format_crate: input.parse()?,
            _comma: input.parse()?,
            format_str: input.parse()?,
            args: input.parse_terminated(<Token![,]>::parse)?,
        })
    }
}

struct FormatString {
    lit: LitStr,
    args: Punctuated<LitStr, FormatArgument>,
}

#[derive(Clone)]
enum FormatArgument {
    Positional,
    Numbered(usize),
    Named(Ident),
}

impl Parse for FormatString {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit: LitStr = input.parse()?;

        let mut args = Punctuated::new();

        let value = lit.value();
        let mut char_iter = value.chars().peekable();
        let mut partial = String::with_capacity(value.len());

        while let Some(c) = char_iter.next() {
            match (c, char_iter.peek()) {
                ('{', Some('{')) => partial.push('{'),
                ('{', _) => {
                    args.push_value(LitStr::new(&partial, lit.span()));
                    partial.clear();

                    let mut argument_string = String::new();
                    loop {
                        match char_iter.next() {
                            Some('}') => break,
                            Some(c) => argument_string.push(c),
                            None => {
                                return Err(syn::Error::new_spanned(
                                    lit,
                                    "invalid format string: expected `}` but string was terminated",
                                ))
                            }
                        }
                    }
                    let argument = argument_string.as_str().trim_ascii();
                    if argument.is_empty() {
                        args.push_punct(FormatArgument::Positional);
                    } else {
                        match argument.parse() {
                            Ok(num) => {
                                args.push_punct(FormatArgument::Numbered(num));
                            }
                            Err(_) => match syn::parse_str(argument) {
                                Ok(ident) => {
                                    args.push_punct(FormatArgument::Named(ident));
                                }
                                Err(e) => return Err(syn::Error::new_spanned(lit, e)),
                            },
                        }
                    }
                }
                ('}', Some('}')) => partial.push('}'),
                ('}', _) => {
                    return Err(syn::Error::new_spanned(
                        lit,
                        "invalid format string: unmatched `}` found",
                    ))
                }
                (other, _) => partial.push(other),
            }
        }

        if partial.len() > 0 {
            args.push_value(LitStr::new(&partial, lit.span()));
        }

        Ok(Self { args, lit })
    }
}
