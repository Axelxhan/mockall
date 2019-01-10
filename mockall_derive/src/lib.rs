// vim: tw=80
extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};

/// Generate a mock identifier from the regular one: eg "Foo" => "MockFoo"
fn mock_ident(ident: &syn::Ident) -> syn::Ident {
    syn::Ident::new(&format!("Mock{}", ident), ident.span())
}

/// Generate a mock path from a regular one:
/// eg "foo::bar::Baz" => "foo::bar::MockBaz"
fn mock_path(path: &syn::Path) -> syn::Path {
    let mut outsegs = path.segments.clone();
    let mut last_segment = outsegs.last_mut().unwrap();
    last_segment.value_mut().ident = mock_ident(&last_segment.value().ident);
    syn::Path{leading_colon: path.leading_colon, segments: outsegs}
}

/// Generate a mock method and its expectation method
fn mock_method(meth: &syn::ImplItemMethod) -> (TokenStream, TokenStream) {
    let mut mock_output = TokenStream::new();
    // First the mock method
    if let Some(d) = meth.defaultness {
        d.to_tokens(&mut mock_output);
    }
    meth.vis.to_tokens(&mut mock_output);
    if let Some(c) = &meth.sig.constness {
        c.to_tokens(&mut mock_output);
    }
    if let Some(u) = &meth.sig.unsafety {
        u.to_tokens(&mut mock_output);
    }
    if let Some(a) = &meth.sig.asyncness {
        a.to_tokens(&mut mock_output);
    }
    if let Some(a) = &meth.sig.abi {
        a.to_tokens(&mut mock_output);
    }
    meth.sig.decl.fn_token.to_tokens(&mut mock_output);
    meth.sig.ident.to_tokens(&mut mock_output);
    meth.sig.decl.generics.to_tokens(&mut mock_output);
    let mut inputs = TokenStream::new();
    meth.sig.decl.inputs.to_tokens(&mut inputs);
    quote!((#inputs)).to_tokens(&mut mock_output);
    if let Some(v) = &meth.sig.decl.variadic {
        v.to_tokens(&mut mock_output);
    }
    meth.sig.decl.output.to_tokens(&mut mock_output);

    let mut input_type
        = syn::punctuated::Punctuated::<syn::Type, syn::Token![,]>::new();
    for fn_arg in meth.sig.decl.inputs.iter() {
        match fn_arg {
            syn::FnArg::Captured(arg) => input_type.push(arg.ty.clone()),
            syn::FnArg::SelfRef(_) => /* ignore */(),
            syn::FnArg::SelfValue(_) => /* ignore */(),
            _ => unimplemented!(),
        }
    }
    let output_type: syn::Type = match &meth.sig.decl.output {
        syn::ReturnType::Default => {
            let paren_token = syn::token::Paren{span: Span::call_site()};
            let elems = syn::punctuated::Punctuated::new();
            syn::Type::Tuple(syn::TypeTuple{paren_token, elems})
        },
        syn::ReturnType::Type(_, ty) => (**ty).clone()
    };
    let ident = format!("{}", meth.sig.ident);

    quote!({self.e.called::<(#input_type), #output_type>(#ident)})
        .to_tokens(&mut mock_output);

    // Then the expectation method
    let mut expect_output = TokenStream::new();
    let expect_ident = syn::Ident::new(&format!("expect_{}", meth.sig.ident),
                                       meth.sig.ident.span());
    quote!(pub fn #expect_ident(&mut self)
           -> mockall::Expectation<(#input_type), #output_type> {
        self.e.expect::<(#input_type), #output_type>(#ident)
   }).to_tokens(&mut expect_output);

    (mock_output, expect_output)
}

fn gen_impl(item: syn::ItemImpl) -> TokenStream {
    let mut output = TokenStream::new();
    let mut mock_body = TokenStream::new();
    let mut expect_body = TokenStream::new();

    let mock_type = match *item.self_ty {
        syn::Type::Path(type_path) => {
            assert!(type_path.qself.is_none(), "What is qself?");
            mock_path(&type_path.path)
        },
        _ => unimplemented!("This self type is not yet supported by MockAll")
    };

    for impl_item in item.items {
        match impl_item {
            syn::ImplItem::Const(_) => {
                // const items can easily be added by the user in a separate
                // impl block
            },
            syn::ImplItem::Existential(ty) => ty.to_tokens(&mut mock_body),
            syn::ImplItem::Type(ty) => ty.to_tokens(&mut mock_body),
            syn::ImplItem::Method(meth) => {
                let (mock_meth, expect_meth) = mock_method(&meth);
                mock_meth.to_tokens(&mut mock_body);
                expect_meth.to_tokens(&mut expect_body);
            },
            _ => {
                unimplemented!("This impl item is not yet supported by MockAll")
            }
        }
    }

    // Put all mock methods in one impl block
    item.unsafety.to_tokens(&mut output);
    item.impl_token.to_tokens(&mut output);
    item.generics.to_tokens(&mut output);
    if let Some(trait_) = item.trait_ {
        let (bang, path, for_) = trait_;
        if let Some(b) = bang {
            b.to_tokens(&mut output);
        }
        path.to_tokens(&mut output);
        for_.to_tokens(&mut output);
    }
    mock_type.to_tokens(&mut output);
    quote!({#mock_body}).to_tokens(&mut output);

    // Put all expect methods in a separate impl block.  This is necessary when
    // mocking a trait impl, where we can't add any new methods
    item.impl_token.to_tokens(&mut output);
    item.generics.to_tokens(&mut output);
    mock_type.to_tokens(&mut output);
    quote!({#expect_body}).to_tokens(&mut output);

    output
}

fn gen_struct(item: syn::ItemStruct) -> TokenStream {
    let mut output = TokenStream::new();
    item.vis.to_tokens(&mut output);
    item.struct_token.to_tokens(&mut output);
    let ident = mock_ident(&item.ident);
    ident.to_tokens(&mut output);
    item.generics.to_tokens(&mut output);

    let mut body: TokenStream = "e: mockall::Expectations,".parse().unwrap();
    let mut count = 0;
    for param in item.generics.params.iter() {
        if let syn::GenericParam::Type(tp) = param {
            let ty = &tp.ident;
            let phname = format!("_t{}", count);
            let phident = syn::Ident::new(&phname, tp.ident.span());
            let var = quote!(#phident: std::marker::PhantomData<#ty>,);
            var.to_tokens(&mut body);
            count += 1;
        }
    }
    quote!({#body}).to_tokens(&mut output);

    output
}

fn gen_mock(input: TokenStream) -> TokenStream {
    let item: syn::Item = match syn::parse(input.into()) {
        Ok(item) => item,
        Err(err) => {
            // TODO: use Span::call_site().error().emit once proc_macro_span is
            // stable
            // https://github.com/rust-lang/rust/issues/54725
            panic!("Failed to parse: {}", err);
        }
    };
    match item {
        syn::Item::Struct(item_struct) => gen_struct(item_struct),
        syn::Item::Impl(item_impl) => gen_impl(item_impl),
        _ => unimplemented!("TODO")
    }
}

/// Automatically generate mock types for Structs and Traits.
#[proc_macro_attribute]
pub fn mock(_attr: proc_macro::TokenStream, input: proc_macro::TokenStream)
    -> proc_macro::TokenStream
{
    let input: proc_macro2::TokenStream = input.into();
    let mut output = input.clone();
    output.extend(gen_mock(input));
    output.into()
}

#[cfg(feature = "internal_testing")]
#[proc_macro_attribute]
pub fn expect_mock(attr: proc_macro::TokenStream, item: proc_macro::TokenStream)
    -> proc_macro::TokenStream
{
    let expected = attr.to_string();
    let output = gen_mock(item.into()).to_string();
    assert_eq!(expected, output);
    proc_macro::TokenStream::new()
}

/// Test cases for `#[mock]`.
///
/// Proc macros cannot be tested at runtime like normal Rust code.  They can
/// only be tested at compile time.  Run the tests in doc test blocks so that a
/// compile failure by one does not stop the others from running.
///
#[allow(unused)]
#[cfg(feature = "internal_testing")]
mod t {
use super::*;

/// ```no_run
/// # use mockall_derive::{mock, expect_mock};
/// #[expect_mock(
/// struct MockSimpleStruct {
///     e: mockall::Expectations,
/// }
/// )]
/// struct SimpleStruct {
///     x: i16
/// }
/// #[expect_mock(
/// impl MockSimpleStruct {
///     fn foo(&self, x: u32) -> i64 {
///         self.e.called::<(u32), i64>("foo")
///     }
/// }
/// impl MockSimpleStruct {
///     pub fn expect_foo(&mut self) -> mockall::Expectation<(u32), i64> {
///         self.e.expect::<(u32), i64>("foo")
///     }
/// }
/// )]
/// impl SimpleStruct {
///     fn foo(&self, x: u32) -> i64 {
///         42
///     }
/// }
/// ```
type SimpleStruct = ();

/// ```no_run
/// # use mockall_derive::{mock, expect_mock};
/// #[expect_mock(
/// struct MockGenericStruct<'a, T, V> {
///     e: mockall::Expectations,
///     _t0: std::marker::PhantomData<T>,
///     _t1: std::marker::PhantomData<V>,
/// }
/// )]
/// struct GenericStruct<'a, T, V> {
///     x: i16
/// }
/// #[expect_mock(
/// impl<'a, T, V> MockGenericStruct<'a, T, V> {
///     fn foo(&self, x: u32) -> i64 {
///         self.e.called::<(u32), i64>("foo")
///     }
/// }
/// impl<'a, T, V> MockGenericStruct<'a, T, V> {
///     pub fn expect_foo(&mut self) -> mockall::Expectation<(u32), i64> {
///         self.e.expect::<(u32), i64>("foo")
///     }
/// }
/// )]
/// impl<'a, T, V> GenericStruct<'a, T, V> {
///     fn foo(&self, x: u32) -> i64 {
///         42
///     }
/// }
/// ```
type GenericStruct = ();

/// ```no_run
/// # use mockall_derive::{mock, expect_mock};
/// #[expect_mock(
/// impl Foo for MockSomeStruct {
///     fn foo(&self, x: u32) -> i64 {
///         self.e.called::<(u32), i64>("foo")
///     }
/// }
/// impl MockSomeStruct {
///     pub fn expect_foo(&mut self) -> mockall::Expectation<(u32), i64> {
///         self.e.expect::<(u32), i64>("foo")
///     }
/// }
/// )]
/// impl Foo for SomeStruct {
///     fn foo(&self, x: u32) -> i64 {
///         42
///     }
/// }
/// ```
type ImplTrait = ();

/// ```no_run
/// # use mockall_derive::{mock, expect_mock};
/// #[expect_mock(
/// impl MockMethodByValue {
///     fn foo(self, x: u32) -> i64 {
///         self.e.called::<(u32), i64>("foo")
///     }
/// }
/// impl MockMethodByValue {
///     pub fn expect_foo(&mut self) -> mockall::Expectation<(u32), i64> {
///         self.e.expect::<(u32), i64>("foo")
///     }
/// }
/// )]
/// impl MethodByValue {
///     fn foo(self, x: u32) -> i64 {
///         42
///     }
/// }
/// ```
type MethodByValue = ();

/// ```no_run
/// # use mockall_derive::{mock, expect_mock};
/// #[expect_mock(
/// pub struct MockPubStruct {
///     e: mockall::Expectations,
/// }
/// )]
/// pub struct PubStruct {
///     x: i16
/// }
/// #[expect_mock(
/// impl MockPubStruct {
///     pub fn foo(&self, x: u32) -> i64 {
///         self.e.called::<(u32), i64>("foo")
///     }
/// }
/// impl MockPubStruct {
///     pub fn expect_foo(&mut self) -> mockall::Expectation<(u32), i64> {
///         self.e.expect::<(u32), i64>("foo")
///     }
/// }
/// )]
/// impl PubStruct {
///     pub fn foo(&self, x: u32) -> i64 {
///         42
///     }
/// }
/// ```
type PubStruct = ();

/// ```no_run
/// # use mockall_derive::{mock, expect_mock};
/// #[expect_mock(
/// pub(crate) struct MockPubCrateStruct {
///     e: mockall::Expectations,
/// }
/// )]
/// pub(crate) struct PubCrateStruct {
///     x: i16
/// }
/// #[expect_mock(
/// impl MockPubCrateStruct {
///     pub(crate) fn foo(&self, x: u32) -> i64 {
///         self.e.called::<(u32), i64>("foo")
///     }
/// }
/// impl MockPubCrateStruct {
///     pub fn expect_foo(&mut self) -> mockall::Expectation<(u32), i64> {
///         self.e.expect::<(u32), i64>("foo")
///     }
/// }
/// )]
/// impl PubCrateStruct {
///     pub(crate) fn foo(&self, x: u32) -> i64 {
///         42
///     }
/// }
/// ```
type PubCrateStruct = ();

/// ```no_run
/// # use mockall_derive::{mock, expect_mock};
/// #[expect_mock(
/// pub(super) struct MockPubSuperStruct {
///     e: mockall::Expectations,
/// }
/// )]
/// pub(super) struct PubSuperStruct {
///     x: i16
/// }
/// #[expect_mock(
/// impl MockPubSuperStruct {
///     pub(super) fn foo(&self, x: u32) -> i64 {
///         self.e.called::<(u32), i64>("foo")
///     }
/// }
/// impl MockPubSuperStruct {
///     pub fn expect_foo(&mut self) -> mockall::Expectation<(u32), i64> {
///         self.e.expect::<(u32), i64>("foo")
///     }
/// }
/// )]
/// impl PubSuperStruct {
///     pub(super) fn foo(&self, x: u32) -> i64 {
///         42
///     }
/// }
/// ```
type PubSuperStruct = ();
}
