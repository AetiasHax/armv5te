use anyhow::{bail, Context, Result};
use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use syn::{
    parse_quote,
    visit_mut::{self, VisitMut},
    Expr, ExprLit, Ident, Lit,
};

use crate::{
    args::{ArgType, IsaArgs},
    isa::{Field, FieldValue, Isa, Opcode},
    iter::cartesian,
    search::SearchTree,
    token::HexLiteral,
};

pub fn generate_disasm(isa: &Isa, isa_args: &IsaArgs, module: &str) -> Result<TokenStream> {
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
    let field_accessors_tokens = generate_field_accessors(isa, isa_args)?;

    // Generate modifier case enums
    let case_enums_tokens = generate_modifier_case_enums(isa);

    // Generate modifier accessors
    let modifier_accessors_tokens = generate_modifier_accessors(isa)?;

    // Generate parse functions
    let max_args = isa.get_max_args()?;
    let parse_functions = generate_parse_functions(isa, isa_args, max_args, &isa.opcodes, &num_opcodes_token)?;

    let max_args = Literal::usize_unsuffixed(max_args);
    let module = Ident::new(module, Span::call_site());
    Ok(quote! {
        #![cfg_attr(rustfmt, rustfmt_skip)]
        #![allow(unused)]
        #![allow(clippy::double_parens, clippy::unnecessary_cast)]
        #[comment = " Generated by armv5te-generator. Do not edit!"]

        use crate::{
            args::*,
            #module::disasm::{Ins, ParsedIns}
        };

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
    isa_args: &IsaArgs,
    max_args: usize,
    sorted_opcodes: &[Opcode],
    num_opcodes_token: &Literal,
) -> Result<TokenStream, anyhow::Error> {
    let illegal_args = (0..max_args).map(|_| quote! { Argument::None });
    let illegal_ins = quote! {
        ParsedIns {
            mnemonic: "<illegal>",
            args: [
                #(#illegal_args),*
            ],
        }
    };

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
                let args = generate_mnemonic_args(isa_args, max_args, opcode_args)?;
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
                    let mnemonic = opcode.base_name().to_string() + &suffix + &opcode.suffix;
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

                    let args = generate_mnemonic_args(isa_args, max_args, case_args)?;
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
            if ins.op != Opcode::Illegal {
                MNEMONIC_PARSERS[ins.op as usize](out, ins);
            } else {
                *out = #illegal_ins;
            }
        }
    });
    Ok(parse_functions)
}

fn generate_mnemonic_args(isa_args: &IsaArgs, max_args: usize, args: Vec<&Field>) -> Result<Vec<TokenStream>> {
    let args = (0..max_args)
        .map(|i| {
            if i < args.len() {
                let field = args[i];
                let accessor = Ident::new(&field.accessor_name(), Span::call_site());
                let arg = isa_args.get_arg(&field.arg)?;
                let arg_variant = Ident::new(&arg.pascal_case_name(), Span::call_site());
                let access_variant = quote! { #arg_variant(ins.#accessor()) };
                Ok(quote! { Argument::#access_variant })
            } else {
                Ok(quote! { Argument::None })
            }
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(args)
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
                #[derive(Debug, Clone, Copy, PartialEq, Eq)]
                pub enum #enum_ident {
                    Illegal,
                    #variants_tokens
                }
            })
        }
    }
    case_enums_tokens
}

