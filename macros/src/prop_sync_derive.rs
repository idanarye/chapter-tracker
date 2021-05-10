use crate::util::{iter_attrs_parts, path_to_single_string};
use quote::quote;
use syn::parse::Error;

pub fn impl_prop_sync_derive(ast: &syn::DeriveInput) -> Result<proc_macro2::TokenStream, Error> {
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(fields),
        ..
    }) = &ast.data
    {
        fields
    } else {
        return Err(Error::new_spanned(
            ast,
            "ProcSync only supports structs with named fields",
        ));
    };
    let mut fields_to_sync = Vec::new();
    for field in fields.named.iter() {
        let mut getter = false;
        let mut setter = false;
        let mut field_property = None;
        iter_attrs_parts(&field.attrs, "prop_sync", |expr| {
            match expr {
                syn::Expr::Path(path) => {
                    match path_to_single_string(&path.path)?.as_str() {
                        "get" => {
                            getter = true;
                        }
                        "set" => {
                            setter = true;
                        }
                        _ => {
                            return Err(Error::new_spanned(path, "unknown attribute"));
                        }
                    }
                }
                syn::Expr::Type(syn::ExprType {expr, ty, ..}) => {
                    if let syn::Expr::Lit(syn::ExprLit {
                        attrs: _,
                        lit: syn::Lit::Str(property),
                    }) = *expr {
                        field_property = Some((property, *ty));
                    } else {
                        return Err(Error::new_spanned(expr, "expected a string literal (representing a GTK property)"));
                    }
                }
                _ => {
                    return Err(Error::new_spanned(expr, "Expected (<...>=<...>)"));
                }
            }
            Ok(())
        })?;
        if getter || setter {
            fields_to_sync.push(FieldToSync {
                ident: field.ident.as_ref().unwrap(),
                ty: &field.ty,
                property: field_property,
                getter,
                setter
            });
        }
    }
    let setter = gen_setter(ast, &fields_to_sync)?;
    let getter = gen_getter(ast, &fields_to_sync)?;
    // eprintln!("{}", getter);
    Ok(quote! {
        #setter
        #getter
    })
}

#[derive(Debug)]
struct FieldToSync<'a> {
    ident: &'a syn::Ident,
    ty: &'a syn::Type,
    property: Option<(syn::LitStr, syn::Type)>,
    getter: bool,
    setter: bool,
}

fn gen_setter(ast: &syn::DeriveInput, fields: &[FieldToSync]) -> Result<proc_macro2::TokenStream, Error> {
    if !fields.iter().any(|f| f.setter) {
        return Ok(quote!());
    }

    let struct_name = &ast.ident;
    let setter_name = format!("{}PropSetter", struct_name);
    let setter_name = syn::Ident::new(&setter_name, ast.ident.span());
    let vis = &ast.vis;

    let mut lifetime = None;

    let mut struct_fields = Vec::new();
    let mut prop_assignment = Vec::new();

    for field in fields.iter() {
        if !field.setter {
            continue;
        }
        let ident = field.ident;
        if let Some((prop, ty)) = &field.property {
            if let syn::Type::Reference(ty_ref) = ty {
                let mut ty_ref = ty_ref.clone();
                ty_ref.lifetime = Some(lifetime.get_or_insert_with(|| syn::Lifetime::new("'a", proc_macro2::Span::call_site())).clone());
                struct_fields.push(quote! {
                    #ident: #ty_ref
                });
            } else {
                struct_fields.push(quote! {
                    #ident: #ty
                });
            }
            prop_assignment.push(quote! {
                self.#ident.set_property(#prop, &setter.#ident).unwrap();
            });
        } else {
            let ty = field.ty;
            let lifetime = lifetime.get_or_insert_with(|| syn::Lifetime::new("'a", proc_macro2::Span::call_site()));
            let as_trait = quote! {
                <#ty as crate::util::prop_sync::PropSyncWidgetDefaultProp<#lifetime>>
            };
            struct_fields.push(quote! {
                #ident: #as_trait::SetType
            });
            prop_assignment.push(quote! {
                self.#ident.set_property(#as_trait::PROP_NAME, &setter.#ident).unwrap();
            });
        }
    }

    Ok(quote! {
        #vis struct #setter_name <#lifetime> {
            #(#struct_fields),*
        }

        impl #struct_name {
            #vis fn set_data<#lifetime>(&self, setter: &#lifetime #setter_name<#lifetime>) {
                #(#prop_assignment)*
            }
        }
    })
}

fn gen_getter(ast: &syn::DeriveInput, fields: &[FieldToSync]) -> Result<proc_macro2::TokenStream, Error> {
    if !fields.iter().any(|f| f.setter) {
        return Ok(quote!());
    }

    let struct_name = &ast.ident;
    let setter_name = format!("{}PropGetter", struct_name);
    let setter_name = syn::Ident::new(&setter_name, ast.ident.span());
    let vis = &ast.vis;

    let mut struct_fields = Vec::new();
    let mut field_from_prop = Vec::new();

    for field in fields.iter() {
        if !field.setter {
            continue;
        }
        let ident = field.ident;
        if let Some((prop, ty)) = &field.property {
            if let syn::Type::Reference(ty_ref) = ty {
                let ty = &ty_ref.elem;
                struct_fields.push(quote! {
                    #ident: <#ty as std::borrow::ToOwned>::Owned
                });
            } else {
                struct_fields.push(quote! {
                    #ident: #ty
                });
            }
            field_from_prop.push(quote! {
                #ident: self.#ident.get_property(#prop).unwrap().get().unwrap().unwrap()
            });
        } else {
            let ty = field.ty;
            let as_trait = quote! {
                <#ty as crate::util::prop_sync::PropSyncWidgetDefaultProp<'static>>
            };
            struct_fields.push(quote! {
                #ident: #as_trait::GetType
            });
            field_from_prop.push(quote! {
                #ident: self.#ident.get_property(#as_trait::PROP_NAME).unwrap().get().unwrap().unwrap()
            });
        }
    }

    Ok(quote! {
        #vis struct #setter_name {
            #(#struct_fields),*
        }

        impl #struct_name {
            #vis fn get_data(&self) -> #setter_name {
                #setter_name {
                    #(#field_from_prop),*
                }
            }
        }
    })
}
