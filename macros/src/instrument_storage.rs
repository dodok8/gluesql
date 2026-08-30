use {
    crate::resolve_gluesql_crate,
    proc_macro2::TokenStream,
    quote::quote,
    syn::{
        Expr, ExprLit, FnArg, GenericArgument, ImplItem, ItemImpl, Lit, MetaNameValue, Pat,
        PathArguments, ReturnType, Token, Type, parse::Parser, punctuated::Punctuated,
    },
};

struct Args {
    name: String,
    capture_full: bool,
    trace_iterators: bool,
}

impl Args {
    fn parse(tokens: TokenStream) -> Result<Self, syn::Error> {
        let values = Punctuated::<MetaNameValue, Token![,]>::parse_terminated.parse2(tokens)?;
        let mut name = None;
        let mut capture_full = true;
        let mut trace_iterators = true;

        for MetaNameValue { path, value, .. } in values {
            let Expr::Lit(ExprLit {
                lit: Lit::Str(value),
                ..
            }) = value
            else {
                return Err(syn::Error::new_spanned(value, "expected a string literal"));
            };

            if path.is_ident("name") {
                name = Some(value.value());
            } else if path.is_ident("capture") {
                capture_full = parse_mode(&value, "capture")?;
            } else if path.is_ident("iterator") {
                trace_iterators = parse_mode(&value, "iterator")?;
            } else {
                return Err(syn::Error::new_spanned(path, "unsupported option"));
            }
        }

        Ok(Self {
            name: name.ok_or_else(|| {
                syn::Error::new(proc_macro2::Span::call_site(), "missing `name = \"...\"`")
            })?,
            capture_full,
            trace_iterators,
        })
    }
}

fn parse_mode(value: &syn::LitStr, option: &str) -> Result<bool, syn::Error> {
    match value.value().as_str() {
        "full" => Ok(true),
        "off" => Ok(false),
        _ => Err(syn::Error::new(
            value.span(),
            format!("`{option}` must be `full` or `off`"),
        )),
    }
}

pub fn expand(attr: TokenStream, item: TokenStream) -> Result<TokenStream, syn::Error> {
    let args = Args::parse(attr)?;
    let mut implementation: ItemImpl = syn::parse2(item)?;
    let gluesql = resolve_gluesql_crate()?;

    for item in &mut implementation.items {
        let ImplItem::Fn(method) = item else {
            continue;
        };

        let method_name = method.sig.ident.to_string();
        let span_name = format!("gluesql.{}.{method_name}", args.name);
        let mut fields = Vec::new();

        if args.capture_full {
            for input in &method.sig.inputs {
                let FnArg::Typed(input) = input else {
                    continue;
                };
                let Pat::Ident(pattern) = input.pat.as_ref() else {
                    continue;
                };
                let ident = &pattern.ident;
                fields.push(quote!(#ident = ?#ident));

                if ident == "rows" || ident == "keys" {
                    fields.push(quote!(row_count = #ident.len()));
                }
            }
        }

        let records_error = result_ok_type(&method.sig.output).is_some();
        let attribute = match (fields.is_empty(), records_error) {
            (true, true) => quote!(
                #[tracing::instrument(
                    target = "gluesql",
                    name = #span_name,
                    level = "trace",
                    skip_all,
                    err(Debug)
                )]
            ),
            (true, false) => quote!(
                #[tracing::instrument(
                    target = "gluesql",
                    name = #span_name,
                    level = "trace",
                    skip_all
                )]
            ),
            (false, true) => quote!(
                #[tracing::instrument(
                    target = "gluesql",
                    name = #span_name,
                    level = "trace",
                    skip_all,
                    fields(#(#fields),*),
                    err(Debug)
                )]
            ),
            (false, false) => quote!(
                #[tracing::instrument(
                    target = "gluesql",
                    name = #span_name,
                    level = "trace",
                    skip_all,
                    fields(#(#fields),*)
                )]
            ),
        };
        let mut attribute = syn::Attribute::parse_outer.parse2(attribute)?;
        method.attrs.append(&mut attribute);

        let explicitly_traced = method
            .attrs
            .iter()
            .any(|attribute| attribute.path().is_ident("trace_iterator"));
        method
            .attrs
            .retain(|attribute| !attribute.path().is_ident("trace_iterator"));
        let should_trace_iterator = args.trace_iterators
            && (explicitly_traced
                || matches!(method_name.as_str(), "scan_data" | "scan_indexed_data"));

        if should_trace_iterator {
            let ok_type = result_ok_type(&method.sig.output).ok_or_else(|| {
                syn::Error::new_spanned(
                    &method.sig.output,
                    "traced iterator methods must return `Result<Box<dyn Iterator<...>>>`",
                )
            })?;
            let iterator_span_name = format!("gluesql.{}.{}_rows", args.name, method_name);
            let block = &method.block;
            method.block = syn::parse_quote!({
                let __gluesql_result = (|| #block)();
                __gluesql_result.map(|__gluesql_iterator| {
                    let __gluesql_span = tracing::trace_span!(
                        target: "gluesql",
                        #iterator_span_name,
                        row_count = tracing::field::Empty,
                        error_count = tracing::field::Empty,
                        completed = tracing::field::Empty
                    );
                    let __gluesql_iterator: #ok_type = Box::new(
                        #gluesql::__private::TracedResultIterator::new(
                            __gluesql_iterator,
                            __gluesql_span,
                        ),
                    );
                    __gluesql_iterator
                })
            });
        }
    }

    Ok(quote!(#implementation))
}

fn result_ok_type(output: &ReturnType) -> Option<Type> {
    let ReturnType::Type(_, output) = output else {
        return None;
    };
    let Type::Path(output) = output.as_ref() else {
        return None;
    };
    let result = output.path.segments.last()?;
    if result.ident != "Result" {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &result.arguments else {
        return None;
    };

    arguments.args.iter().find_map(|argument| match argument {
        GenericArgument::Type(ok_type) => Some(ok_type.clone()),
        _ => None,
    })
}
