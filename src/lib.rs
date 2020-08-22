//! This crate provides an attribute macro to associate struct-type constants with enum variants.
//!
//! ## Syntax
//!
//! Place `#[const_table]` on an enum with at least two variants, where
//!
//! * the first has named fields and defines the type of the associated constants, and
//! * all following have discriminant expressions of that type:
//!
//! ```
//! use const_table::const_table;
//!
//! #[const_table]
//! pub enum Planet {
//!     PlanetInfo {
//!         pub mass: f32,
//!         pub radius: f32,
//!     },
//!
//!     Mercury = PlanetInfo { mass: 3.303e+23, radius: 2.4397e6 },
//!     Venus = PlanetInfo { mass: 4.869e+24, radius: 6.0518e6 },
//!     Earth = PlanetInfo { mass: 5.976e+24, radius: 6.37814e6 },
//!     Mars = PlanetInfo { mass: 6.421e+23, radius: 3.3972e6 },
//!     Jupiter = PlanetInfo { mass: 1.9e+27, radius: 7.1492e7 },
//!     Saturn = PlanetInfo { mass: 5.688e+26, radius: 6.0268e7 },
//!     Uranus = PlanetInfo { mass: 8.686e+25, radius: 2.5559e7 },
//!     Neptune = PlanetInfo { mass: 1.024e+26, radius: 2.4746e7 },
//! }
//! ```
//!
//! This expands to the following:
//!
//! ```
//! #[repr(u32)]
//! #[derive(core::marker::Copy, core::clone::Clone, core::fmt::Debug, core::hash::Hash, core::cmp::PartialEq, core::cmp::Eq)]
//! pub enum Planet {
//!     Mercury,
//!     Venus,
//!     Earth,
//!     Mars,
//!     Jupiter,
//!     Saturn,
//!     Uranus,
//!     Neptune,
//! }
//!
//! pub struct PlanetInfo {
//!     pub mass: f32,
//!     pub radius: f32,
//! }
//!
//! impl Planet {
//!     const COUNT: usize = 8;
//!     pub fn iter() -> impl core::iter::DoubleEndedIterator<Item = Self> {
//!         // transmuting here is fine because... (see try_from)
//!         (0..Self::COUNT).map(|i| unsafe { core::mem::transmute(i as u32) })
//!     }
//! }
//!
//! impl core::ops::Deref for Planet {
//!     type Target = PlanetInfo;
//!     fn deref(&self) -> &Self::Target {
//!         use Planet::*;
//!         const TABLE: [PlanetInfo; 8] = [
//!             PlanetInfo { mass: 3.303e+23, radius: 2.4397e6 },
//!             PlanetInfo { mass: 4.869e+24, radius: 6.0518e6 },
//!             PlanetInfo { mass: 5.976e+24, radius: 6.37814e6 },
//!             PlanetInfo { mass: 6.421e+23, radius: 3.3972e6 },
//!             PlanetInfo { mass: 1.9e+27, radius: 7.1492e7 },
//!             PlanetInfo { mass: 5.688e+26, radius: 6.0268e7 },
//!             PlanetInfo { mass: 8.686e+25, radius: 2.5559e7 },
//!             PlanetInfo { mass: 1.024e+26, radius: 2.4746e7 },
//!         ];
//!
//!         &TABLE[*self as usize]
//!     }
//! }
//!
//! impl core::convert::TryFrom<u32> for Planet {
//!     type Error = u32;
//!     fn try_from(i: u32) -> Result<Self, Self::Error> {
//!         if (i as usize) < Self::COUNT {
//!             // transmuting here is fine because all values in range are valid, since
//!             // discriminants are assigned linearly starting at 0.
//!             Ok(unsafe { core::mem::transmute(i) })
//!         } else {
//!             Err(i)
//!         }
//!     }
//! }
//! ```
//!
//! Note the automatically inserted `repr` and `derive` attributes. You may place a different `repr` attribute as normal,
//! although only `u8`, `u16`, `u32` and `u64` are supported; an implementation of `TryFrom<T>` is provided, where `T` is
//! the chosen `repr` type. You may also `derive` additional traits on the enum.
//!
//! Any attributes placed on the first variant will be placed on the corresponding struct in the expanded code.
//!
//! Also, note that the macro places the discriminant expressions inside a scope that imports all variants of your enum.
//! This makes it convenient to make the values refer to each other, e.g. in a graph-like structure.
//!
//! Because the macro implements `Deref` for your enum, you can access fields of the target type like `Planet::Earth.mass`.
//!
//! Finally, `Planet::iter()` gives a `DoubleEndedIterator` over all variants in declaration order, and `Planet::COUNT` is
//! the total number of variants.

extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use proc_macro2::Span;

use quote::quote;
use syn::parse::Error;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Expr, Ident, ItemEnum, ItemStruct, Variant};

