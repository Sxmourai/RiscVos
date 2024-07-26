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


#[proc_macro]
pub fn instruction_r(input: TokenStream) -> TokenStream {
    let Instruction { name, code } = parse_macro_input!(input as _);

    quote! {
        (stringify!(#name),InstructionFormat::R, |vm, instruction| {
            let (rs1, rs2, rd) = instruction.parse_r();
            let vs2 = *vm.cpu.reg(rs2);
let vs1 = *vm.cpu.reg(rs1);
*vm.cpu.reg(rd) = #code
        })
    }.into()
}

#[proc_macro]
pub fn instruction_i(input: TokenStream) -> TokenStream {
    let Instruction { name, code } = parse_macro_input!(input as _);

    quote! {
        (stringify!(#name),InstructionFormat::I, |vm, instruction| {
            let (imm, rs1, rd) = instruction.parse_i();
            let vs1 = *vm.cpu.reg(rs1);
*vm.cpu.reg(rd) = #code
        })
    }.into()
}

#[proc_macro]
pub fn instruction_s(input: TokenStream) -> TokenStream {
    let Instruction { name, code } = parse_macro_input!(input as _);

    quote! {
        (stringify!(#name),InstructionFormat::S, |vm, instruction| {
            let (imm, rs1, rs2) = instruction.parse_s();
            let vs2 = *vm.cpu.reg(rs2);
let vs1 = *vm.cpu.reg(rs1);
#code
        })
    }.into()
}

#[proc_macro]
pub fn instruction_b(input: TokenStream) -> TokenStream {
    let Instruction { name, code } = parse_macro_input!(input as _);

    quote! {
        (stringify!(#name),InstructionFormat::B, |vm, instruction| {
            let (imm, rs1, rs2) = instruction.parse_b();
            let vs2 = *vm.cpu.reg(rs2);
let vs1 = *vm.cpu.reg(rs1);
#code
        })
    }.into()
}

#[proc_macro]
pub fn instruction_u(input: TokenStream) -> TokenStream {
    let Instruction { name, code } = parse_macro_input!(input as _);

    quote! {
        (stringify!(#name),InstructionFormat::U, |vm, instruction| {
            let (imm, rd) = instruction.parse_u();
            *vm.cpu.reg(rd) = #code
        })
    }.into()
}

#[proc_macro]
pub fn instruction_j(input: TokenStream) -> TokenStream {
    let Instruction { name, code } = parse_macro_input!(input as _);

    quote! {
        (stringify!(#name),InstructionFormat::J, |vm, instruction| {
            let (imm, rd) = instruction.parse_j();
            *vm.cpu.reg(rd) = #code
        })
    }.into()
}
