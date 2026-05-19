use std::fs;
use std::path::{PathBuf, Path};
use compact_str::CompactString;
use std::cell::Cell;
use std::sync::Arc;
use arc_swap::ArcSwap;
use std::collections::VecDeque;
use arboard::Clipboard;
use syntect::{
  parsing::ScopeStack,
  highlighting::{
      ThemeSet,
      Highlighter,
      HighlightState,
  }
};

pub struct Editor {
    pub lines: Vec<CompactString>,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub target_x: usize,
    pub scroll_y: Cell<usize>,
    pub scroll_x: Cell<usize>,
    pub selection_start: Option<(usize, usize)>, // (start_x, start_y)
    pub filepath: Option<PathBuf>,
    pub dirty: bool,
    pub is_diff: bool,
    pub lang: CompactString,
    pub highlight_cache: Arc<ArcSwap<VecDeque<HighlightState>>>,
    pub clipboard: Option<arboard::Clipboard>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            lines: vec![CompactString::default()],
            cursor_x: 0,
            cursor_y: 0,
            target_x: 0,
            scroll_y: Cell::new(0),
            scroll_x: Cell::new(0),
            selection_start: None,
            filepath: None,
            dirty: false,
            is_diff: false,
            lang: CompactString::default(),
            highlight_cache: Arc::new(
                ArcSwap::from_pointee(
                  VecDeque::from([
                    HighlightState::new(
                        &Highlighter::new(
                            &ThemeSet::load_defaults().themes["base16-ocean.dark"]),
                            ScopeStack::new()
                        )
                    ])
                )
            ),
            clipboard:
                if let Some(clip_board) = Some(Clipboard::new()) {
                    if clip_board.is_ok() {
                        Some(clip_board.unwrap())
                    } else {
                        None
                    }
                } else {
                    None
                },
        }
    }

    pub fn load_file(&mut self, path: &Path) -> bool {
        if let Ok(content) = fs::read_to_string(path) {
            self.lines = content.lines().map(|s| CompactString::from(s)).collect();
            if self.lines.is_empty() {
                self.lines.push(CompactString::default());
            }
            self.filepath = Some(path.to_path_buf());
            self.dirty = false;
            self.is_diff = false;
            self.lang = CompactString::const_new(
                if let Some(path) = &self.filepath {
                // TODO: Implement Naive Bayes algorithm for file detection, rather than rely purely on file ending
                //       Once we implement that, we can then remove `#[allow(unreachable_patterns)]`
                #[allow(unreachable_patterns)]
                match path.extension()
                    .and_then(|e| e.to_str())
                    .or_else(|| path.file_name().and_then(|n| n.to_str())).unwrap()
                    {
                    "cpp" => "C++ 󰙲",
                    "hpp" => "C++ Header 󰙲",
                    "rs" => "Rust 󱘗",
                    "lua" => "Lua ",
                    "ll" => "LLVM ",
                    "asm" | "s" => "Assembly",
                    "c" => "C 󰙱",
                    "h" => "C Header 󰙱",
                    "js" => "JavaScript ",
                    "ml" | "mli" => "OCaml ",
                    "html" => "HTML ",
                    "md" => "MarkDown 󰍔",
                    "css" => "CSS ",
                    "mi" => "Minis",
                    "cs" => "C# 󰌛",
                    "gd" => "Godot Script ",
                    "py" => "Python 󰌠",
                    "java" => "Java 󰬷",
                    "fs" => "F#",
                    "fsx" => "F# Script",
                    "bat" => "Bash ",
                    "sh" => "Shell ",
                    "go" => "Go 󰟓",
                    "php" => "PHP 󰌟",
                    "rb" => "Ruby ",
                    "ts" => "TypeScript 󰛦",
                      "f"
                    | "for"
                    | "f08"
                    | "f90"
                    | "f03"
                    | "f95"
                    | "F90"
                    | "F"
                    | "f15"
                    | "f20" => "Fortran 󱈚",
                    "m" => "Objective-C ",
                    "mm" => "Objective-C++",
                    "adb" => "Ada",
                    "d" => "D ",
                    "mod" => "Modula",
                    "cob" | "cbl" | "cpy" | "pco" => "COBOL",
                    "a68" => "ALGOL",
                    "ipynb" => "Jupyter Notebook ",
                    "red" => "Red",
                    "json" => "JSON ",
                    "r" => "R ",
                    "lhs" => "Haskell ",
                    "xaml" => "XAML 󰙳",
                    "yaml" => "YAML ",
                    "kt" => "Kotlin ",
                    "kts" => "Kotlin Script ",
                    "txt" => "Plain Text ",
                    "toml" => "TOML ",
                    ".gitignore" => "GITIGNORE ",
                    "lock" => "LOCK",
                    "scala" | "sc" => "Scala ",
                    "sbt" => "Scala Build Tool ",
                    "dart" => "Dart ",
                    "cpl" => "Common Programming Language",
                    "bcpl" => "Basic Common Programming Language",
                    "+" |  "a" | "m" => "A+",
                    "abap" => "Advanced Business Application Programming",
                    "abc" => "ABC",
                    "acc" => "AutoCoder Complier",
                    "act" => "Rational Synergy",
                    "as" => "ActionScript",
                    "cls" | "act" => "Actor",
                    "ad" => "Adenine",
                    "prx" | "prw" | "tlpp" => "Advanced Protheus Language",
                    "agda" | "lagda" => "Agda",
                    "vee" => "Agilent VEE",
                    "ago" => "Agora",
                    "aimms" | "ams" => "Advanced Interactive Mathematical Modeling System",
                    "as" | "al" => "Aldor",
                    "alef" => "Alef",
                    "alf" => "Algebraic Logic Functional",
                    "a0" | "alma" => "Alma-0",
                    "at" => "AmbientTalk",
                    "e" => "Amiga E",
                    "mod" | "dat" => "Mathematical Programming Language",
                    "angelscript" | "as" => "AngelScript",
                    "pig" => "Apache Pig Latin",
                    "cls" | "trigger" => "Apex Salesforce ",
                    "apl" => "A Programming Language", // This is actually what it stands for...
                    "aia" | "blk" => "MIT App Inventor",
                    "applescript" | "scpt" => "AppleScript ",
                    "apt" | "cls" => "Automatic Programmed Tool",
                    "arc" => "Arc",
                    "ets" => "ArkTS",
                    "78" => "78",
                    "bf" => "BrainFuck",
                    "rexx" | "rx" => "ARexx",
                    "arg" | "argus" => "Argus",
                    "dats" | "sats" => "Applied Type System",
                    "ahk" => "AutoHotkey",
                    "au3" => "AutoIt",
                    "asp" => "AutoLISP",
                    "awk" => "AWK ",
                    "ax" => "Axum",
                    "aml" => "Alice ML",
                    "cmm" | "c--" => "C--",
                    "b" => "B",
                    "bal" => "Ballerina ",
                    "bas" => "BASIC",
                    "bc" => "bc",
                    "bsh" => "BeanShell",
                    "bet" => "BETA",
                    "bliss" | "bli" => "BLISS",
                    "boo" => "Boo",
                    "bsq" => "Bosque",
                    "al" => "C/AL",
                    "cls" | "mac" | "int" => "Caché ObjectScript",
                    "csh" => "C Shell",
                    "clp" => "Calcpad",
                    "ml" => "Caml",
                    "carbon" => "Carbon ",
                    "catrobat" => "Catrobat",
                    "ces" => "CESIL",
                    "ceu" => "Céu",
                    "ceylon" => "Ceylon ",
                    "cf" => "CFEngine",
                    "cg" | "hlsl" => "Cg/HLSL",
                    "ch" => "Ch",
                    "chpl" => "Chapel",
                    "scm" => "CHICKEN", // esoteric languages be like 💔
                    "c8" => "CHIP-8",
                    "ck" => "ChucK",
                    "cilk" | "cilkpp" => "Cilk",
                    "claire" => "Claire",
                    "clw" => "Clarion",
                    "icl" | "dcl" => "Clean",
                    "prg" => "Clipper",
                    "clp" => "CLIPS",
                    "clist" | "exec" => "CLIST",
                    "clj" | "cljs" | "cljc" | "edn" => "Clojure ",
                    "clu" => "CLU",
                    "cbl" => "CoolScript",
                    "cobra" => "Cobra",
                    "coffee" => "CoffeeScript ",
                    "cfm" | "cfml" | "cfc" => "ColdFusion",
                    "cml" => "COMAL",
                    "cil" => "Common Intermediate Language",
                    "lisp" | "lsp" | "l" | "cl" | "fasl" => "Common Lisp ",
                    "cp" | "mod" => "Component Pascal",
                    "chr" => "Constraint Handling Rules",
                    "v" => "Rocq",
                    "cry" => "Cryptol",
                    "cr" => "Crystal ",
                    "csd" | "orc" | "sco" => "Csound",
                    "cfl" => "Cuneiform",
                    "curl" => "Curl",
                    "curry" => "Curry",
                    "cyc" => "Cyclone",
                    "cypher" => "Cypher Query Language",
                    "pyx" | "pxd" | "pxi" => "Cython ",
                    "df" => "DataFlex",
                    "dl" | "datalog" => "Datalog",
                    "dbf" => "dBase",
                    "dc" => "dc",
                    "dcl" | "com" => "DCL",
                    "pas" | "dpr" | "dfm" | "dpk" => "Delphi",
                    "dib" => "DIBOL",
                    "dra" => "Draco",
                    "dyl" | "dylan" => "Dylan",
                    "dax" => "DAX",
                    "e" => "E 󰫲",
                    "ecma" => "ECMAScript",
                    "egl" => "EGL",
                    "e" => "Eiffel 󱕫", // In reality, the logo is only a portion of the eiffel tower,
                    "ex" | "exs" => "Elixir ",
                    "elm" => "Elm ",
                    "el" => "Emacs Lisp ",
                    "erl" | "hrl" => "Erlang ",
                    "strl" => "Esterel",
                    "eu" => "Euphoria",
                    "factor" => "Factor",
                    "fan" => "Fantom",
                    "dsp" => "FAUST",
                    "fish" => "fish ",
                    "flix" => "Flix",
                    "fth" | "4th" | "forth" => "Forth",
                    "fpr" => "FoxBase/FoxPro",
                    "fut" => "Futhark",
                    "gml" => "Game Maker Language ",
                    "gms" => "GAMS",
                    "g" | "gap" => "GAP",
                    "nc" | "gcode" | "ngc" => "G-code",
                    "gleam" => "Gleam",
                    "glsl" | "vert" | "frag" | "geom" | "comp" | "tesc" | "tese" => "GLSL",
                    "golo" => "Golo",
                    "gs" => "Google Apps Script ",
                    "gsp" => "Gosu",
                    "groovy" | "gvy" | "gy" | "gsh" => "Groovy ",
                    "hack" | "hh" => "Hack ",
                    "hl" => "Halide",
                    "hrb" => "Harbour",
                    "hx" | "hxml" => "Haxe ",
                    "hla" => "HLA",
                    "hc" => "HolyC",
                    "hy" => "Hy",
                    "rpg" | "rpgle" | "sqlrpgle" => "IBM RPG",
                    "icn" => "Icon",
                    "idl" => "IDL",
                    "idr" | "lidr" => "Idris",
                    "ni" | "i7x" => "Inform",
                    "io" => "Io",
                    "ijs" => "J 󰫷",
                    "jade" => "JADE",
                    "jai" => "Jai",
                    "jal" => "JAL",
                    "jass" => "JASS",
                    "fx" => "JavaFX Script ",
                    "jcl" => "JCL",
                    "jov" => "JOVIAL",
                    "joy" => "Joy",
                    "jq" => "jq",
                    "jl" => "Julia ",
                    "k" => "K 󰫸",
                    "kix" => "KiXtart",
                    "kif" => "KIF",
                    "ksh" => "KornShell",
                    "kv" => "Kv",
                    "lasso" | "las" => "Lasso",
                    "lean" => "Lean",
                    "ly" => "LilyPond",
                    "limbo" | "b" => "Limbo",
                    "lingo" => "Lingo",
                    "lsp" => "Lisp ",
                    "ls" => "LiveScript ",
                    "lgo" | "logo" => "Logo",
                    "lgt" | "logtalk" => "Logtalk",
                    "lsl" => "LSL",
                    "lucid" => "Lucid",
                    "lus" => "Lustre",
                    "magik" => "Magik",
                    "mpl" | "maple" => "Maple",
                    "mat" | "matlab" => "MATLAB ",
                    "maude" => "Maude",
                    "ms" | "mcr" => "MaxScript",
                    "mel" => "Maya MEL ",
                    "m" => "Mercury",
                    "mirah" => "Mirah",
                    "miranda" => "Miranda",
                    "mv" => "MIVA Script",
                    "mo" => "Modelica",
                    "mojo" | "🔥" => "Mojo",
                    "moo" => "MOO",
                    "msl" => "MSL",
                    "mum" | "mumps" => "MUMPS",
                    "neko" | "n" => "Neko",
                    "n" => "Nemerle",
                    "nlogo" => "NetLogo",
                    "nrx" => "NetRexx",
                    "nl" => "NewLISP ",
                    "ns" => "Newspeak",
                    "nim" | "nims" => "Nim ",
                    "nix" => "Nix ", //  isn't a joke...
                    "nxc" => "NXC",
                    "nqc" => "NQC",
                    "nu" => "Nu",
                    "nsi" | "nsh" => "NSIS",
                    "nss" => "NWScript",
                    "ob" | "obn" => "Oberon",
                    "opa" => "Opa",
                    "cl" => "OpenCL ",
                    "qasm" => "OpenQASM",
                    "orc" => "Orc",
                    "oxygene" => "Oxygene",
                    "oz" => "Oz",
                    "p4" => "P4",
                    "parasail" => "ParaSail",
                    "gp" => "PARI/GP",
                    "pli" | "pl1" => "PL/I",
                    "pony" => "Pony",
                    "ps" => "PostScript",
                    "pov" => "POV-Ray SDL ",
                    "ps1" | "psm1" | "psd1" => "PowerShell ",
                    "pde" => "Processing",
                    "pl" | "pm" => "Prolog ",
                    "pml" => "Promela",
                    "pure" => "Pure",
                    "pd" => "Pure Data",
                    "purs" => "PureScript ",
                    "q" => "Q 󰫾",
                    "qs" => "Q#",
                    "qc" => "QuakeC",
                    "rkt" => "Racket",
                    "raku" | "p6" | "pm6" => "Raku",
                    "re" => "Reason",
                    "rebol" | "reb" | "r" => "REBOL",
                    "rc" => "Redcode",
                    "res" => "ReScript ",
                    "rexx" | "rx" => "REXX",
                    "sas" => "SAS",
                    "sa" => "Sather",
                    "sci" | "sce" => "Scilab",
                    "scratch" | "sb" | "sb2" | "sb3" => "Scratch",
                    "sed" => "Sed",
                    "sd7" => "Seed7",
                    "self" => "Self",
                    "sol" => "Solidity ",
                    "spl" => "SPARK",
                    "spin" => "SPIN",
                    "sql" => "SQL ", // maybe be ?
                    "nut" => "Squirrel ",
                    "do" | "ado" => "Stata ",
                    "sc" => "SuperCollider",
                    "swift" => "Swift ",
                    "tads" | "t" => "TADS",
                    "tcl" | "tk" => "Tcl",
                    "tex" | "sty" | "cls" => "TeX ",
                    "t" => "Turing",
                    "txl" => "TXL",
                    "uc" => "UnrealScript",
                    "v" => "V 󰬃",
                    "vala" | "vapi" => "Vala ",
                    "vim" | "vimrc" => "Vim script ",
                    "vy" => "Viper",
                    "wasm" | "wat" => "WebAssembly ",
                    "wl" | "nb" | "wls" => "Wolfram Language",
                    "x10" => "X10",
                    "xc" => "XC",
                    "xl" => "XL 󱎧",
                    "xojo" => "Xojo",
                    "xpl" => "XPL",
                    "xq" | "xqy" | "xquery" => "XQuery",
                    "xsl" | "xslt" => "XSLT",
                    "xtend" => "Xtend",
                    "yor" => "Yorick",
                    "zsh" => "Z Shell ",
                    "zpl" => "ZPL",
                    "zig" => "Zig ",
                    "zon" => "Zonnon",
                    "sass" => "Sass ",
                    "hell" => "Malbolge",
                    _ => "Unknown",
                }
            } else {
                "Unknown"
            });
            true
        } else {
            false
        }
    }

    pub fn load_diff(&mut self, path: &Path, content: Vec<CompactString>) {
        self.lines = content;
        if self.lines.is_empty() {
            self.lines.push(CompactString::default());
        }
        self.filepath = Some(path.to_path_buf());
        self.dirty = false;
        self.is_diff = true;
    }

    pub fn insert_char(&mut self, c: char) {
        self.delete_selection();
        let idx = Self::char_to_byte_idx(&self.lines[self.cursor_y], self.cursor_x);
        self.lines[self.cursor_y].insert(idx, c);
        self.cursor_x += 1;
        self.target_x = self.cursor_x;
    }

    pub fn insert_newline(&mut self) {
        self.delete_selection();
        let idx = Self::char_to_byte_idx(&self.lines[self.cursor_y], self.cursor_x);
        let remainder = self.lines[self.cursor_y].split_off(idx);
        self.cursor_y += 1;
        self.lines.insert(self.cursor_y, remainder);
        self.cursor_x = 0;
        self.target_x = self.cursor_x;
    }

    pub fn backspace_char(&mut self) {
        if self.delete_selection() {
            return;
        }

        if self.cursor_x > 0 {
            self.cursor_x = self.cursor_x.saturating_sub(1);
            let idx = Self::char_to_byte_idx(&self.lines[self.cursor_y], self.cursor_x);
            self.lines[self.cursor_y].remove(idx);
        } else if self.cursor_y > 0 {
            let current_line = self.lines.remove(self.cursor_y);
            self.cursor_y = self.cursor_y.saturating_sub(1);
            self.cursor_x = self.lines[self.cursor_y].chars().count();
            self.lines[self.cursor_y].push_str(&current_line);
        }
        self.target_x = self.cursor_x;
    }

    pub fn delete_char(&mut self) {
        if self.delete_selection() {
            return;
        }

        let line_len = self.lines[self.cursor_y].chars().count();

        if self.cursor_x < line_len {
            let idx = Self::char_to_byte_idx(&self.lines[self.cursor_y], self.cursor_x);
            self.lines[self.cursor_y].remove(idx);
        } else if self.cursor_y < self.lines.len() - 1 {
            let next_line = self.lines.remove(self.cursor_y + 1);
            self.lines[self.cursor_y].push_str(&next_line);
        }
        self.target_x = self.cursor_x;
    }

    // selection logic
    pub fn is_selected(&self, check_x: usize, check_y: usize) -> bool {
        if let Some((start_x, start_y)) = self.selection_start {
            let (first_x, first_y, last_x, last_y) = if start_y < self.cursor_y || (start_y == self.cursor_y && start_x < self.cursor_x) {
                (start_x, start_y, self.cursor_x, self.cursor_y)
            } else {
                (self.cursor_x, self.cursor_y, start_x, start_y)
            };

            if check_y < first_y || check_y > last_y {
                return false;
            }
            if check_y == first_y && check_y == last_y {
                return check_x >= first_x && check_x < last_x;
            }
            if check_y == first_y {
                return check_x >= first_x;
            }
            if check_y == last_y {
                return check_x < last_x;
            }
            return true;
        }
        false
    }

    fn char_to_byte_idx(s: &str, char_idx: usize) -> usize {
        s.char_indices().nth(char_idx).map(|(i, _)| i).unwrap_or(s.len())
    }

    pub fn delete_selection(&mut self) -> bool {
        if let Some((start_x, start_y)) = self.selection_start {
            let ((sy, sx), (ey, ex)) = if start_y < self.cursor_y || (start_y == self.cursor_y && start_x < self.cursor_x) {
                ((start_y, start_x), (self.cursor_y, self.cursor_x))
            } else {
                ((self.cursor_y, self.cursor_x), (start_y, start_x))
            };

            if sy == ey {
                let bs = Self::char_to_byte_idx(&self.lines[sy], sx);
                let be = Self::char_to_byte_idx(&self.lines[sy], ex);
                self.lines[sy].replace_range(bs..be, "");
            } else {
                let bs = Self::char_to_byte_idx(&self.lines[sy], sx);
                let mut new_start = CompactString::from(self.lines[sy][..bs].to_string());

                let be = Self::char_to_byte_idx(&self.lines[ey], ex);
                let new_end = self.lines[ey][be..].to_string();

                new_start.push_str(&new_end);

                self.lines.drain(sy..=ey);
                self.lines.insert(sy, new_start);
            }
            self.dirty = true;
            self.cursor_y = sy;
            self.cursor_x = sx;
            self.target_x = sx;
            self.selection_start = None;
            return true;
        }
        false
    }

    pub fn update_selection(&mut self, shift: bool) {
        if shift {
            if self.selection_start.is_none() {
                self.selection_start = Some((self.cursor_x, self.cursor_y));
            }
        } else {
            self.selection_start = None;
        }
    }

    // movement
    pub fn move_up(&mut self, shift: bool) {
        self.update_selection(shift);
        if self.cursor_y > 0 {
            self.cursor_y = self.cursor_y.saturating_sub(1);
            self.cursor_x = self.target_x.min(self.lines[self.cursor_y].len());
        }
    }

    pub fn move_down(&mut self, shift: bool) {
        self.update_selection(shift);
        if self.cursor_y < self.lines.len() - 1 {
            self.cursor_y += 1;
            self.cursor_x = self.target_x.min(self.lines[self.cursor_y].len());
        }
    }

    pub fn move_left(&mut self, shift: bool, ctrl: bool) {
        self.update_selection(shift);
        if self.cursor_x > 0 {
            if ctrl {
                let line = &self.lines[self.cursor_y];
                let bytes = line.as_bytes();

                // Skip all whitespace
                while self.cursor_x > 0 && bytes[self.cursor_x - 1] == b' ' {
                    self.cursor_x = self.cursor_x.saturating_sub(1);
                }

                // Skip what isn't whitespace
                while self.cursor_x > 0 && bytes[self.cursor_x - 1] != b' ' {
                    self.cursor_x = self.cursor_x.saturating_sub(1);
                }
            } else {
                self.cursor_x = self.cursor_x.saturating_sub(1);
            }
        } else if self.cursor_y > 0 {
            // Move to the end of the previous line
            self.cursor_y = self.cursor_y.saturating_sub(1);
            self.cursor_x = self.lines[self.cursor_y].len();
        }
        self.target_x = self.cursor_x;
    }

    pub fn move_right(&mut self, shift: bool, ctrl: bool) {
        self.update_selection(shift);

        let line_len = self.lines[self.cursor_y].len();
        if self.cursor_x < line_len {
            if ctrl {
                let line = &self.lines[self.cursor_y];
                let bytes = line.as_bytes();

                // Skip non-whitespace
                while self.cursor_x < line_len && bytes[self.cursor_x] != b' ' {
                    self.cursor_x += 1;
                }
                // Skip whitespace
                while self.cursor_x < line_len && bytes[self.cursor_x] == b' ' {
                    self.cursor_x += 1;
                }
            } else {
                self.cursor_x += 1;
            }
        } else if self.cursor_y < self.lines.len() - 1 {
            // Move to the start of the next line
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
        self.target_x = self.cursor_x;
    }

    fn get_selected_text(&self) -> Option<String> {
        let (start_x, start_y) = self.selection_start?;

        // Normalize coordinates (ensure we know which is start vs end)
        let ((sy, sx), (ey, ex)) = if start_y < self.cursor_y || (start_y == self.cursor_y && start_x < self.cursor_x) {
            ((start_y, start_x), (self.cursor_y, self.cursor_x))
        } else {
            ((self.cursor_y, self.cursor_x), (start_y, start_x))
        };

        if sy == ey {
            // Single line selection
            let line = &self.lines[sy];
            let bs = Self::char_to_byte_idx(line, sx);
            let be = Self::char_to_byte_idx(line, ex);
            Some(line[bs..be].to_string())
        } else {
            // Multi-line selection
            let mut result = String::new();

            // First line: from start_x to end
            let first_line = &self.lines[sy];
            let bs = Self::char_to_byte_idx(first_line, sx);
            result.push_str(&first_line[bs..]);
            result.push('\n');

            // Middle lines: full content
            for y in (sy + 1)..ey {
                result.push_str(&self.lines[y]);
                result.push('\n');
            }

            // Last line: from start to end_x
            let last_line = &self.lines[ey];
            let be = Self::char_to_byte_idx(last_line, ex);
            result.push_str(&last_line[..be]);

            Some(result)
        }
    }

    pub fn copy(&mut self) {
        if let Some(text) = self.get_selected_text() {
            if let Some(clipboard) = &mut self.clipboard {
                let _ = clipboard.set_text(text);
            }
        }
    }

    pub fn cut(&mut self) {
        if self.selection_start.is_some() {
            self.copy();
            self.delete_selection();
            self.dirty = true;
        }
    }

    pub fn paste(&mut self) {
        if let Some(clipboard) = &mut self.clipboard {
            if let Ok(text) = clipboard.get_text() {
                // If we have a selection, delete it first so we "replace" it
                self.delete_selection();

                let paste_lines: Vec<&str> = text.split('\n').collect();

                if paste_lines.is_empty() { return; }

                if paste_lines.len() == 1 {
                    // Simple single line paste
                    let idx = Self::char_to_byte_idx(&self.lines[self.cursor_y], self.cursor_x);
                    self.lines[self.cursor_y].insert_str(idx, paste_lines[0]);
                    self.cursor_x += paste_lines[0].chars().count();
                } else {
                    // Multi-line paste
                    let idx = Self::char_to_byte_idx(&self.lines[self.cursor_y], self.cursor_x);

                    // Split the current line at cursor
                    let current_line_suffix = self.lines[self.cursor_y].split_off(idx);

                    // Add the first part of the paste to the current line
                    self.lines[self.cursor_y].push_str(paste_lines[0]);

                    // Insert middle lines
                    for i in 1..paste_lines.len() - 1 {
                        self.lines.insert(self.cursor_y + i, CompactString::from(paste_lines[i]));
                    }

                    // Handle the last line of the paste
                    let last_paste_line = paste_lines.last().unwrap();
                    let mut new_last_line = CompactString::from(*last_paste_line);
                    let final_cursor_x = new_last_line.chars().count();
                    new_last_line.push_str(&current_line_suffix);

                    self.cursor_y += paste_lines.len() - 1;
                    self.lines.insert(self.cursor_y, new_last_line);
                    self.cursor_x = final_cursor_x;
                }

                self.target_x = self.cursor_x;
                self.dirty = true;
            }
        }
    }
}
