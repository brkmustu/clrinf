//! Compile-time CRUD DTO, entity wiring, and route metadata macro for Rust.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, LitStr};

const CANONICAL_OPS: &[&str] = &["create", "get_by_id", "get_list", "update", "delete"];

fn canonical_op(raw: &str) -> Option<&'static str> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "create" => Some("create"),
        "get_by_id" | "getbyid" => Some("get_by_id"),
        "get_list" | "getlist" => Some("get_list"),
        "update" => Some("update"),
        "delete" => Some("delete"),
        _ => None,
    }
}

fn to_snake_case(pascal: &str) -> String {
    let mut out = String::new();
    for (i, ch) in pascal.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            for lc in ch.to_lowercase() {
                out.push(lc);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn pluralize_snake(pascal: &str) -> String {
    let snake = to_snake_case(pascal);
    if snake.ends_with('s') {
        format!("{snake}es")
    } else if let Some(stem) = snake.strip_suffix('y') {
        if !stem.ends_with(['a', 'e', 'i', 'o', 'u']) {
            return format!("{stem}ies");
        }
        format!("{snake}s")
    } else {
        format!("{snake}s")
    }
}

#[proc_macro_derive(ClrinfCrud, attributes(clrinf_crud, crud_ignore))]
pub fn derive_clrinf_crud(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &input.ident;
    let ident_str = ident.to_string();

    let Data::Struct(ref data_struct) = input.data else {
        return Err(syn::Error::new_spanned(
            ident,
            "#[derive(ClrinfCrud)] only supports structs",
        ));
    };
    let Fields::Named(ref named_fields) = data_struct.fields else {
        return Err(syn::Error::new_spanned(
            ident,
            "#[derive(ClrinfCrud)] requires named struct fields",
        ));
    };

    let mut route: Option<String> = None;
    let mut id_name = "id".to_string();
    let mut tenant_name = "tenant_id".to_string();
    let mut skipped: Vec<&'static str> = Vec::new();

    for attr in &input.attrs {
        if !attr.path().is_ident("clrinf_crud") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("route") {
                let s: LitStr = meta.value()?.parse()?;
                route = Some(s.value());
                Ok(())
            } else if meta.path.is_ident("id") {
                let s: LitStr = meta.value()?.parse()?;
                id_name = s.value();
                Ok(())
            } else if meta.path.is_ident("tenant") {
                let s: LitStr = meta.value()?.parse()?;
                tenant_name = s.value();
                Ok(())
            } else if meta.path.is_ident("skip") {
                meta.parse_nested_meta(|nested| {
                    let Some(op_ident) = nested.path.get_ident() else {
                        return Err(nested.error("expected operation identifier inside skip(...)"));
                    };
                    let raw = op_ident.to_string();
                    let Some(canon) = canonical_op(&raw) else {
                        return Err(nested.error(format!(
                            "unknown CRUD operation '{raw}'; expected create, get_by_id, get_list, update, or delete"
                        )));
                    };
                    if !skipped.contains(&canon) {
                        skipped.push(canon);
                    }
                    Ok(())
                })
            } else {
                Err(meta.error("unknown #[clrinf_crud] key; expected route, id, tenant, or skip(...)"))
            }
        })?;
    }

    let route_prefix = route.unwrap_or_else(|| format!("/api/{}", pluralize_snake(&ident_str)));

    let mut id_field: Option<&syn::Field> = None;
    let mut tenant_field: Option<&syn::Field> = None;
    let mut dto_fields: Vec<&syn::Field> = Vec::new();
    let mut ignored_fields: Vec<&syn::Field> = Vec::new();

    for field in &named_fields.named {
        let fname = field
            .ident
            .as_ref()
            .expect("named field always has ident")
            .to_string();
        if fname == id_name {
            id_field = Some(field);
        } else if fname == tenant_name {
            tenant_field = Some(field);
        } else if field.attrs.iter().any(|a| a.path().is_ident("crud_ignore")) {
            ignored_fields.push(field);
        } else {
            dto_fields.push(field);
        }
    }

    let Some(id_f) = id_field else {
        return Err(syn::Error::new_spanned(
            ident,
            format!("Struct '{ident_str}' is missing required ID field '{id_name}'"),
        ));
    };
    let Some(tenant_f) = tenant_field else {
        return Err(syn::Error::new_spanned(
            ident,
            format!("Struct '{ident_str}' is missing required tenant field '{tenant_name}'"),
        ));
    };

    let id_ident = id_f.ident.as_ref().unwrap();
    let id_ty = &id_f.ty;
    let tenant_ident = tenant_f.ident.as_ref().unwrap();

    let create_dto_ident = format_ident!("Create{}Request", ident);
    let update_dto_ident = format_ident!("Update{}Request", ident);

    let dto_struct_fields = dto_fields.iter().map(|f| {
        let fi = f.ident.as_ref().unwrap();
        let ft = &f.ty;
        quote! { pub #fi: #ft }
    });
    let dto_struct_fields_u = dto_struct_fields.clone();

    let dto_param = if dto_fields.is_empty() {
        quote! { _dto }
    } else {
        quote! { dto }
    };

    let create_assigns = dto_fields.iter().map(|f| {
        let fi = f.ident.as_ref().unwrap();
        quote! { #fi: dto.#fi }
    });
    let ignored_assigns = ignored_fields.iter().map(|f| {
        let fi = f.ident.as_ref().unwrap();
        quote! { #fi: ::core::default::Default::default() }
    });
    let update_assigns = dto_fields.iter().map(|f| {
        let fi = f.ident.as_ref().unwrap();
        quote! { self.#fi = dto.#fi; }
    });

    let active_ops: Vec<&'static str> = CANONICAL_OPS
        .iter()
        .copied()
        .filter(|op| !skipped.contains(op))
        .collect();

    Ok(quote! {
        #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
        pub struct #create_dto_ident {
            #(#dto_struct_fields,)*
        }

        #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
        pub struct #update_dto_ident {
            #(#dto_struct_fields_u,)*
        }

        impl ::clrinf_core::crud::CrudEntity for #ident {
            type Id = #id_ty;

            fn id(&self) -> Self::Id {
                self.#id_ident.clone()
            }

            fn set_id(&mut self, id: Self::Id) {
                self.#id_ident = id;
            }

            fn tenant_id(&self) -> &str {
                self.#tenant_ident.as_str()
            }

            fn set_tenant_id(&mut self, tenant_id: String) {
                self.#tenant_ident = tenant_id;
            }
        }

        impl ::clrinf_core::crud::CrudMetadata for #ident {
            type CreateDto = #create_dto_ident;
            type UpdateDto = #update_dto_ident;

            const ENTITY_NAME: &'static str = #ident_str;
            const ROUTE_PREFIX: &'static str = #route_prefix;
            const OPERATIONS: &'static [&'static str] = &[#(#active_ops),*];
            const SKIPPED_OPERATIONS: &'static [&'static str] = &[#(#skipped),*];

            fn from_create(id: Self::Id, tenant_id: String, #dto_param: Self::CreateDto) -> Self {
                Self {
                    #id_ident: id,
                    #tenant_ident: tenant_id,
                    #(#create_assigns,)*
                    #(#ignored_assigns,)*
                }
            }

            fn apply_update(&mut self, #dto_param: Self::UpdateDto) {
                #(#update_assigns)*
            }
        }
    })
}
