use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};

// May be useful in the future:
// 🖸🖴🖵🖱🖰🖲🖶🖻🖺🖮🖫🖪🕯
pub fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let text = match app.current_editor() {
        Some(editor) => {
              let lang = if let Some(path) = &editor.filepath {
                match path.extension().and_then(|e| e.to_str()) {
                    Some("cpp") => "C++ 󰙲",
                    Some("hpp") => "C++ Header 󰙲",
                    Some("rs") => "Rust 󱘗",
                    Some("lua") => "Lua ",
                    Some("ll") => "LLVM ",
                    Some("asm") | Some("s") => "Assembly",
                    Some("c") => "C 󰙱",
                    Some("h")=> "C Header 󰙱",
                    Some("js") => "JavaScript ",
                    Some("ml") | Some("mli") => "OCaml ",
                    Some("html") => "HTML ",
                    Some("md") => "MarkDown 󰍔",
                    Some("css") => "CSS ",
                    Some("mi") => "Minis",
                    Some("cs") => "C# 󰌛",
                    Some("gd") => "Godot Script ",
                    Some("py") => "Python 󰌠",
                    Some("java") => "Java 󰬷",
                    Some("fs") => "F#",
                    Some("bat") => "Bash ",
                    Some("sh") => "Shell ",
                    Some("go") => "Go 󰟓",
                    Some("php") => "PHP 󰌟",
                    Some("rb") => "Ruby ",
                    Some("ts") => "TypeScript 󰛦",
                      Some("f")
                    | Some("for")
                    | Some("f08")
                    | Some("f90")
                    | Some("f03")
                    | Some("f95")
                    | Some("F90")
                    | Some("F")
                    | Some("f15")
                    | Some("f20") => "Fortran 󱈚",
                    Some("m") => "Objective-C ",
                    Some("mm") => "Objective-C++",
                    Some("adb") => "Ada",
                    Some("d") => "D ",
                    Some("mod") => "Modula",
                    Some("cob") => "COBOL",
                    Some("a68") => "ALGOL",
                    Some("ipynb") => "Jupyter Notebook",
                    Some("red") => "Red",
                    Some("json") => "JSON ",
                    Some("r") => "R ",
                    Some("lhs") => "Haskel ",
                    Some("xaml") => "XAML 󰙳",
                    Some("yaml") => "YAML ",
                    Some("kt") => "Kotlin ",
                    Some("kts") => "Kotlin Script ",
                    Some("txt") => "Plain Text ",
                    _ => "Unknown",
                }
            } else {
                "Unknown"
            };

            let os = if cfg!(target_os = "windows") {
              ""
            } else if cfg!(target_os = "macos"){
              ""
            } else if cfg!(target_os = "linux") {
              match os_info::get().os_type() {
                os_info::Type::Pop => "!Pop_OS ",
                os_info::Type::Arch => "Arch Linux 󰣇",
                os_info::Type::Fedora => "Fedora ",
                os_info::Type::Gentoo => "Gentoo ",
                os_info::Type::Redhat | os_info::Type::RedHatEnterprise => "Redhat ",
                os_info::Type::AlmaLinux => "AlmaLinux ",
                os_info::Type::AOSC => "AOSC ",
                os_info::Type::Artix => "Artix ",
                os_info::Type::CentOS => "CentOS ",
                os_info::Type::Cygwin => "Cygwin ",
                os_info::Type::Debian => "Debian ",
                os_info::Type::Elementary => "ElementaryOS ",
                os_info::Type::EndeavourOS => "EndeavourOS ",
                os_info::Type::FreeBSD => "FreeBSD ",
                os_info::Type::Garuda => "Garuda ",
                os_info::Type::Illumos => "Illumos ",
                os_info::Type::Kali => "Kali Linux ",
                os_info::Type::Manjaro => "Manjaro ",
                os_info::Type::Mint => "Linux Mint 󰣭",
                os_info::Type::NixOS => "NixOS ",
                os_info::Type::Nobara => "Nobara ",
                os_info::Type::OpenBSD => "OpenBSD ",
                os_info::Type::Raspbian => "Raspbian ",
                os_info::Type::RockyLinux => "RockyLinux ",
                os_info::Type::openSUSE => "openSUSE ",
                os_info::Type::SUSE => "SUSE ",
                os_info::Type::Solus => "Solus ",
                os_info::Type::Ubuntu => "Ubuntu 󰕈",
                os_info::Type::Void => "Void Linux ",
                os_info::Type::Zorin => "Zorin ",
                _ => ""
              }
            } else {
              "?"
            };

            format!(
                " {} | Lines: {} | Col: {} | Tab Size: {} | OS: {} ",
                lang,
                editor.lines.len(),
                editor.cursor_x + 1,
                app.config.get("Tab Size").and_then(|v| v.as_u64()).unwrap_or(4),
                os
            )
        }
        None => format!(" No files open | Tab Size: {} ", app.config.get("Tab Size").and_then(|v| v.as_u64()).unwrap_or(4)),
    };

    let p = Paragraph::new(text).style(Style::default().bg(Color::Blue).fg(Color::White));
    f.render_widget(p, area);
}
