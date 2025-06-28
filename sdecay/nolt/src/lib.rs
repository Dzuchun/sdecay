#![allow(missing_docs, reason = "That's an internal procmacro crate")]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    AngleBracketedGenericArguments, AssocConst, AssocType, BareFnArg, BoundLifetimes, ConstParam,
    Constraint, GenericArgument, GenericParam, Ident, Lifetime, LifetimeParam,
    ParenthesizedGenericArguments, Path, PathArguments, PathSegment, QSelf, ReturnType, TraitBound,
    Type, TypeArray, TypeBareFn, TypeGroup, TypeImplTrait, TypeParam, TypeParamBound, TypeParen,
    TypePath, TypePtr, TypeReference, TypeSlice, TypeTraitObject, TypeTuple, parse_macro_input,
    punctuated::Punctuated,
};

trait Nolt {
    fn transform(&mut self);
}

impl<T: Nolt> Nolt for Box<T> {
    fn transform(&mut self) {
        T::transform(&mut **self);
    }
}

impl<T: Nolt> Nolt for Option<T> {
    fn transform(&mut self) {
        if let Some(s) = self.as_mut() {
            s.transform();
        }
    }
}

impl<T: Nolt> Nolt for Vec<T> {
    fn transform(&mut self) {
        self.iter_mut().for_each(T::transform);
    }
}

impl<T: Nolt, S> Nolt for Punctuated<T, S> {
    fn transform(&mut self) {
        self.iter_mut().for_each(T::transform);
    }
}

//

impl Nolt for Lifetime {
    fn transform(&mut self) {
        self.ident = Ident::new("static", Span::call_site());
    }
}

impl Nolt for GenericParam {
    fn transform(&mut self) {
        match self {
            GenericParam::Lifetime(LifetimeParam {
                lifetime, bounds, ..
            }) => {
                lifetime.transform();
                bounds.transform();
            }
            GenericParam::Type(TypeParam {
                bounds, default, ..
            }) => {
                bounds.transform();
                default.transform();
            }
            GenericParam::Const(ConstParam { ty, .. }) => {
                ty.transform();
            }
        }
    }
}

impl Nolt for BoundLifetimes {
    fn transform(&mut self) {
        self.lifetimes.transform();
    }
}

impl Nolt for BareFnArg {
    fn transform(&mut self) {
        self.ty.transform();
    }
}

impl Nolt for ReturnType {
    fn transform(&mut self) {
        match self {
            ReturnType::Default => {}
            ReturnType::Type(_, ty) => ty.transform(),
        }
    }
}

impl Nolt for GenericArgument {
    fn transform(&mut self) {
        match self {
            GenericArgument::Lifetime(lifetime) => lifetime.transform(),
            GenericArgument::Type(tp) => tp.transform(),
            GenericArgument::AssocType(AssocType { generics, ty, .. }) => {
                generics.transform();
                ty.transform();
            }
            GenericArgument::AssocConst(AssocConst { generics, .. }) => {
                generics.transform();
            }
            GenericArgument::Constraint(Constraint {
                generics, bounds, ..
            }) => {
                generics.transform();
                bounds.transform();
            }
            _ => {}
        }
    }
}

impl Nolt for AngleBracketedGenericArguments {
    fn transform(&mut self) {
        let Self { args, .. } = self;
        args.transform();
    }
}

impl Nolt for ParenthesizedGenericArguments {
    fn transform(&mut self) {
        let Self { inputs, output, .. } = self;
        inputs.transform();
        output.transform();
    }
}

impl Nolt for PathArguments {
    fn transform(&mut self) {
        match self {
            PathArguments::None => {}
            PathArguments::AngleBracketed(angle) => angle.transform(),
            PathArguments::Parenthesized(paren) => paren.transform(),
        }
    }
}

impl Nolt for PathSegment {
    fn transform(&mut self) {
        self.arguments.transform();
    }
}

impl Nolt for Path {
    fn transform(&mut self) {
        self.segments.transform();
    }
}

impl Nolt for TypeParamBound {
    fn transform(&mut self) {
        match self {
            syn::TypeParamBound::Trait(TraitBound {
                lifetimes, path, ..
            }) => {
                lifetimes.transform();
                path.transform();
            }
            syn::TypeParamBound::Lifetime(lifetime) => lifetime.transform(),
            _ => {}
        }
    }
}

impl Nolt for QSelf {
    fn transform(&mut self) {
        let Self { ty, .. } = self;
        ty.transform();
    }
}

impl Nolt for Type {
    fn transform(&mut self) {
        match self {
            Type::BareFn(TypeBareFn {
                lifetimes,
                inputs,
                output,
                ..
            }) => {
                lifetimes.transform();
                inputs.transform();
                output.transform();
            }
            Type::Group(TypeGroup { elem, .. })
            | Type::Paren(TypeParen { elem, .. })
            | Type::Ptr(TypePtr { elem, .. })
            | Type::Slice(TypeSlice { elem, .. })
            | Type::Array(TypeArray { elem, .. }) => {
                elem.transform();
            }
            Type::ImplTrait(TypeImplTrait { bounds, .. })
            | Type::TraitObject(TypeTraitObject { bounds, .. }) => bounds.transform(),
            Type::Path(TypePath { qself, path }) => {
                qself.transform();
                path.transform();
            }
            Type::Reference(TypeReference { lifetime, elem, .. }) => {
                lifetime.transform();
                elem.transform();
            }
            Type::Tuple(TypeTuple { elems, .. }) => elems.transform(),
            _ => {}
        }
    }
}

#[proc_macro]
pub fn nolt(input: TokenStream) -> TokenStream {
    let mut tp = parse_macro_input!(input as Type);
    tp.transform();
    tp.to_token_stream().into()
}
