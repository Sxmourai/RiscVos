import os

def gen_func(ty: str, args: str):
    macro_name = f"instruction_{ty.lower()}"
    parser = f"parse_{ty.lower()}"
    inner_code = "#code"
    if "rs1" in args:
        inner_code = f"let vs1 = *vm.cpu.reg(rs1);\n{inner_code}"
    if "rs2" in args:
        inner_code = f"let vs2 = *vm.cpu.reg(rs2);\n{inner_code}"
    if "rd" in args:
        inner_code = inner_code.replace("#code", f"*vm.cpu.reg(rd) = #code")
        
    code = f"""
#[proc_macro]
pub fn {macro_name}(input: TokenStream) -> TokenStream {{
    let InstructionMacro {{ name, code }} = parse_macro_input!(input as _);

    quote! {{
        (stringify!(#name),Instruction32Format::{ty}, |vm, instruction| {{
            let {args} = instruction.{parser}();
            {inner_code}
        }})
    }}.into()
}}
"""
    
    return code

# pub fn #name(vm: &mut crate::vm::VM, instruction: Instruction) {{
#             let {args} = instruction.{parser}();
#             #code
#         }}

with open("src/_lib.rs", "r") as f:
    c = f.readlines()
    previous = c[:]
    for i,line in enumerate(c):
        if line.startswith("gen_instruction_fmt!("):
            macro_args = line[len("gen_instruction_fmt!("):(line.index(";")-1)]
            ty = macro_args[0]
            args = macro_args[macro_args.index("("):]
            lines = gen_func(ty,args).splitlines(keepends=True)
            c.pop(i)
            for j in range(len(lines)):
                c.insert(i+j, lines[j])
with open("src/lib.rs", "w") as f:
    f.writelines(c)

# os.system("cargo b")

# with open("src/lib.rs", "w") as f:
#     f.writelines(previous)
