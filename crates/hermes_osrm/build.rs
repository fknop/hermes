use std::process::Command;

// https://github.com/google/flatbuffers/releases
fn get_executable_path() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "../../tools/macos/flatc",
        ("windows", _) => "../../tools/windows/flatc.exe",
        _ => panic!("Unsupported platform: add the flatc binary in the tools folder"),
    }
}

fn main() {
    println!("cargo:rerun-if-changed=schemas/fbresult.fbs");
    println!("cargo:rerun-if-changed=schemas/position.fbs");
    println!("cargo:rerun-if-changed=schemas/route.fbs");
    println!("cargo:rerun-if-changed=schemas/table.fbs");
    println!("cargo:rerun-if-changed=schemas/waypoint.fbs");
    println!("cargo:rerun-if-changed=build.rs");

    let flatc = get_executable_path();

    let status = Command::new(flatc)
        .args([
            "--rust",
            "--gen-all",
            "-o",
            "src/generated/",
            "-I",
            "schemas/",
            "schemas/fbresult.fbs",
        ])
        .status()
        .expect("failed to run flatc");

    if !status.success() {
        panic!("flatc failed with {status}");
    }
}
