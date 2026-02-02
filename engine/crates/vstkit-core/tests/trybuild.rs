#[test]
#[cfg(target_os = "macos")]
fn trybuild_macros() {
    let t = trybuild::TestCases::new();
    // Use Cargo-based test packages so the test packages can depend on `vstkit-core`
    t.pass("tests/trybuild/minimal/src/main.rs");
    t.pass("tests/trybuild/full/src/main.rs");
}
