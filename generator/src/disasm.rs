use std::cmp::Ordering;

use anyhow::Result;
use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::{
    isa::{Isa, Opcode},
    token::HexLiteral,
};

pub fn generate_disasm(isa: &Isa, bucket_bitmask: u32) -> Result<TokenStream> {
    // To improve the search, the opcodes are sorted by bucket index. Read further for more info.
    let sorted_opcodes = sort_opcodes(isa, bucket_bitmask);
    let (opcode_patterns_tokens, opcode_enum_tokens, opcode_mnemonics_tokens, num_opcodes_token) =
        generate_opcode_tokens(&sorted_opcodes);

    // We could use a binary search on sorted_opcodes, but we can do better. We generate a lookup table which maps a bucket
    // index to the range of opcodes in sorted_opcodes with that bucket index.
    let bucket_count = 1 << bucket_bitmask.count_ones();
    let lookup_table = create_lookup_table(bucket_count, sorted_opcodes);
    let lookup_table_tokens = generate_lookup_table_tokens(lookup_table);

    // Generate bucket index function
    let mut bucket_index_body_tokens = TokenStream::new();
    bucket_index_body_tokens.extend(quote! {
        let mut index = 0;
    });
    let mut bitmask = bucket_bitmask;
    let mut total_shift = 0;
    let mut bucket_bits = 0;
    while bitmask != 0 {
        let zero_shift = bitmask.trailing_zeros();
        bitmask >>= zero_shift;
        let one_shift = bitmask.trailing_ones();
        let bits = (1 << one_shift) - 1;
        bitmask >>= one_shift;

        total_shift += zero_shift;
        let mask_token = HexLiteral(bits << total_shift);
        let bucket_shift: i32 = total_shift as i32 - bucket_bits as i32;
        let shift_token = Literal::i32_unsuffixed(bucket_shift.abs());
        match bucket_shift.cmp(&0) {
            Ordering::Greater => bucket_index_body_tokens.extend(quote! {
                index |= (code & #mask_token) >> #shift_token;
            }),
            Ordering::Less => bucket_index_body_tokens.extend(quote! {
                index |= (code & #mask_token) << #shift_token;
            }),
            Ordering::Equal => bucket_index_body_tokens.extend(quote! {
                index |= code & #mask_token;
            }),
        }
        bucket_bits += one_shift;

        // let shift_token = Literal::u32_unsuffixed(total_shift);
        // let bits_token = HexLiteral(bits);
        // if total_shift == 0 {
        //     bucket_index_body_tokens.extend(quote! {
        //         index |= code & #bits_token;
        //     });
        // } else if total_shift + one_shift == 32 {
        //     bucket_index_body_tokens.extend(quote! {
        //         index |= code >> #shift_token;
        //     });
        // } else {
        // bucket_index_body_tokens.extend(quote! {
        //     index |= (code >> #shift_token) & #bits_token;
        // });
        // }
        total_shift += one_shift;
    }
    bucket_index_body_tokens.extend(quote! {
        index.try_into().unwrap()
    });

    let mut bucket_index_tokens = TokenStream::new();
    bucket_index_tokens.extend(quote! {
        fn bucket_index(code: u32) -> usize {
            #bucket_index_body_tokens
        }
    });

    // TODO: Generate field accessors

    // TODO: Generate modifier accessors

    let bucket_count = Literal::u8_unsuffixed(bucket_count);
    Ok(quote! {
        #![cfg_attr(rustfmt, rustfmt_skip)]
        #[comment = " Generated by armv5te-generator. Do not edit!"]

        #[doc = " This lookup table limits the search to at most 7 opcodes by reading 6 pre-selected bits from the instruction."]
        static LOOKUP_TABLE: [(u8, u8); #bucket_count] = [#lookup_table_tokens];

        #[doc = " These tuples contain a bitmask and pattern for each opcode."]
        static OPCODE_PATTERNS: [(u32, u32); #num_opcodes_token] = [#opcode_patterns_tokens];

        #[doc = " These are the mnemonics of each opcode. Some opcodes are duplicated due to them having multiple formats."]
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
            #[inline]
            pub fn find(code: u32) -> Self {
                let index = bucket_index(code);
                let lookup = LOOKUP_TABLE[index];
                if lookup.0 == lookup.1 {
                    return Self::Illegal;
                }
                for i in lookup.0..lookup.1 {
                    let (bitmask, pattern) = OPCODE_PATTERNS[i as usize];
                    if (code & bitmask) == pattern {
                        return unsafe { core::mem::transmute::<u8, Opcode>(i) };
                    }
                }
                Self::Illegal
            }
            pub fn mnemonic(self) -> &'static str {
                OPCODE_MNEMONICS[self as usize]
            }
            pub fn count() -> usize {
                #num_opcodes_token
            }
        }

        #bucket_index_tokens
    })
}

fn generate_lookup_table_tokens(lookup_table: Vec<(u8, u8)>) -> TokenStream {
    let mut lookup_table_tokens = TokenStream::new();
    for entry in lookup_table {
        let start = Literal::u8_unsuffixed(entry.0);
        let end = Literal::u8_unsuffixed(entry.1);
        lookup_table_tokens.extend(quote! { (#start, #end), });
    }
    lookup_table_tokens
}

fn create_lookup_table(bucket_count: u8, sorted_opcodes: Vec<(Opcode, u8)>) -> Vec<(u8, u8)> {
    (0..bucket_count)
        .map(|bucket| {
            let start = sorted_opcodes
                .iter()
                .position(|(_, b)| *b == bucket)
                .map(|p| p.try_into().unwrap())
                .unwrap_or(0);
            let end = sorted_opcodes
                .iter()
                .rev()
                .position(|(_, b)| *b == bucket)
                .map(|p| (sorted_opcodes.len() - p).try_into().unwrap())
                .unwrap_or(0);
            (start, end)
        })
        .collect()
}

fn generate_opcode_tokens(sorted_opcodes: &[(Opcode, u8)]) -> (TokenStream, TokenStream, TokenStream, Literal) {
    let mut opcode_patterns_tokens = TokenStream::new();
    let mut opcode_enum_tokens = TokenStream::new();
    let mut opcode_mnemonics_tokens = TokenStream::new();
    let num_opcodes_token = Literal::usize_unsuffixed(sorted_opcodes.len());
    for (i, (opcode, _)) in sorted_opcodes.iter().enumerate() {
        let bitmask = HexLiteral(opcode.bitmask);
        let pattern = HexLiteral(opcode.pattern);
        let doc = opcode.doc();
        opcode_patterns_tokens.extend(quote! {
            #[comment = #doc]
            (#bitmask, #pattern),
        });

        let name = &opcode.name();
        opcode_mnemonics_tokens.extend(quote! { #name, });

        let enum_name = Ident::new(&opcode.enum_name(), Span::call_site());
        let enum_value = Literal::u8_unsuffixed(i.try_into().unwrap());
        opcode_enum_tokens.extend(quote! {
            #[doc = #doc]
            #enum_name = #enum_value,
        });
    }
    (
        opcode_patterns_tokens,
        opcode_enum_tokens,
        opcode_mnemonics_tokens,
        num_opcodes_token,
    )
}

fn sort_opcodes(isa: &Isa, bucket_bitmask: u32) -> Vec<(Opcode, u8)> {
    let mut opcodes: Vec<(Opcode, u8)> = isa
        .opcodes
        .iter()
        .cloned()
        .map(|op| {
            let bucket = op.bucket_index(bucket_bitmask);
            (op, bucket.try_into().unwrap())
        })
        .collect();
    opcodes.sort_by(|(a, bucket_a), (b, bucket_b)| {
        bucket_a
            .cmp(bucket_b)
            // When bitmask A is a subset of B, then B must be first, otherwise Opcode::find will never choose B
            .then(b.bitmask.count_ones().cmp(&a.bitmask.count_ones()))
    });
    opcodes
}
