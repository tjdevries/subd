use proc_macro::Span;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::FieldsNamed;
use syn::FnArg;
use syn::Ident;
use syn::ItemMod;
use syn::Pat;
use syn::PatIdent;
use syn::PatType;
use syn::Token;

// impl Parse for Item {
//     fn parse(input: ParseStream) -> Result<Self> {
//         let lookahead = input.lookahead1();
//         if lookahead.peek(Token![struct]) {
//             input.parse().map(Item::Struct)
//         } else if lookahead.peek(Token![enum]) {
//             input.parse().map(Item::Enum)
//         } else {
//             Err(lookahead.error())
//         }
//     }
// }

#[proc_macro_attribute]
pub fn database_model(_attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as ItemMod);
    let input_content =
        input.content.expect("Must have content inside of module").1;

    let name = input.ident;
    let mut models = vec![];
    let mut content = vec![];
    for item in input_content {
        match item {
            syn::Item::Struct(s) if s.ident == "Model" => models.push(s),
            _ => content.push(item),
        };
    }

    assert!(
        models.len() == 1,
        "must have one struct named Model in the mod"
    );
    let mut model = models.pop().unwrap();

    let mut primary_key = None;

    let mut model_update = model.clone();
    model_update.ident = Ident::new("ModelUpdate", Span::call_site().into());
    model_update.fields = match model_update.fields {
        syn::Fields::Named(fields) => {
            let mut named = Punctuated::new();
            fields.named.iter().for_each(|f| {
                if f.attrs
                    .iter()
                    .find(|a| {
                        a.path()
                            .segments
                            .iter()
                            .any(|s| s.ident == "primary_key")
                    })
                    .is_some()
                {
                    primary_key = Some(f.clone());
                    return;
                }

                if f.attrs
                    .iter()
                    .find(|a| {
                        a.path().segments.iter().any(|s| s.ident == "immutable")
                    })
                    .is_some()
                {
                    return;
                }

                let mut new_field = f.clone();
                new_field.vis = syn::Visibility::Public(syn::token::Pub {
                    span: Span::call_site().into(),
                });
                let ty = f.ty.clone();
                new_field.ty = syn::Type::Verbatim(
                    TokenStream::from(quote!(Option<#ty>)).into(),
                );

                named.push(new_field);
            });

            syn::Fields::Named(FieldsNamed {
                brace_token: fields.brace_token,
                named,
            })
        }
        _ => panic!("Only NamedFields are allowed"),
    };

    let mut new_args: Punctuated<FnArg, Token![,]> = Punctuated::new();
    model.fields.iter().for_each(|f| {
        if let Some(ident) = &f.ident {
            new_args.push(FnArg::Typed(PatType {
                attrs: vec![],
                pat: Box::new(Pat::Ident(PatIdent {
                    attrs: vec![],
                    by_ref: None,
                    mutability: None,
                    ident: ident.clone(),
                    subpat: None,
                })),
                colon_token: f.colon_token.unwrap(),
                ty: Box::new(f.ty.clone()),
            }))
        }
    });

    let self_body = model
        .fields
        .iter()
        .filter_map(|f| f.ident.clone())
        .collect::<Punctuated<Ident, Token![,]>>();

    // Remove attrs (todo, only immutable attrs)
    model.fields.iter_mut().for_each(|f| f.attrs = vec![]);

    let model_update_identifiers = model_update
        .fields
        .iter()
        .filter_map(|f| f.ident.clone())
        .map(|ident| {
            quote!(
                if let Some(#ident) = updates.#ident {
                    self.#ident = #ident
                }
            )
        })
        .collect::<Vec<_>>();

    let field_list = model
        .fields
        .iter()
        .filter_map(|f| f.ident.as_ref().map(|ident| ident.to_string()))
        .collect::<Vec<_>>()
        .join(", ");

    let read = match primary_key {
        Some(primary_key) => {
            let query = format!(
                "SELECT {} FROM {} WHERE {} = $1",
                field_list,
                name,
                primary_key.ident.unwrap(),
            );

            quote! {
                pub async fn read(
                    pool: &sqlx::PgPool,
                    id: sqlx::types::Uuid,
                ) -> Result<Option<Self>> {
                    Ok(sqlx::query_as!(
                        Self, #query, id
                    )
                    .fetch_optional(pool)
                    .await?)
                }
            }
        }
        None => quote!(),
    };
    // let primary_key = primary_key.expect("Model must have primary key");

    let vis = input.vis;
    TokenStream::from(quote! {
         #vis mod #name {
            #[derive(Debug, Default)]
            #model

            #[derive(Debug, Default)]
            #model_update

            #(#content)*

            // Should I pass in default here???
            impl Model {
                pub fn new(#new_args) -> Self {
                    Self { #self_body, ..Default::default() }
                }

                #read

                pub async fn update(mut self, pool: &sqlx::PgPool, updates: ModelUpdate) -> Result<Self> {
                    #(#model_update_identifiers)*
                    Ok(self.save(pool).await?)
                }
            }
        }
    })
}