fn generate_field_accessors(isa: &Isa, isa_args: &IsaArgs) -> Result<TokenStream> {
    let accessors = isa
        .fields
        .iter()
        .map(|field| {
            let arg = isa_args.get_arg(&field.arg)?;
            let body = match &arg.r#type {
                ArgType::Struct(members) => {
                    let values = if let FieldValue::Struct(values) = &field.value {
                        values
                    } else {
                        bail!("Value of field '{}' must be a struct", field.name);
                    };

                    let struct_ident = Ident::new(&arg.pascal_case_name(), Span::call_site());
                    let struct_members = members
                        .iter()
                        .map(|(name, member)| {
                            let value = values.get(name).with_context(|| {
                                format!("Member '{}' missing from struct value in field '{}'", name, field.name)
                            })?;
                            let expr = generate_argument_expr(value, field)?;
                            let expr = match &member.r#type {
                                ArgType::Struct(_) => {
                                    bail!("Nested structs (in argument '{}') are not supported", arg.name);
                                }
                                ArgType::Enum(_) => {
                                    bail!("Nested enums (in argument '{}') are not supported", arg.name);
                                }
                                ArgType::U32 => expr,
                                ArgType::I32 => quote! { (#expr) as i32 },
                                ArgType::Bool => {
                                    if let FieldValue::Bool(_) = value {
                                        quote! { #expr }
                                    } else {
                                        quote! { (#expr) != 0 }
                                    }
                                }
                                ArgType::Custom(custom_name) => {
                                    let custom_type = isa_args.get_type(custom_name)?;
                                    let custom_ident = Ident::new(&custom_type.pascal_case_name(), Span::call_site());
                                    quote! { #custom_ident::parse(#expr) }
                                }
                            };

                            let ident = Ident::new(name, Span::call_site());
                            Ok(quote! {
                                #ident: #expr
                            })
                        })
                        .collect::<Result<Vec<_>>>()?;

                    quote! {
                        #struct_ident {
                            #(#struct_members),*
                        }
                    }
                }
                ArgType::Enum(_) => {
                    let enum_ident = Ident::new(&arg.pascal_case_name(), Span::call_site());
                    let expr = generate_argument_expr(&field.value, field)?;
                    quote! { #enum_ident::parse(#expr) }
                }
                ArgType::U32 => generate_argument_expr(&field.value, field)?,
                ArgType::I32 => {
                    let body = generate_argument_expr(&field.value, field)?;
                    quote! { (#body) as i32 }
                }
                ArgType::Bool => generate_argument_expr(&field.value, field)?,
                ArgType::Custom(custom_name) => {
                    let custom_type = isa_args.get_type(custom_name)?;
                    let custom_ident = Ident::new(&custom_type.pascal_case_name(), Span::call_site());
                    let expr = generate_argument_expr(&field.value, field)?;
                    quote! { #custom_ident::parse(#expr) }
                }
            };

            let arg_ident = Ident::new(&arg.pascal_case_name(), Span::call_site());
            let return_type = match arg.r#type {
                ArgType::Struct(_) => quote! { #arg_ident },
                ArgType::Enum(_) => quote! { #arg_ident },
                ArgType::U32 => quote! { u32 },
                ArgType::I32 => quote! { i32 },
                ArgType::Bool => quote! { bool },
                ArgType::Custom(_) => quote! { #arg_ident },
            };

            let accessor_ident = Ident::new(&field.accessor_name(), Span::call_site());
            Ok(quote! {
                pub fn #accessor_ident(&self) -> #return_type {
                    #body
                }
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        #(#accessors)*
    })
}

struct FoldFieldExpr;

impl VisitMut for FoldFieldExpr {
    fn visit_expr_mut(&mut self, node: &mut Expr) {
        if let Expr::MethodCall(call) = node {
            let lhs = call.receiver.as_ref();
            match call.method.to_string().as_str() {
                "bits" => {
                    if call.args.len() != 2 {
                        return;
                    }
                    let start = get_literal_value(&call.args[0]);
                    let end = get_literal_value(&call.args[1]);

                    let shift = start;
                    let mask = HexLiteral((1 << (end - start)) - 1);
                    if shift == 0 {
                        *node = parse_quote! { (#lhs & #mask) };
                    } else {
                        let shift = Literal::i32_unsuffixed(shift);
                        *node = parse_quote! { ((#lhs >> #shift) & #mask) };
                    }
                }
                "bit" => {
                    if call.args.len() != 1 {
                        return;
                    }
                    let bit = get_literal_value(&call.args[0]);
                    let mask = HexLiteral(1 << bit);
                    *node = parse_quote! { ((#lhs & #mask) != 0) };
                }
                "negate" => {
                    if call.args.len() != 1 {
                        return;
                    }
                    let rhs = call.args[0].clone();
                    *node = parse_quote! { {
                        let value = #lhs as i32;
                        if #rhs {
                            -value
                        } else {
                            value
                        }
                    } };
                }
                "arm_shift" => {
                    if call.args.len() != 1 {
                        return;
                    }
                    let rhs = call.args[0].clone();
                    *node = parse_quote! { {
                        let value = #lhs;
                        match #rhs {
                            1 | 2 => if value == 0 { 32 } else { value },
                            _ => value
                        }
                    } };
                }
                _ => {}
            }
        }
        // println!("{}", node.to_token_stream());
        visit_mut::visit_expr_mut(self, node);
    }
}

fn get_literal_value(expr: &Expr) -> i32 {
    if let Expr::Lit(ExprLit {
        attrs: _,
        lit: Lit::Int(int),
    }) = expr
    {
        int.base10_parse().unwrap_or(i32::MIN)
    } else {
        i32::MIN
    }
}

fn generate_argument_expr(value: &FieldValue, field: &Field) -> Result<TokenStream, anyhow::Error> {
    let expr = match value {
        FieldValue::Bits(range) => {
            let start = Literal::u8_unsuffixed(range.0.start);
            let end = Literal::u8_unsuffixed(range.0.end);
            let mut expr = parse_quote! { self.code.bits(#start,#end) };
            FoldFieldExpr.visit_expr_mut(&mut expr);
            quote! { #expr }
        }
        FieldValue::Bool(value) => {
            if *value {
                quote! { true }
            } else {
                quote! { false }
            }
        }
        FieldValue::U32(value) => {
            let value = Literal::u32_unsuffixed(*value);
            quote! { #value }
        }
        FieldValue::Struct(_) => {
            bail!("Nested structs (in field '{}') are not supported", field.name);
        }
        FieldValue::Expr(expr) => {
            let mut expr = syn::parse_str(expr)?;
            FoldFieldExpr.visit_expr_mut(&mut expr);
            quote! { #expr }
        }
    };
    Ok(expr)
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