#[proc_macro_attribute]
pub fn const_table(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut errors = proc_macro2::TokenStream::new();

    let input_item = parse_macro_input!(item as syn::Item);
    let input_item = if let syn::Item::Enum(e) = input_item {
        e
    } else {
        let span = input_item.span();
        let message = "the const_table attribute may only be applied to enums";
        return Error::new(span, message).to_compile_error().into();
    };

    if !input_item.generics.params.is_empty() {
        let span = input_item.generics.params.span();
        let message = "a const_table enum cannot be generic";
        errors.extend(Error::new(span, message).to_compile_error());
    }

    let (enum_attrs, repr_type) = {
        let mut attrs = Vec::with_capacity(input_item.attrs.len());
        let mut repr = None;

        for attr in input_item.attrs {
            if attr.path.is_ident("derive") {
                let mut conflict_found = false;
                if let Ok(syn::Meta::List(derive_attr)) = attr.parse_meta() {
                    for arg in &derive_attr.nested {
                        if let syn::NestedMeta::Meta(syn::Meta::Path(p)) = arg {
                            if p.is_ident("Copy") || p.is_ident("Clone") ||
                                p.is_ident("Debug") || p.is_ident("Hash") ||
                                p.is_ident("PartialEq") || p.is_ident("Eq")
                            {
                                let span = p.span();
                                let message = format!("the {} trait is already implemented by the const_table macro", p.get_ident().unwrap());
                                errors.extend(Error::new(span, message).to_compile_error());
                                conflict_found = true;
                            }
                        }
                    }
                }

                if conflict_found {
                    continue;
                }
            }

            if attr.path.is_ident("repr") {
                let ident: Ident = attr.parse_args().unwrap();
                if ident != "u8" && ident != "u16" && ident != "u32" && ident != "u64" {
                    let span = attr.tokens.span();
                    let message = "unsupported repr hint for a const_table enum: expected one of u8, u16, u32 or u64 (default is u32)";
                    errors.extend(Error::new(span, message).to_compile_error());
                    continue;
                }

                repr = Some(ident);
            } else {
                attrs.push(attr);
            }
        }

        (attrs, repr.unwrap_or_else(|| Ident::new("u32", Span::call_site())))
    };

    let mut input_variants = input_item.variants.iter();
    let first_variant = input_variants.next();

    let (variants, value_exprs): (Punctuated<Variant, syn::token::Comma>, Vec<Expr>) = input_variants.map(|variant| {
        if !variant.fields.is_empty() {
            let span = variant.fields.span();
            let message = "in a const_table enum, only the first variant should have fields";
            errors.extend(Error::new(span, message).to_compile_error());
        }

        if let Some((_, expr)) = &variant.discriminant {
            let v = Variant {
                discriminant: None,
                fields: syn::Fields::Unit,
                ..(*variant).clone()
            };

            (v, expr.clone())
        } else {
            let span = variant.span();
            let message = "in a const_table enum, all but the first variant should have a discriminant expression";
            errors.extend(Error::new(span, message).to_compile_error());

            let empty_expr = Expr::Tuple(syn::ExprTuple {
                attrs: Vec::new(), paren_token: syn::token::Paren { span: variant.ident.span() }, elems: Punctuated::new()
            });

            (variant.clone(), empty_expr)
        }
    }).unzip();

    if variants.is_empty() {
        let span = input_item.brace_token.span;
        let message = "a const_table enum needs at least one variant with a discriminant expression";
        errors.extend(Error::new(span, message).to_compile_error());
        return errors.into();
    }

    let struct_decl = if let Some(v) = first_variant {
        use syn::Fields::Named;
        if let Named(fields) = &v.fields {
            ItemStruct {
                attrs: v.attrs.clone(),
                vis: input_item.vis.clone(),
                struct_token: syn::token::Struct {
                    span: Span::call_site(),
                },
                ident: v.ident.clone(),
                generics: Default::default(),
                fields: Named((*fields).clone()),
                semi_token: None,
            }
        } else {
            let span = v.span();
            let message = "the first variant of a const_table enum should have named fields to specify the table layout";
            errors.extend(Error::new(span, message).to_compile_error());
            return errors.into();
        }
    } else {
        let span = input_item.brace_token.span;
        let message = "a const_table enum needs at least one variant with named fields to specify the table layout";
        errors.extend(Error::new(span, message).to_compile_error());
        return errors.into();
    };
    let struct_name = &struct_decl.ident;

    let table_size = variants.len();
    let enum_decl = ItemEnum {
        attrs: enum_attrs,
        variants,
        ..input_item
    };
    let enum_name = &enum_decl.ident;

    let expanded = quote! {
        #errors

        #[repr(#repr_type)]
        #[derive(core::marker::Copy, core::clone::Clone, core::fmt::Debug, core::hash::Hash, core::cmp::PartialEq, core::cmp::Eq)]
        #enum_decl

        #struct_decl

        impl #enum_name {
            pub const COUNT: usize = #table_size;
            pub fn iter() -> impl core::iter::DoubleEndedIterator<Item = Self> {
                // transmuting here is fine because... (see try_from)
                (0..Self::COUNT).map(|i| unsafe { core::mem::transmute(i as #repr_type) })
            }
        }

        impl core::ops::Deref for #enum_name {
            type Target = #struct_name;
            fn deref(&self) -> &Self::Target {
                use #enum_name::*;
                const TABLE: [#struct_name; #table_size] = [ #(#value_exprs),* ];
                &TABLE[*self as usize]
            }
        }

        impl core::convert::TryFrom<#repr_type> for #enum_name {
            type Error = #repr_type;
            fn try_from(i: #repr_type) -> core::result::Result<Self, #repr_type> {
                if (i as usize) < Self::COUNT {
                    // transmuting here is fine because all values in range are valid, since
                    // discriminants are assigned linearly starting at 0.
                    core::result::Result::Ok(unsafe { core::mem::transmute(i) })
                } else {
                    core::result::Result::Err(i)
                }
            }
        }
    };
    expanded.into()
}
