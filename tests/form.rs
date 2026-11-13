//! plays the form with real keys (tests/form_scenarios.py) when python3 is
//! there; the script checks the store after each scenario

#[test]
fn form_scenarios_pass_with_real_keys() {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/form_scenarios.py");
    let Ok(out) = std::process::Command::new("python3").args(["-I", script, env!("CARGO_BIN_EXE_cake")]).output() else {
        eprintln!("python3 missing, form scenarios skipped");
        return;
    };
    let text = String::from_utf8_lossy(&out.stdout);
    println!("{}", text);
    assert!(out.status.success(), "{}\n{}", text, String::from_utf8_lossy(&out.stderr));
}
