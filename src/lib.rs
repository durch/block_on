use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Block, FnArg, Ident, ImplItem, ItemImpl, LitStr};

/// Generate a blocking method for each async method in an impl block. Supports either `tokio` or `async-std` backend.
/// Generated methods are suffixed with `_blocking`.
///
/// # Example `tokio`
/// ```
/// use block_on_proc::block_on;
///
/// struct Tokio {}
///
/// #[block_on("tokio")]
/// impl Tokio {
///     async fn test_async(&self) {}        
/// }
/// ```
///
/// Generates the following impl block
/// ```no_run
/// # struct Dummy {}
/// # impl Dummy {
/// async fn test_async(&self) {}
///         
/// fn test_async_blocking(&self) {
///     use tokio::runtime::Runtime;
///     let mut rt = Runtime::new().unwrap();
///     rt.block_on(self.test_async())
/// }
/// # }
/// ```
///
/// # Example `async-std`
/// ```
/// use block_on_proc::block_on;
///
/// struct AsyncStd {}
///
/// #[block_on("async-std")]
/// impl AsyncStd {
///     async fn test_async(&self) {}        
/// }
/// ```
///
/// Generates the following method in the same impl block
/// ```no_run
/// # struct Dummy {}
/// # impl Dummy {
/// async fn test_async(&self) {}        
///
/// fn test_async_blocking(&self) {
///       use async_std::task;
///       task::block_on(self.test_async())
/// }
/// # }
/// ```

#[proc_macro_attribute]
pub fn block_on(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as LitStr).value();

    let orig_tokens = tokens.clone();

    let in_impl = parse_macro_input!(orig_tokens as ItemImpl);
    let strct = in_impl.self_ty.clone();
    let mut orig_impl = in_impl.clone();
    let mut out_impl = in_impl.clone();
    out_impl.items = Vec::new();

    for item in in_impl.items {
        match item {
            ImplItem::Method(method) => {
                let name = &method.sig.ident;
                let mut out_method = method.clone();
                if out_method.sig.asyncness.is_none() {
                    continue;
                }
                out_method.sig.asyncness = None;

                out_method.sig.ident = Ident::new(
                    &format!("{}_blocking", method.sig.ident.to_string()),
                    method.sig.ident.span(),
                );

                let inputs = &method.sig.inputs;

                let rec = inputs.into_iter().any(|arg| match arg {
                    FnArg::Receiver(_) => true,
                    FnArg::Typed(_) => false,
                });

                let call_args = inputs
                    .into_iter()
                    .map(|arg| match arg {
                        FnArg::Receiver(_) => None,
                        FnArg::Typed(arg) => Some(arg.pat.clone()),
                    })
                    .filter(|pat| pat.is_some())
                    .map(|arg| arg.unwrap());

                let block_proc2 = if rec {
                    {
                        if attr == "tokio" {
                            quote! {
                                    {
                                        use tokio::runtime::Runtime;
                                        let mut rt = Runtime::new().unwrap();
                                        rt.block_on(self.#name(#(#call_args),*))
                                    }
                            }
                        } else if attr == "async-std" {
                            quote! {
                                    {
                                        use async_std::task;
                                        task::block_on(self.#name(#(#call_args),*))
                                    }
                            }
                        } else {
                            panic!("Only `tokio` and `async-std` backends are supported!")
                        }
                    }
                } else {
                    if attr == "tokio" {
                        quote! {
                                {
                                    use tokio::runtime::Runtime;
                                    let mut rt = Runtime::new().unwrap();
                                    rt.block_on(#strct::#name(#(#call_args),*))
                                }
                        }
                    } else if attr == "async-std" {
                        quote! {
                                {
                                    use async_std::task;
                                    task::block_on(#strct::#name(#(#call_args),*))
                                }
                        }
                    } else {
                        panic!("Only `tokio` and `async-std` backends are supported!")
                    }
                };

                let block_proc = proc_macro::TokenStream::from(block_proc2);
                out_method.block = parse_macro_input!(block_proc as Block);
                orig_impl.items.push(ImplItem::Method(out_method));
            }
            _ => {}
        }
    }

    // Returns generated tokens
    let out = quote! {
        #orig_impl
    };

    out.into()
}

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("src/test.rs");
}
