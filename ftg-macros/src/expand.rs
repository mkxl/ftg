use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{Data, DeriveInput, Fields, FieldsUnnamed};

pub fn expand(item: TokenStream) -> TokenStream {
    let mut derive_input = syn::parse_macro_input!(item as DeriveInput);
    let Data::Enum(data_enum) = &mut derive_input.data else {
        return derive_input.into_token_stream().into();
    };
    let struct_attributes = &derive_input.attrs;
    let struct_visibility = &derive_input.vis;
    let mut structs = std::vec![];

    for variant in data_enum.variants.iter_mut() {
        // NOTE: we parse FieldsUnnamed from quote::quote! { ... } rather than constructing it directly because to
        // construct it directly requires handling/passing Spans which did not seem worth it
        let Fields::Named(_fields_named) = &variant.fields else {
            continue;
        };
        let struct_ident = &variant.ident;
        let fields_unnamed = quote::quote! { (#struct_ident) }.into_token_stream();
        let fields_unnamed = syn::parse2::<FieldsUnnamed>(fields_unnamed).unwrap();
        let struct_fields = std::mem::replace(&mut variant.fields, Fields::Unnamed(fields_unnamed));
        let structs_item = quote::quote! {
            #([#struct_attributes])*
            #struct_visibility struct #struct_ident {
                #struct_fields
            }
        };

        structs.push(structs_item);
    }

    quote::quote! {
        #(#structs)*
        #derive_input
    }
    .into()
}
