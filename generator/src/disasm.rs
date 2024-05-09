use anyhow::{bail, Context, Result};
use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::{
    isa::{Field, Isa, Opcode},
    iter::cartesian,
    search::SearchTree,
    token::HexLiteral,
};

pub fn generate_disasm(isa: &Isa) -> Result<TokenStream> {
    // Generate opcode enum and mnemonics array
    let (opcode_enum_tokens, opcode_mnemonics_tokens, num_opcodes_token) = generate_opcode_tokens(&isa.opcodes);

    // Generate opcode search function
    let mut opcodes = isa.opcodes.to_vec();
    let tree = SearchTree::optimize(&opcodes, u32::MAX).unwrap();
    let body = generate_search_node(Some(Box::new(tree)), &mut opcodes);
    let opcode_find_tokens = quote! {
        #[inline]
        pub fn find(code: u32) -> Self {
            #body
            Opcode::Illegal
        }
    };

    // Generate field accessors
    let field_accessors_tokens = generate_field_accessors(isa)?;

    // Generate modifier case enums
    let case_enums_tokens = generate_modifier_case_enums(isa);

    // Generate modifier accessors
    let modifier_accessors_tokens = generate_modifier_accessors(isa)?;

    // Generate argument types
    let argument_enum_tokens = generate_argument_enums(isa)?;

    // Generate parse functions
    let max_args = isa.get_max_args()?;
    let parse_functions = generate_parse_functions(isa, max_args, &isa.opcodes, &num_opcodes_token)?;

    let max_args = Literal::usize_unsuffixed(max_args);
    Ok(quote! {
        #![cfg_attr(rustfmt, rustfmt_skip)]
        #![allow(unused)]
        #[comment = " Generated by armv5te-generator. Do not edit!"]

        use crate::disasm::{Ins, ParsedIns};

        #[doc = " These are the mnemonics of each opcode. Some mnemonics are duplicated due to them having multiple formats."]
        static OPCODE_MNEMONICS: [&str; #num_opcodes_token] = [#opcode_mnemonics_tokens];

        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        #[repr(u8)]
        #[non_exhaustive]
        pub enum Opcode {
            #[doc = " Illegal or unknown"]
            #[default]
            Illegal = u8::MAX,
            #opcode_enum_tokens
        }
        impl Opcode {
            #opcode_find_tokens
            pub fn mnemonic(self) -> &'static str {
                OPCODE_MNEMONICS[self as usize]
            }
            pub fn count() -> usize {
                #num_opcodes_token
            }
        }

        impl Ins {
            #field_accessors_tokens
            #modifier_accessors_tokens
        }

        #case_enums_tokens

        pub type Arguments = [Argument; #max_args];
        #argument_enum_tokens

        #parse_functions
    })
}

fn generate_search_node(node: Option<Box<SearchTree>>, opcodes: &mut Vec<Opcode>) -> TokenStream {
    if let Some(node) = node {
        let bitmask_token = HexLiteral(node.bitmask);
        let pattern_token = HexLiteral(node.left_pattern);

        let (mut left, mut right) = node.filter(opcodes);
        let left_node = generate_search_node(node.left, &mut left);
        let right_node = generate_search_node(node.right, &mut right);

        let body = quote! {
            if (code & #bitmask_token) == #pattern_token {
                #left_node
            } else #right_node
        };
        body
    } else {
        // When bitmask A is a subset of B, then B must be first, otherwise we might never choose B
        opcodes.sort_unstable_by_key(|op| 32 - op.bitmask.count_ones());
        let opcode_checks = opcodes.iter().map(|op| {
            let bitmask_token = HexLiteral(op.bitmask);
            let pattern_token = HexLiteral(op.pattern);
            let variant_token = Ident::new(&op.enum_name(), Span::call_site());
            quote! {
                if (code & #bitmask_token) == #pattern_token {
                    return Opcode::#variant_token;
                }
            }
        });
        quote! {
            #(#opcode_checks)else*
        }
    }
}

fn generate_parse_functions(
    isa: &Isa,
    max_args: usize,
    sorted_opcodes: &[Opcode],
    num_opcodes_token: &Literal,
) -> Result<TokenStream, anyhow::Error> {
    let mut parse_functions = TokenStream::new();
    for opcode in isa.opcodes.iter() {
        let modifiers = if let Some(modifiers) = &opcode.modifiers {
            let modifier_values: Result<Vec<_>> = modifiers.iter().map(|modifier| isa.get_modifier(modifier)).collect();
            modifier_values?
        } else {
            vec![]
        };
        let modifier_values: Result<Vec<_>> = modifiers
            .iter()
            .map(|modifier| {
                let accessor = Ident::new(&modifier.accessor_name(), Span::call_site());
                Ok(quote! { ins.#accessor() })
            })
            .collect();
        let modifier_values = modifier_values?;

        let opcode_args = opcode
            .args
            .as_ref()
            .map(|args| args.iter().map(|arg| isa.get_field(arg)).collect::<Result<Vec<_>>>())
            .unwrap_or(Ok(vec![]))?;

        let modifier_cases = opcode.get_modifier_cases(isa)?;
        let body = {
            let mut case_bodies: Vec<TokenStream> = vec![];
            if modifier_cases.is_empty() {
                let mnemonic = opcode.name().to_string();
                let args = generate_mnemonic_args(isa, opcode, max_args, opcode_args)?;
                quote! {
                    *out = ParsedIns {
                        mnemonic: #mnemonic,
                        args: [ #(#args),* ],
                    }
                }
            } else {
                for cases in cartesian(&modifier_cases) {
                    let mut case_values = cases.iter().zip(modifiers.iter()).map(|(case, modifier)| {
                        if modifier.pattern.is_some() {
                            if case.pattern != 0 {
                                quote! { true }
                            } else {
                                quote! { false }
                            }
                        } else {
                            let enum_name = Ident::new(&modifier.enum_name(), Span::call_site());
                            let variant_name = Ident::new(&case.variant_name(), Span::call_site());
                            quote! { #enum_name::#variant_name }
                        }
                    });
                    let suffix = cases
                        .iter()
                        .map(|case| case.suffix.clone().unwrap_or("".to_string()))
                        .collect::<String>();
                    let mnemonic = opcode.name().to_string() + &suffix;
                    let case_args = {
                        let mut case_args = opcode_args.clone();
                        for case in cases.iter() {
                            if let Some(args) = &case.args {
                                for arg in args.iter() {
                                    let arg = isa.get_field(arg)?;
                                    case_args.push(arg);
                                }
                            }
                        }
                        case_args
                    };

                    let args = generate_mnemonic_args(isa, opcode, max_args, case_args)?;
                    if case_values.len() > 1 {
                        case_bodies.push(quote! {
                            (#(#case_values),*) => ParsedIns {
                                mnemonic: #mnemonic,
                                args: [ #(#args),* ],
                            }
                        });
                    } else {
                        let case_value = case_values.next().unwrap();
                        case_bodies.push(quote! {
                            #case_value => ParsedIns {
                                mnemonic: #mnemonic,
                                args: [ #(#args),* ],
                            }
                        });
                    }
                }
                let illegal_args = (0..max_args).map(|_| quote! { Argument::None });
                let illegal_ins = quote! {
                    ParsedIns {
                        mnemonic: "<illegal>",
                        args: [
                            #(#illegal_args),*
                        ],
                    }
                };
                if modifier_values.len() > 1 {
                    quote! {
                        *out = match (#(#modifier_values),*) {
                            #(#case_bodies),*,
                            _ => #illegal_ins,
                        }
                    }
                } else {
                    let modifier_value = &modifier_values[0];
                    quote! {
                        *out = match #modifier_value {
                            #(#case_bodies),*,
                            _ => #illegal_ins,
                        }
                    }
                }
            }
        };

        let parse_fn = Ident::new(&opcode.parser_name(), Span::call_site());
        parse_functions.extend(quote! {
            fn #parse_fn(out: &mut ParsedIns, ins: Ins) {
                #body
            }
        })
    }
    let parser_fns = sorted_opcodes
        .iter()
        .map(|op| Ident::new(&op.parser_name(), Span::call_site()));
    parse_functions.extend(quote! {
        type MnemonicParser = fn(&mut ParsedIns, Ins);
        static MNEMONIC_PARSERS: [MnemonicParser; #num_opcodes_token] = [
            #(#parser_fns),*
        ];
        #[inline]
        pub fn parse(out: &mut ParsedIns, ins: Ins) {
            MNEMONIC_PARSERS[ins.op as usize](out, ins);
        }
    });
    Ok(parse_functions)
}

fn generate_mnemonic_args(isa: &Isa, opcode: &Opcode, max_args: usize, args: Vec<&Field>) -> Result<Vec<TokenStream>> {
    let args = (0..max_args)
        .map(|i| {
            if i < args.len() {
                let field = args[i];
                let accessor = Ident::new(&field.accessor_name(), Span::call_site());
                let arg = isa.get_arg(&field.arg)?;
                let arg_variant = Ident::new(&arg.variant_name(), Span::call_site());
                match (&arg.values, arg.signed, arg.boolean) {
                    (None, true, false) | (None, false, true) | (None, false, false) | (Some(_), false, false) => {
                        Ok(quote! { Argument::#arg_variant(ins.#accessor()) })
                    }
                    (Some(_), false, true) => bail!(
                        "Can't generate arg '{}' (for opcode '{}') which has values and is boolean",
                        arg.name,
                        opcode.name
                    ),
                    (Some(_), true, false) => bail!(
                        "Can't generate arg '{}' (for opcode '{}') which has values and is signed",
                        arg.name,
                        opcode.name
                    ),
                    (Some(_), true, true) => bail!(
                        "Can't generate arg '{}' (for opcode '{}') which has values and is signed and boolean",
                        arg.name,
                        opcode.name
                    ),
                    (None, true, true) => bail!(
                        "Can't generate arg '{}' (for opcode '{}') which is signed and boolean",
                        arg.name,
                        opcode.name
                    ),
                }
            } else {
                Ok(quote! { Argument::None })
            }
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(args)
}

fn generate_argument_enums(isa: &Isa) -> Result<TokenStream> {
    let mut argument_variants = TokenStream::new();
    let mut argument_sub_enum_tokens = TokenStream::new();
    for arg in isa.args.iter() {
        let variant_name = arg.variant_name();
        let variant_ident = Ident::new(&variant_name, Span::call_site());
        let doc = arg.doc();

        let contents = match (&arg.values, arg.signed, arg.boolean) {
            (None, true, false) => quote! { (i32) },
            (None, false, true) => quote! { (bool) },
            (None, false, false) => quote! { (u32) },
            (Some(values), false, false) => {
                // Create enum with the same name as the argument
                let mut sub_variants = TokenStream::new();
                let illegal_value = Literal::u8_unsuffixed(u8::MAX);
                sub_variants.extend(quote! {
                    Illegal = #illegal_value,
                });
                for value in values.iter() {
                    let sub_variant_name = value.variant_name();
                    let sub_variant_ident = Ident::new(&sub_variant_name, Span::call_site());
                    let sub_variant_value = Literal::u8_unsuffixed(value.value);
                    let sub_doc = value.doc();
                    let sub_doc = if !sub_doc.is_empty() {
                        quote! { #[doc = #sub_doc] }
                    } else {
                        quote! {}
                    };
                    sub_variants.extend(quote! {
                        #sub_doc
                        #sub_variant_ident = #sub_variant_value,
                    });
                }

                argument_sub_enum_tokens.extend(quote! {
                    #[derive(Clone, Copy, PartialEq, Eq)]
                    #[repr(u8)]
                    pub enum #variant_ident {
                        #sub_variants
                    }
                });

                if !arg.is_continuous() {
                    bail!("No support for discontinuous args currently")
                }
                let min_value = values.iter().min_by_key(|v| v.value).unwrap();
                let max_value = values.iter().max_by_key(|v| v.value).unwrap();
                let min_value_token = Literal::u8_unsuffixed(min_value.value);
                let max_value_token = Literal::u8_unsuffixed(max_value.value);
                let range_tokens = match (min_value.value == u8::MIN, max_value.value == u8::MAX) {
                    (true, true) => bail!("Arg value range spanning 0-255 not supported, for arg '{}'", arg.name),
                    (true, false) => quote! { value <= #max_value_token },
                    (false, true) => quote! { value >= #min_value_token},
                    (false, false) => quote! { value >= #min_value_token && value <= #max_value_token },
                };
                argument_sub_enum_tokens.extend(quote! {
                    impl #variant_ident {
                        pub const fn parse(value: u8) -> Self {
                            if #range_tokens {
                                unsafe { std::mem::transmute::<u8, Self>(value) }
                            } else {
                                Self::Illegal
                            }
                        }
                    }
                });

                quote! { (#variant_ident) }
            }

            (None, true, true) => bail!("Can't generate argument variant '{}' which is signed and boolean", arg.name),
            (Some(_), true, true) => bail!(
                "Can't generate argument variant '{}' which has values and is signed and boolean",
                arg.name
            ),
            (Some(_), true, false) => bail!(
                "Can't generate argument variant '{}' which has values and is signed",
                arg.name
            ),
            (Some(_), false, true) => bail!(
                "Can't generate argument variant '{}' which has values and is boolean",
                arg.name
            ),
        };

        argument_variants.extend(quote! {
            #[doc = #doc]
            #variant_ident #contents,
        })
    }
    let argument_enum_tokens = quote! {
        #[derive(Default, Clone, Copy, PartialEq, Eq)]
        pub enum Argument {
            #[default]
            None,
            #argument_variants
        }
        #argument_sub_enum_tokens
    };
    Ok(argument_enum_tokens)
}

fn generate_modifier_accessors(isa: &Isa) -> Result<TokenStream> {
    let mut modifier_accessors_tokens = TokenStream::new();
    for modifier in isa.modifiers.iter() {
        let (inner, ret_type) = match (modifier.bitmask, modifier.pattern, &modifier.cases) {
            (Some(bitmask), Some(pattern), None) => {
                let bitmask_token = HexLiteral(bitmask);
                let pattern_token = HexLiteral(pattern);
                (
                    quote! { (self.code & #bitmask_token) == #pattern_token },
                    Ident::new("bool", Span::call_site()),
                )
            }
            (bitmask, None, Some(cases)) => {
                let enum_name = modifier.enum_name();
                let enum_ident = Ident::new(&enum_name, Span::call_site());

                let sorted_cases = {
                    let mut sorted_cases = Vec::from(cases.clone());
                    // When bitmask A is a subset of B, then B must be first, otherwise we will never choose B
                    sorted_cases.sort_by_key(|case| 32 - case.bitmask.unwrap_or(0).count_ones());
                    sorted_cases
                };

                if let Some(bitmask) = bitmask {
                    let bitmask_token = HexLiteral(bitmask);
                    let mut match_tokens = TokenStream::new();
                    for case in sorted_cases.iter() {
                        let pattern_token = HexLiteral(case.pattern);
                        let variant_name = case.variant_name();
                        let variant_ident = Ident::new(&variant_name, Span::call_site());
                        match_tokens.extend(quote! {
                            #pattern_token => #enum_ident::#variant_ident,
                        });
                    }

                    (
                        quote! {
                            match self.code & #bitmask_token {
                                #match_tokens
                                _ => #enum_ident::Illegal,
                            }
                        },
                        enum_ident,
                    )
                } else {
                    let mut if_tokens = vec![];
                    for case in sorted_cases.iter() {
                        let bitmask = case.bitmask.with_context(|| {
                            format!("Modifier case '{}' in modifier '{}' has no bitmask", case.name, modifier.name)
                        })?;
                        let bitmask_token = HexLiteral(bitmask);
                        let pattern_token = HexLiteral(case.pattern);
                        let variant_name = case.variant_name();
                        let variant_ident = Ident::new(&variant_name, Span::call_site());
                        if_tokens.push(quote! {
                            if (self.code & #bitmask_token) == #pattern_token {
                                #enum_ident::#variant_ident
                            }
                        });
                    }

                    (
                        quote! {
                            #(#if_tokens)else*
                            else {
                                #enum_ident::Illegal
                            }
                        },
                        enum_ident,
                    )
                }
            }
            (None, Some(_), None) => bail!("Can't generate modifier accessor '{}' with only a pattern", modifier.name),
            (_, Some(_), Some(_)) => bail!(
                "Can't generate modifier accessor '{}' with a pattern and cases",
                modifier.name
            ),
            (Some(_), None, None) => bail!("Can't generate modifier accessor '{}' with only a bitmask", modifier.name),
            (None, None, None) => bail!(
                "Can't generate modifier accessor '{}' without a pattern, bitmask and/or cases",
                modifier.name
            ),
        };

        let doc = modifier.doc();
        let fn_name = Ident::new(&modifier.accessor_name(), Span::call_site());

        modifier_accessors_tokens.extend(quote! {
            #[doc = #doc]
            #[inline(always)]
            pub const fn #fn_name(&self) -> #ret_type {
                #inner
            }
        })
    }
    Ok(modifier_accessors_tokens)
}

fn generate_modifier_case_enums(isa: &Isa) -> TokenStream {
    let mut case_enums_tokens = TokenStream::new();
    for modifier in isa.modifiers.iter() {
        if let Some(cases) = &modifier.cases {
            let mut variants_tokens = TokenStream::new();
            for case in cases.iter() {
                let variant_name = case.variant_name();
                let variant_ident = Ident::new(&variant_name, Span::call_site());
                let doc = case.doc();
                variants_tokens.extend(quote! {
                    #[doc = #doc]
                    #variant_ident,
                });
            }
            let enum_name = modifier.enum_name();
            let enum_ident = Ident::new(&enum_name, Span::call_site());
            let doc = modifier.doc();
            case_enums_tokens.extend(quote! {
                #[doc = #doc]
                pub enum #enum_ident {
                    Illegal,
                    #variants_tokens
                }
            })
        }
    }
    case_enums_tokens
}

fn generate_field_accessors(isa: &Isa) -> Result<TokenStream> {
    let mut field_accessors_tokens = TokenStream::new();
    for field in isa.fields.iter() {
        let num_bits = field.bits.0.len();
        let shift = field.bits.0.start;
        let bitmask = HexLiteral(((1 << num_bits) - 1) << shift);
        let shift_token = Literal::u8_unsuffixed(shift);

        let body_tokens = if shift > 0 && num_bits > 1 {
            quote! { (self.code & #bitmask) >> #shift_token }
        } else {
            quote! { self.code & #bitmask }
        };

        let arg = isa.get_arg(&field.arg)?;
        let arg_ident = Ident::new(&arg.variant_name(), Span::call_site());

        let doc = field.doc();
        let fn_name = Ident::new(&field.accessor_name(), Span::call_site());
        let (ret_type, inner) = match (&arg.values, arg.signed, arg.boolean) {
            (None, true, false) => (quote! { i32 }, quote! { (#body_tokens) as i32 }),
            (None, false, true) => (quote! { bool }, quote! { (#body_tokens) != 0 }),
            (None, false, false) => (quote! { u32 }, quote! { #body_tokens }),
            (Some(_), false, false) => (quote! { #arg_ident }, quote! { #arg_ident::parse((#body_tokens) as u8) }),
            _ => unreachable!(),
        };

        field_accessors_tokens.extend(quote! {
            #[doc = #doc]
            #[inline(always)]
            pub const fn #fn_name(&self) -> #ret_type {
                #inner
            }
        });
    }
    Ok(field_accessors_tokens)
}

fn generate_opcode_tokens(sorted_opcodes: &[Opcode]) -> (TokenStream, TokenStream, Literal) {
    let mut opcode_enum_tokens = TokenStream::new();
    let mut opcode_mnemonics_tokens = TokenStream::new();
    let num_opcodes_token = Literal::usize_unsuffixed(sorted_opcodes.len());
    for (i, opcode) in sorted_opcodes.iter().enumerate() {
        let name = &opcode.name();
        opcode_mnemonics_tokens.extend(quote! { #name, });

        let enum_name = Ident::new(&opcode.enum_name(), Span::call_site());
        let enum_value = Literal::u8_unsuffixed(i.try_into().unwrap());
        let doc = opcode.doc();
        opcode_enum_tokens.extend(quote! {
            #[doc = #doc]
            #enum_name = #enum_value,
        });
    }
    (opcode_enum_tokens, opcode_mnemonics_tokens, num_opcodes_token)
}
