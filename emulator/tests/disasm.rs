use std::fmt::Write;
pub fn strip_raw(raw: String) -> Option<String> {
    let mut parsed = String::new();
    for line in raw.split("\n") {
        if line.ends_with(":") {continue} // Function definition, they are skipped when in binary
        let parsed_line = line.trim().replace(",", "");
        writeln!(parsed, "{}", parsed_line).ok()?;
    }
    Some(parsed)
}

#[test]
pub fn disasm_simple() {
    let mut cmd = std::process::Command::new("python3");
    cmd.args(&["compile.py", "test"]);
    let mut handle = cmd.spawn().unwrap();
    assert!(handle.wait().unwrap().success());

    let raw = std::fs::read_to_string("test.s").unwrap();
    let parsed = emulator::vm::disasm(std::fs::read("target/test.o").unwrap()).unwrap();
    assert_eq!(strip_raw(raw).unwrap(),parsed);
}