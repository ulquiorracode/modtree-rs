//! Procedural macro attributes for `modtree`.

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Expr, ExprLit, ExprPath, GenericParam, ItemImpl, ItemStruct, Lit, Meta,
    MetaNameValue, Path, Token, parse_macro_input, parse_quote,
};

/// Parsed metadata configuration extracted from macro attributes.
#[derive(Default)]
struct NodeMetaArgs {
    name: Option<String>,
    enter: Option<Path>,
    exit: Option<Path>,
    context_ty: Option<syn::Type>,
    intent_ty: Option<syn::Type>,
    outcome_ty: Option<syn::Type>,
    error_ty: Option<syn::Type>,
}

fn parse_type_from_expr(expr: &Expr) -> syn::Result<syn::Type> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(s), ..
    }) = expr
    {
        syn::parse_str::<syn::Type>(&s.value())
    } else {
        syn::parse2::<syn::Type>(quote!(#expr))
    }
}

impl Parse for NodeMetaArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut meta = NodeMetaArgs::default();
        if input.is_empty() {
            return Ok(meta);
        }

        let punctuated: Punctuated<Meta, Token![,]> = Punctuated::parse_terminated(input)?;
        for item in punctuated {
            match item {
                Meta::NameValue(MetaNameValue { path, value, .. }) => {
                    let ident = path
                        .get_ident()
                        .ok_or_else(|| syn::Error::new_spanned(&path, "expected identifier"))?;
                    match ident.to_string().as_str() {
                        "name" => {
                            if let Expr::Lit(ExprLit {
                                lit: Lit::Str(lit_str),
                                ..
                            }) = &value
                            {
                                meta.name = Some(lit_str.value());
                            } else if let Expr::Path(ExprPath { path, .. }) = &value {
                                meta.name = path.get_ident().map(|id| id.to_string());
                            }
                        }
                        "enter" => {
                            if let Expr::Path(ExprPath { path, .. }) = &value {
                                meta.enter = Some(path.clone());
                            } else {
                                return Err(syn::Error::new_spanned(
                                    &value,
                                    "expected function path for enter hook",
                                ));
                            }
                        }
                        "exit" => {
                            if let Expr::Path(ExprPath { path, .. }) = &value {
                                meta.exit = Some(path.clone());
                            } else {
                                return Err(syn::Error::new_spanned(
                                    &value,
                                    "expected function path for exit hook",
                                ));
                            }
                        }
                        "context" | "ctx" => {
                            meta.context_ty = Some(parse_type_from_expr(&value)?);
                        }
                        "intent" => {
                            meta.intent_ty = Some(parse_type_from_expr(&value)?);
                        }
                        "outcome" => {
                            meta.outcome_ty = Some(parse_type_from_expr(&value)?);
                        }
                        "error" | "err" => {
                            meta.error_ty = Some(parse_type_from_expr(&value)?);
                        }
                        _ => {}
                    }
                }
                Meta::List(list) => {
                    let ident = list.path.get_ident().ok_or_else(|| {
                        syn::Error::new_spanned(&list.path, "expected identifier")
                    })?;
                    match ident.to_string().as_str() {
                        "name" => {
                            let lit: Lit = syn::parse2(list.tokens)?;
                            if let Lit::Str(s) = lit {
                                meta.name = Some(s.value());
                            }
                        }
                        "enter" => {
                            let path: Path = syn::parse2(list.tokens)?;
                            meta.enter = Some(path);
                        }
                        "exit" => {
                            let path: Path = syn::parse2(list.tokens)?;
                            meta.exit = Some(path);
                        }
                        "context" | "ctx" => {
                            let ty: syn::Type = syn::parse2(list.tokens)?;
                            meta.context_ty = Some(ty);
                        }
                        "intent" => {
                            let ty: syn::Type = syn::parse2(list.tokens)?;
                            meta.intent_ty = Some(ty);
                        }
                        "outcome" => {
                            let ty: syn::Type = syn::parse2(list.tokens)?;
                            meta.outcome_ty = Some(ty);
                        }
                        "error" | "err" => {
                            let ty: syn::Type = syn::parse2(list.tokens)?;
                            meta.error_ty = Some(ty);
                        }
                        _ => {}
                    }
                }
                Meta::Path(p) => {
                    if let Some(id) = p.get_ident().filter(|_| meta.name.is_none()) {
                        meta.name = Some(id.to_string());
                    }
                }
            }
        }
        Ok(meta)
    }
}

