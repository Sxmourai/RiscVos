#![allow(unused_macros, unused_imports, unused_variables)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::{Parse, ParseStream}, parse_macro_input, Ident, Token};

extern crate proc_macro;

struct Instruction {
    name: Ident,
    code: syn::Block,
}

// Implement the Parse trait to parse the macro input
impl Parse for Instruction {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        // Parse the comma
        let _: Token![,] = input.parse()?;
        let code = input.parse()?;
        
        Ok(Self { name, code })
    }
}

macro_rules! gen_instruction_fmt {
    ($out: ident, $args: tt) => {compile_error!("Implemented in python build script, not here !");};
}

gen_instruction_fmt!(R, (rs1, rs2, rd));
gen_instruction_fmt!(I, (imm, rs1, rd));
gen_instruction_fmt!(S, (imm, rs1, rs2));
gen_instruction_fmt!(B, (imm, rs1, rs2));
gen_instruction_fmt!(U, (imm, rd));
gen_instruction_fmt!(J, (imm, rd));
