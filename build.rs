use std::fs;
use std::path::Path;

fn main() {
    // Rebuild any time these change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=README.md");

    // Sets the ENV VAR OUT_DIR
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_usage_path = Path::new(&out_dir).join("usage.rs");

    let mut usage_raw = fs::read_to_string("README.md")
        .expect("Failed to read README");

    let usage_index = usage_raw.find("## Usage").expect("Build failed; Make sure the README has `## Usage`, or update the build script to match a new name.");

    // 13 to account for `## Usage`, back ticks, and newlines
    usage_raw = String::from(&usage_raw[usage_index + 13 ..]);
    usage_raw = String::from(usage_raw.split_once("```").expect("The README `Usage` section may have missing back ticks.").0);

    let mut usage_code = String::from("{\n");
    for line in usage_raw.lines() {
        let escaped_line = line.replace('"', "\\\"");
        usage_code.push_str(&format!("    println!(\"{}\");\n", escaped_line));
    }
    usage_code.push_str("}\n");

    fs::write(&dest_usage_path, usage_code).unwrap();
}
