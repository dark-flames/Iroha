use proc_macro2::{TokenStream, Punct, Spacing, Span};
use syn::{DeriveInput, Error, Data, Fields, Field as SynField, Ident, Type, Index};
use iroha_internal::get_wrapped_value;
use quote::{ToTokens, TokenStreamExt};

pub struct Interpolated(String);

impl ToTokens for Interpolated {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Punct::new('#', Spacing::Alone));
        tokens.append(Ident::new(self.0.as_str(), Span::call_site()));
    }
}

enum StructType {
    NoField,
    Tuple,
    Struct
}

impl StructType {
    pub fn get_params(&self, values: TokenStream) -> TokenStream {
        match self {
            StructType::NoField => TokenStream::new(),
            StructType::Tuple => quote::quote! {(#values)},
            StructType::Struct => quote::quote! {{#values}},
        }
    }
}

#[allow(dead_code)]
pub struct StructStructure {
    name: Ident,
    fields: Option<Vec<Field>>,
    mod_path: Option<TokenStream>,
    struct_type: StructType
}

impl StructStructure {
    pub fn from_ast(input: &DeriveInput, mod_path: Option<TokenStream>) -> Result<Self, Error> {
        let name = input.ident.clone();

        let data_struct = match &input.data {
            Data::Struct(data) => data,
            _ => unreachable!()
        };

        let fields = if let Fields::Unit = &data_struct.fields {
            None
        } else {
            Some(data_struct.fields.iter().enumerate().map(
                |(index, field)| Field::from_ast(field, index)
            ).collect::<Result<Vec<Field>, Error>>()?)
        };

        let struct_type = match &data_struct.fields {
            Fields::Unit => StructType::NoField,
            Fields::Unnamed(_) => StructType::Tuple,
            Fields::Named(_) => StructType::Struct
        };

        Ok(StructStructure {
            name,
            fields,
            mod_path,
            struct_type
        })
    }

    pub fn get_implement(self) -> TokenStream {
        let name = &self.name;
        let (
            field_idents,
            fn_new_params,
            temp_values
        ) = match &self.fields {
            Some(fields_vec) => (
                    fields_vec.iter().map(
                        |field| field.get_temp_value_ident()
                    ).collect::<Vec<Ident>>(),
                    fields_vec.iter().map(
                        |field| field.get_construct_param()
                    ).collect::<Vec<TokenStream>>(),
                    fields_vec.iter().map(
                        |field| field.temp_value_token_stream()
                    ).collect::<Vec<TokenStream>>()
                ),
            _ => (Vec::new(), Vec::new(), Vec::new())
        };

        let params = self.struct_type.get_params(
            quote::quote! {#(#field_idents,)*}
        );

        let construct_params: Vec<Interpolated> = field_idents.iter().map(
            |ident| {
                Interpolated(ident.to_string())
            }
        ).collect();

        let mod_path_token = self.mod_path.as_ref().map(
            |path| quote::quote! {#path::}
        ).unwrap_or(TokenStream::new());
        quote::quote! {
            impl #name {
                pub fn new(#(#fn_new_params),*) -> Self {
                    #name #params
                }
            }

            impl quote::ToTokens for #name {
                fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                    use iroha::Tokenizable;
                    #(#temp_values;)*
                    use quote::TokenStreamExt;
                    tokens.append(proc_macro2::Group::new(
                        proc_macro2::Delimiter::Brace,
                        quote::quote! {
                            #mod_path_token#name::new(#(#construct_params),*)
                        }
                    ))
                }
            }
        }
    }
}

#[allow(dead_code)]
struct Field {
    name: Option<Ident>,
    index: usize,
    ty: Type
}

impl Field {
    pub fn from_ast(field: &SynField, index: usize) -> Result<Self, Error> {
        let name = field.ident.clone();
        let ty = field.ty.clone();

        Ok(Field {
            name,
            index,
            ty
        })
    }

    fn get_ident(&self) -> TokenStream {
        if let Some(ident) = &self.name {
            quote::quote! {
                self.#ident
            }
        } else {
            let index = Index::from(self.index);
            quote::quote! {
                self.#index
            }
        }
    }

    pub fn get_construct_param(&self) -> TokenStream {
        let value = self.get_temp_value_ident();
        let ty = &self.ty;

        quote::quote! {
            #value: #ty
        }
    }

    pub fn get_temp_value_ident(&self) -> Ident {
        if let Some(ident) = &self.name {
            ident.clone()
        } else {
            quote::format_ident!("field_{}", self.index)
        }
    }

    pub fn temp_value_token_stream(&self) -> TokenStream {
        let temp_value_ident = self.get_temp_value_ident();
        let value = get_wrapped_value(&self.ty, self.get_ident());
        quote::quote! {
            let #temp_value_ident = #value
        }
    }
}
