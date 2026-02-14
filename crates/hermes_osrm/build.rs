use std::process::Command;

fn main() {
    let flatc = if cfg!(windows) {
        "../../tools/flatc.exe"
    } else {
        "../../tools/flatc"
    };

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
