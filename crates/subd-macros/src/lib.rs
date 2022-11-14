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
use syn::VisPublic;
use syn::Visibility;

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
            syn::Item::Struct(s) if s.ident.to_string() == "Model" => {
                models.push(s)
            }
            _ => content.push(item),
        };
    }

    assert!(
        models.len() == 1,
        "must have one struct named Model in the mod"
    );
    let mut model = models.pop().unwrap();

    let mut model_update = model.clone();
    model_update.ident = Ident::new("ModelUpdate", Span::call_site().into());
    model_update.fields = match model_update.fields {
        syn::Fields::Named(fields) => {
            let mut named = Punctuated::new();
            fields.named.iter().for_each(|f| {
                if f.attrs
                    .iter()
                    .find(|a| {
                        a.path.segments.len() == 1
                            && a.path.segments[0].ident.to_string()
                                == "immutable"
                    })
                    .is_some()
                {
                    return;
                }

                let mut new_field = f.clone();
                new_field.vis = Visibility::Public(VisPublic {
                    pub_token: syn::token::Pub {
                        span: Span::call_site().into(),
                    },
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
    model.fields.iter().for_each(|f| match &f.ident {
        Some(ident) => new_args.push(FnArg::Typed(PatType {
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
        })),
        None => (),
    });

    let self_body = model
        .fields
        .iter()
        .filter_map(|f| match &f.ident {
            Some(ident) => Some(ident.clone()),
            None => None,
        })
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

    TokenStream::from(quote! {
        mod #name {
            #[derive(Debug, Default)]
            #model

            #[derive(Debug, Default)]
            #model_update

            #(#content)*

            impl Model {
                pub fn new(#new_args) -> Self {
                    Self { #self_body }
                }

                pub async fn update(mut self, conn: &mut SqliteConnection, updates: ModelUpdate) -> Result<Self> {
                    #(#model_update_identifiers)*
                    self.save(conn).await?;
                    Ok(self)
                }
            }
        }
    })
}