/// Extracts helper attributes like `#[name(...)]`, `#[enter(...)]`, `#[exit(...)]`.
fn extract_sibling_attributes(attrs: &mut Vec<Attribute>, meta: &mut NodeMetaArgs) {
    let mut retained = Vec::new();
    for attr in attrs.drain(..) {
        let segs: Vec<_> = attr
            .path()
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect();
        let is_modtree_attr = if segs.len() == 1 {
            matches!(
                segs[0].as_str(),
                "name"
                    | "enter"
                    | "exit"
                    | "context"
                    | "ctx"
                    | "intent"
                    | "outcome"
                    | "error"
                    | "err"
            )
        } else if segs.len() == 2 && segs[0] == "modtree" {
            matches!(
                segs[1].as_str(),
                "name"
                    | "enter"
                    | "exit"
                    | "context"
                    | "ctx"
                    | "intent"
                    | "outcome"
                    | "error"
                    | "err"
            )
        } else {
            false
        };

        if is_modtree_attr {
            let last_seg = segs.last().unwrap();
            match last_seg.as_str() {
                "name" => {
                    if let Ok(parsed) = attr.parse_args::<syn::LitStr>() {
                        meta.name = Some(parsed.value());
                    } else if let Ok(parsed) = attr.parse_args::<syn::Ident>() {
                        meta.name = Some(parsed.to_string());
                    }
                }
                "enter" => {
                    if let Ok(path) = attr.parse_args::<Path>() {
                        meta.enter = Some(path);
                    }
                }
                "exit" => {
                    if let Ok(path) = attr.parse_args::<Path>() {
                        meta.exit = Some(path);
                    }
                }
                "context" | "ctx" => {
                    if let Ok(ty) = attr.parse_args::<syn::Type>() {
                        meta.context_ty = Some(ty);
                    }
                }
                "intent" => {
                    if let Ok(ty) = attr.parse_args::<syn::Type>() {
                        meta.intent_ty = Some(ty);
                    }
                }
                "outcome" => {
                    if let Ok(ty) = attr.parse_args::<syn::Type>() {
                        meta.outcome_ty = Some(ty);
                    }
                }
                "error" | "err" => {
                    if let Ok(ty) = attr.parse_args::<syn::Type>() {
                        meta.error_ty = Some(ty);
                    }
                }
                _ => {}
            }
        } else {
            retained.push(attr);
        }
    }
    *attrs = retained;
}

/// Converts PascalCase to snake_case.
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.extend(ch.to_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}

