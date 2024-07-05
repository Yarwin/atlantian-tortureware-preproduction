extern crate proc_macro;

use quote::{quote};
use syn;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(ActionDispatchEnum, attributes(implementation))]
pub fn static_enum_dispatch(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    static_enum_dispatch2(input.into()).into()
}


fn static_enum_dispatch2(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let cloned_input = input.clone();
    let DeriveInput {
        ident: enum_name_ident,
        data,
        ..
    } = parse_macro_input!(cloned_input as DeriveInput);
    let (effects, preconditions, execute_action, finish, is_complete, is_interruptible, check_precondtions, cost) = {
        if let syn::Data::Enum(e) = data {
            (
                generate_description_for_enum(&e, "get_effects", quote! {action_arguments}, &enum_name_ident),
                generate_description_for_enum(&e, "get_preconditions", quote! {}, &enum_name_ident),
                generate_description_for_enum(&e, "execute_action", quote! {action_arguments}, &enum_name_ident),
                generate_description_for_enum(&e, "finish", quote! {action_arguments}, &enum_name_ident),
                generate_description_for_enum(&e, "is_action_complete", quote! {action_arguments}, &enum_name_ident),
                generate_description_for_enum(&e, "is_action_interruptible", quote! {action_arguments}, &enum_name_ident),
                generate_description_for_enum(&e, "check_procedural_preconditions", quote! {action_arguments}, &enum_name_ident),
                generate_description_for_enum(&e, "get_cost", quote! {action_arguments}, &enum_name_ident),
            )
        } else {
            panic!("")
        }
    };
    let expanded = quote! {
        impl ActionBehavior for #enum_name_ident {
            fn get_effects<'a, 'b: 'a>(&'a self, action_arguments: &'b AgentActionPlanContext) -> &'a WorldState {
                match self {
                    #effects
                }
            }
            fn get_preconditions(&self) -> &WorldState {
                match self {
                    #preconditions
                }
            }
            fn execute_action(&self, action_arguments: AgentActionWorldContext) {
                match self {
                    #execute_action
                }
            }
            fn finish(&self, action_arguments: AgentActionWorldContext) {
                match self {
                    #finish
                }
            }
            fn is_action_complete(&self, action_arguments: &AgentActionWorldContext) -> bool {
                match self {
                    #is_complete
                }
            }
            fn is_action_interruptible(&self, action_arguments: &AgentActionWorldContext) -> bool {
                match self {
                    #is_interruptible
                }
            }
            fn check_procedural_preconditions(&self, action_arguments: &AgentActionPlanContext) -> bool {
                match self {
                    #check_precondtions
                }
            }
            fn get_cost(&self, action_arguments: &AgentActionPlanContext) -> u32 {
                match self {
                    #cost
                }
            }

        }
    };
    expanded.into()
}


fn generate_description_for_enum(e: &syn::DataEnum, method_name: &str, args: proc_macro2::TokenStream, enum_name_ident: &proc_macro2::Ident) -> proc_macro2::TokenStream {
    e
        .variants
        .iter()
        .filter_map(|variant| {
            let field_name = &variant.ident;
            if variant.attrs.len() == 0 {
                return None;
            }
            let att = &variant.attrs[0];
            if let Ok(syn::Meta::NameValue(nv)) = att.parse_meta() {
                if let syn::Lit::Str(s) = nv.lit {
                    let arg: proc_macro2::TokenStream = s.value().parse().unwrap();
                    let method_declaration: proc_macro2::TokenStream = method_name.parse().unwrap();
                    if args.is_empty() {
                        return Some( quote! {
                        #enum_name_ident::#field_name(inner) => #arg::#method_declaration(inner),
                    });
                    }
                    return Some( quote! {
                        #enum_name_ident::#field_name(inner) => #arg::#method_declaration(inner, #args),
                    });
                }
            }
            None
        }).collect()
}