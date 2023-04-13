use proc_macro2::{TokenStream,  Ident};
use quote::{quote, quote_spanned};
use syn::{Data, spanned::Spanned};

#[proc_macro_derive(PacketEnumHolder)]
pub fn packet_enum_holder_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    let tokens = impl_peh(&ast);
    tokens
}

fn impl_peh(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    
    let enum_name = &ast.ident;
    let data = &ast.data;
    
    let bytes_to_packet = gen_bytes_to_packet(data, enum_name);
    let packet_to_bytes = gen_packet_to_bytes(data, enum_name);

    let gen = quote! {
        // use orange_networking::packet::PacketParseable;
        impl PacketEnumHolder for #enum_name {
            fn bytes_to_packet(bytes: &[u8]) -> Result<(#enum_name, usize), orange_networking::packet::PacketParseError> {
                #bytes_to_packet
            }
            fn packet_to_bytes(packet: #enum_name) -> Vec<u8> {
                #packet_to_bytes
            }
        }

    };
    gen.into()
}

fn gen_bytes_to_packet(data: &Data, enum_name: &Ident) -> TokenStream {
    match data {
        Data::Struct(..) => { quote!() },
        Data::Union(..) => { quote!() },
        Data::Enum(enum_data) => {
            let recurse = enum_data.variants.iter().map(|var| {
                let discriminant = &var.discriminant;
                let discriminant_value = if let Some((_, expr)) = discriminant {
                    match expr {
                        syn::Expr::Lit(lit) => {
                            match &lit.lit {
                                LitInt => { quote!(#lit) }
                            }
                        },
                        _ => {
                            unimplemented!("Can only use literal numbers");
                        }
                    }
                } else {
                    unimplemented!("Must specify an id: varient {{data}} = id");
                };
                let name = &var.ident;
                let fields = &var.fields;
                let field_recurse = fields.iter().map(|field| {
                    let field_ident = &field.ident;
                    let field_type = &field.ty;
                    let qt = quote_spanned!(
                        field.span() => 
                        match #field_type::from_packet_bytes(&bytes[start..]) {
                            Ok((value, consumed)) => { start += consumed; value },
                            Err(e) => { return Err(e); },
                        }
                        );
                    quote_spanned!(field.span() => #field_ident: #qt,)
                });
                let variant_quote = quote!( #enum_name::#name { #(#field_recurse)* } );
                quote_spanned!(var.span() => #discriminant_value => { return Ok((#variant_quote, start)); }, )
            });

            quote!(
                let mut start = 0usize;
                let id = match u8::from_packet_bytes(&bytes[start..]) {
                    Ok((value, consumed)) => { start += consumed; value },
                    Err(e) => { return Err(orange_networking::packet::PacketParseError::NotEnoughData); },
                };
                match id {
                    #(#recurse)*
                    _ => { return Err(orange_networking::packet::PacketParseError::NotAPacket); }, 
                }
            )
        }
    }
}

fn gen_packet_to_bytes(data: &Data, enum_name: &Ident) -> TokenStream {
    match data {
        Data::Struct(..) => { quote!() },
        Data::Union(..) => { quote!() },
        Data::Enum(enum_data) => {
            let recurse = enum_data.variants.iter().map(|var| {
                let discriminant = &var.discriminant;
                let discriminant_value = if let Some((_, expr)) = discriminant {
                    match expr {
                        syn::Expr::Lit(lit) => {
                            match &lit.lit {
                                LitInt => { quote!(#lit) }
                            }
                        },
                        _ => {
                            unimplemented!("Can only use literal numbers");
                        }
                    }
                } else {
                    unimplemented!("Must specify an id: varient {{data}} = id");
                };
                let name = &var.ident;
                let fields = &var.fields;
                let field_writes = fields.iter().map(|field| {
                    let field_ident = &field.ident;
                    let field_type = &field.ty; 
                    quote_spanned!(field.span() => #field_ident.to_packet_bytes(), )
                });
                let fields_names = fields.iter().map(|field| {
                    let field_ident = &field.ident;
                    quote_spanned!(field.span() => #field_ident, )
                });
                quote_spanned!(var.span() => #enum_name::#name { #(#fields_names )* } => { return [(#discriminant_value as u8).to_packet_bytes(), #(#field_writes)* ].concat(); }, )
            });

            quote!(
                match packet {
                    #(#recurse)*
                }
            )
        }
    }
}