/// Helper that builds the (impl_generics, (Ctx, Intent, Outcome, E)) tuple for a trait implementation.
fn build_generics(
    orig_generics: &syn::Generics,
    meta: &NodeMetaArgs,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    let mut impl_generics = orig_generics.clone();

    let ctx_tok = if let Some(ty) = &meta.context_ty {
        quote!(#ty)
    } else {
        let p: GenericParam = parse_quote!(Ctx);
        impl_generics.params.push(p);
        quote!(Ctx)
    };

    let intent_tok = if let Some(ty) = &meta.intent_ty {
        quote!(#ty)
    } else {
        let p: GenericParam = parse_quote!(Intent);
        impl_generics.params.push(p);
        quote!(Intent)
    };

    let outcome_tok = if let Some(ty) = &meta.outcome_ty {
        quote!(#ty)
    } else {
        let p: GenericParam = parse_quote!(Outcome);
        impl_generics.params.push(p);
        quote!(Outcome)
    };

    let err_tok = if let Some(ty) = &meta.error_ty {
        quote!(#ty)
    } else {
        let p: GenericParam = parse_quote!(E);
        impl_generics.params.push(p);
        quote!(E)
    };

    let (impl_gen, _, _) = impl_generics.split_for_impl();
    (quote!(#impl_gen), ctx_tok, intent_tok, outcome_tok, err_tok)
}

/// Declares a module node on a struct.
///
/// Automatically mounts child fields implementing `IntoTreeNode`.
#[proc_macro_attribute]
pub fn module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut meta = parse_macro_input!(attr as NodeMetaArgs);
    let mut input = parse_macro_input!(item as ItemStruct);

    extract_sibling_attributes(&mut input.attrs, &mut meta);

    let struct_ident = &input.ident;
    let node_name = meta
        .name
        .clone()
        .unwrap_or_else(|| to_snake_case(&struct_ident.to_string()));

    let enter_setup = meta
        .enter
        .as_ref()
        .map(|path| quote! { module = module.with_enter(#path); });
    let exit_setup = meta
        .exit
        .as_ref()
        .map(|path| quote! { module = module.with_exit(#path); });

    let (impl_gen, ctx_ty, intent_ty, outcome_ty, err_ty) = build_generics(&input.generics, &meta);
    let (_, ty_gen, where_clause) = input.generics.split_for_impl();

    let field_mounts = match &input.fields {
        syn::Fields::Named(named) => {
            let field_idents: Vec<_> = named.named.iter().map(|f| &f.ident).collect();
            quote! {
                #(
                    module.mount(self.#field_idents);
                )*
            }
        }
        syn::Fields::Unit => quote! {},
        syn::Fields::Unnamed(unnamed) => {
            let indices: Vec<_> = (0..unnamed.unnamed.len()).map(syn::Index::from).collect();
            quote! {
                #(
                    module.mount(self.#indices);
                )*
            }
        }
    };

    let field_predicates = match &input.fields {
        syn::Fields::Named(named) => {
            let tys: Vec<_> = named.named.iter().map(|f| &f.ty).collect();
            quote! {
                #(
                    #tys: ::modtree::tree::IntoTreeNode<#ctx_ty, #intent_ty, #outcome_ty, #err_ty>,
                )*
            }
        }
        syn::Fields::Unit => quote! {},
        syn::Fields::Unnamed(unnamed) => {
            let tys: Vec<_> = unnamed.unnamed.iter().map(|f| &f.ty).collect();
            quote! {
                #(
                    #tys: ::modtree::tree::IntoTreeNode<#ctx_ty, #intent_ty, #outcome_ty, #err_ty>,
                )*
            }
        }
    };

    let expanded = quote! {
        #input

        #[automatically_derived]
        impl #impl_gen ::modtree::tree::IntoTreeNode<#ctx_ty, #intent_ty, #outcome_ty, #err_ty> for #struct_ident #ty_gen
        #where_clause
        where
            #field_predicates
        {
            fn into_tree_node(self) -> ::modtree::tree::TreeNode<#ctx_ty, #intent_ty, #outcome_ty, #err_ty> {
                let mut module = ::modtree::tree::ModuleNode::new(#node_name);
                #enter_setup
                #exit_setup
                #field_mounts
                ::modtree::tree::TreeNode::Module(module)
            }
        }
    };

    TokenStream::from(expanded)
}

/// Declares a library node on a struct or `impl` block.
///
/// When declared on an `impl` block, automatically extracts methods marked with
/// `#[command]` and builds a `LibraryNode` containing strictly `LeafNode` instances.
#[proc_macro_attribute]
pub fn library(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut meta = parse_macro_input!(attr as NodeMetaArgs);

    if let Ok(mut item_struct) = syn::parse::<ItemStruct>(item.clone()) {
        extract_sibling_attributes(&mut item_struct.attrs, &mut meta);
        let struct_ident = &item_struct.ident;
        let node_name = meta
            .name
            .clone()
            .unwrap_or_else(|| to_snake_case(&struct_ident.to_string()));

        let enter_setup = meta
            .enter
            .as_ref()
            .map(|path| quote! { lib = lib.with_enter(#path); });
        let exit_setup = meta
            .exit
            .as_ref()
            .map(|path| quote! { lib = lib.with_exit(#path); });

        let (impl_gen, ctx_ty, intent_ty, outcome_ty, err_ty) =
            build_generics(&item_struct.generics, &meta);
        let (_, ty_gen, where_clause) = item_struct.generics.split_for_impl();

        let expanded = quote! {
            #item_struct

            #[automatically_derived]
            impl #impl_gen ::modtree::tree::IntoTreeNode<#ctx_ty, #intent_ty, #outcome_ty, #err_ty> for #struct_ident #ty_gen
            #where_clause
            {
                fn into_tree_node(self) -> ::modtree::tree::TreeNode<#ctx_ty, #intent_ty, #outcome_ty, #err_ty> {
                    let mut lib = ::modtree::tree::LibraryNode::new(#node_name);
                    #enter_setup
                    #exit_setup
                    ::modtree::tree::TreeNode::Library(lib)
                }
            }
        };
        return TokenStream::from(expanded);
    }

    let mut item_impl = parse_macro_input!(item as ItemImpl);
    extract_sibling_attributes(&mut item_impl.attrs, &mut meta);

    let self_ty = &item_impl.self_ty;
    let fallback_name = quote!(#self_ty).to_string();
    let node_name = meta
        .name
        .clone()
        .unwrap_or_else(|| to_snake_case(fallback_name.trim_matches('&')));

    let enter_setup = meta
        .enter
        .as_ref()
        .map(|path| quote! { lib = lib.with_enter(#path); });
    let exit_setup = meta
        .exit
        .as_ref()
        .map(|path| quote! { lib = lib.with_exit(#path); });

    let mut command_registrations = Vec::new();

    for impl_item in &mut item_impl.items {
        if let syn::ImplItem::Fn(method) = impl_item {
            let mut cmd_meta = NodeMetaArgs::default();
            let mut is_command = false;
            let mut remaining_attrs = Vec::new();

            for attr in method.attrs.drain(..) {
                let segs: Vec<_> = attr
                    .path()
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect();
                let matches_cmd = if segs.len() == 1 {
                    segs[0] == "command"
                } else if segs.len() == 2 && segs[0] == "modtree" {
                    segs[1] == "command"
                } else {
                    false
                };

                if matches_cmd {
                    is_command = true;
                    if let Ok(parsed) = attr.parse_args::<NodeMetaArgs>() {
                        cmd_meta = parsed;
                    }
                } else {
                    remaining_attrs.push(attr);
                }
            }

            extract_sibling_attributes(&mut remaining_attrs, &mut cmd_meta);
            method.attrs = remaining_attrs;

            if is_command {
                let method_ident = &method.sig.ident;
                let cmd_name = cmd_meta.name.unwrap_or_else(|| method_ident.to_string());

                let cmd_enter = cmd_meta
                    .enter
                    .map(|path| quote! { leaf = leaf.with_enter(#path); });
                let cmd_exit = cmd_meta
                    .exit
                    .map(|path| quote! { leaf = leaf.with_exit(#path); });

                command_registrations.push(quote! {
                    {
                        let mut leaf = ::modtree::tree::LeafNode::new(#cmd_name, Self::#method_ident);
                        #cmd_enter
                        #cmd_exit
                        lib.add_leaf(leaf);
                    }
                });
            }
        }
    }

    let (impl_gen, ctx_ty, intent_ty, outcome_ty, err_ty) =
        build_generics(&item_impl.generics, &meta);
    let (_, _, where_clause) = item_impl.generics.split_for_impl();

    let expanded = quote! {
        #item_impl

        #[automatically_derived]
        impl #impl_gen ::modtree::tree::IntoTreeNode<#ctx_ty, #intent_ty, #outcome_ty, #err_ty> for #self_ty
        #where_clause
        {
            fn into_tree_node(self) -> ::modtree::tree::TreeNode<#ctx_ty, #intent_ty, #outcome_ty, #err_ty> {
                let mut lib = ::modtree::tree::LibraryNode::new(#node_name);
                #enter_setup
                #exit_setup
                #(
                    #command_registrations
                )*
                ::modtree::tree::TreeNode::Library(lib)
            }
        }
    };

    TokenStream::from(expanded)
}

/// Marks a method or function as an executable command leaf.
#[proc_macro_attribute]
pub fn command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Granular attribute specifying the path segment name of a node.
#[proc_macro_attribute]
pub fn name(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Granular attribute specifying the descent `on_enter` hook function path.
#[proc_macro_attribute]
pub fn enter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Granular attribute specifying the ascent `on_exit` hook function path.
#[proc_macro_attribute]
pub fn exit(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Granular attribute specifying the execution Context type.
#[proc_macro_attribute]
pub fn context(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Granular attribute specifying the Intent type.
#[proc_macro_attribute]
pub fn intent(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Granular attribute specifying the Outcome type.
#[proc_macro_attribute]
pub fn outcome(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Granular attribute specifying the Error type.
#[proc_macro_attribute]
pub fn error(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
