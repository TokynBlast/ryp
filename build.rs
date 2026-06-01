use std::fs;
use std::path::Path;

fn main() {
    // Rebuild any time these change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=README.md");

    // Sets the Usage string for use in
    {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let dest_usage_path = Path::new(&out_dir).join("usage.rs");

        let mut usage_raw = fs::read_to_string("README.md")
            .expect("Failed to read README");

        let usage_index = usage_raw
            .find("## Usage")
            .expect("Build failed; Make sure the README has `## Usage`, or update the build script to match a new name.");

        // 13 to account for `## Usage`, back ticks, and newlines
        usage_raw = String::from(&usage_raw[usage_index + 13..]);
        usage_raw = String::from(
            usage_raw
                .split_once("```")
                .expect("The README `Usage` section may have missing back ticks.")
                .0,
        );

        let mut usage_code = String::from("{\n");
        for line in usage_raw.lines() {
            let escaped_line = line.replace('"', "\\\"");
            usage_code.push_str(&format!("println!(\"{}\");\n", escaped_line));
        }
        usage_code.push_str("}\n");

        fs::write(&dest_usage_path, usage_code).unwrap();
    }

    // Font and licensing
    {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let out_path = Path::new(&out_dir);

        let font_ttf = out_path.join("0xProtoNerdFontMono-Regular.ttf");
        let font_license = out_path.join("LICENSE.0xProto");
        let font_rs = out_path.join("font.rs");

        // Only download if the font isn't already cached in OUT_DIR
        if !font_ttf.exists() || !font_license.exists() {
            println!("cargo:warning=Downloading 0xProto Nerd Font...");

            let url = "https://github.com/ryanoasis/nerd-fonts/releases/download/v3.3.0/0xProto.tar.xz";

            let response = ureq::get(url)
                .call()
                .expect("Failed to download font 0xProto — check your internet connection");

            // Stream HTTP body → xz decoder → tar reader all within memory
            let mut body = response.into_body();
            let xz_reader = xz2::read::XzDecoder::new(body.as_reader());
            let mut archive = tar::Archive::new(xz_reader);

            let mut got_font    = false;
            let mut got_license = false;

            for entry in archive.entries().expect("Failed to read tar entries") {
                let mut entry = entry.expect("Bad tar entry");
                let entry_path = entry.path().expect("Bad entry path").into_owned();
                let name = entry_path.file_name().unwrap_or_default().to_string_lossy();

                if name == "0xProtoNerdFontMono-Regular.ttf" {
                    entry.unpack(&font_ttf).expect("Failed to extract font");
                    got_font = true;
                } else if name == "LICENSE" {
                    entry.unpack(&font_license).expect("Failed to extract LICENSE");
                    got_license = true;
                }

                if got_font && got_license {
                    break;
                }
            }

            assert!(got_font,    "0xProtoNerdFontMono-Regular.ttf not found in archive");
            assert!(got_license, "LICENSE not found in archive");

            println!("cargo:warning=Font ready.");
        }

        let font_rs_code = "pub const FONT_BYTES: &[u8] = \
            include_bytes!(concat!(env!(\"OUT_DIR\"), \"/0xProtoNerdFontMono-Regular.ttf\"));\n";
        fs::write(&font_rs, font_rs_code).expect("Failed to write font.rs");

        println!("cargo:rustc-env=FONT={}", font_ttf.display());
        println!("cargo:rustc-env=OFL_LICENSE={}", font_license.display());

        // Re-run if the cached font disappears
        println!("cargo:rerun-if-changed={}", font_ttf.display());
    }
}
