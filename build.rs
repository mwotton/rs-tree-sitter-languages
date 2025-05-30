use std::{
    cell::OnceCell,
    env,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug)]
struct Sitter {
    path: PathBuf,
    src: PathBuf,
    lang: String,
    version: OnceCell<String>,
}

impl Sitter {
    fn new(feature: String) -> Option<Self> {
        let sitter = match feature.split_once('-') {
            Some((dir, lang)) => {
                let path = PathBuf::from(format!("sitters/tree-sitter-{dir}"));
                let src = path.join(lang).join("src");

                Self {
                    path,
                    src,
                    lang: lang.to_owned(),
                    version: OnceCell::new(),
                }
            }
            None => {
                let lang = feature;
                let path = PathBuf::from(format!("sitters/tree-sitter-{lang}"));
                // Support grammars with nested src directories (e.g., PHP)
                let mut src = path.join("src");
                if !src.exists() {
                    src = path.join(&lang).join("src");
                }

                Self {
                    path,
                    src,
                    lang,
                    version: OnceCell::new(),
                }
            }
        };

        if !sitter.src.exists() {
            return None;
        }

        Some(sitter)
    }

    fn src(&self) -> PathBuf {
        self.src.clone()
    }

    fn version(&self) -> &str {
        self.version.get_or_init(|| {
            let version_file = self.path.join("treesitter-language-version");
            if let Ok(version) = std::fs::read_to_string(version_file) {
                return version.trim().to_owned();
            }

            let output = Command::new("git")
                .arg("rev-parse")
                .arg(format!("HEAD:{}", self.path.display()))
                .output()
                .expect("git rev-parse")
                .stdout;

            String::from_utf8(output).expect("utf-8").trim().to_owned()
        })
    }

    fn grammar(&self) -> Option<PathBuf> {
        let p = self.path.join("grammar.js");
        p.exists().then_some(p)
    }

    fn node_types(&self) -> Option<PathBuf> {
        let p = self.path.join("src").join("node-types.json");
        p.exists().then_some(p)
    }

    fn highlight(&self) -> Option<PathBuf> {
        let p = self.path.join("queries").join("highlights.scm");
        p.exists().then_some(p)
    }

    fn injections(&self) -> Option<PathBuf> {
        let p = self.path.join("queries").join("injections.scm");
        p.exists().then_some(p)
    }

    fn locals(&self) -> Option<PathBuf> {
        let p = self.path.join("queries").join("locals.scm");
        p.exists().then_some(p)
    }

    fn tags(&self) -> Option<PathBuf> {
        let p = self.path.join("queries").join("tags.scm");
        p.exists().then_some(p)
    }
}

fn get_sitters() -> impl Iterator<Item = Sitter> {
    env::vars()
        .filter_map(|(name, _)| {
            name.strip_prefix("CARGO_FEATURE_")
                .map(|x| x.replace('_', "-").to_lowercase())
        })
        .filter_map(Sitter::new)
}

fn ts_highlight() -> bool {
    env::var("CARGO_FEATURE_TS_HIGHLIGHT").is_ok()
}

fn compile_sitter(sitter @ Sitter { lang, .. }: &Sitter) -> bool {
    let src = sitter.src();
    let parser = src.join("parser.c");
    let scanner_c = src.join("scanner.c");
    let scanner_cc = src.join("scanner.cc");

    let mut needs_cpp = false;

    cc::Build::new()
        .flag_if_supported("-w")
        .include(&src)
        .file(&parser)
        .compile(&format!("{lang}-parser"));

    println!("cargo:rerun-if-changed={}", parser.display());

    if scanner_c.exists() {
        cc::Build::new()
            .flag_if_supported("-w")
            .include(&src)
            .file(&scanner_c)
            .compile(&format!("{lang}-scanner_c"));

        println!("cargo:rerun-if-changed={}", scanner_c.display());
    }

    if scanner_cc.exists() {
        let mut cc = cc::Build::new();

        cc.cpp(true)
            .flag_if_supported("-w")
            .include(&src)
            .file(&scanner_cc);

        // Static linking does not work on Mac.
        if !cfg!(target_os = "macos") {
            cc.static_flag(true).cpp_link_stdlib(None);
            needs_cpp = true;
        }

        cc.compile(&format!("{lang}-scanner_cc"));

        println!("cargo:rerun-if-changed={}", scanner_cc.display());
    }

    needs_cpp
}

fn write_parser(sitter @ Sitter { lang, .. }: &Sitter) -> std::io::Result<()> {
    let dest_path = Path::new(&env::var_os("OUT_DIR").unwrap()).join(format!("lang_{lang}.rs"));
    let mut output = File::create(dest_path)?;

    let version = sitter.version();

    writeln!(
        output,
        r#"
        use tree_sitter::Language;

        extern "C" {{
            fn tree_sitter_{lang}() -> Language;
        }}

        /// Get the tree-sitter [Language][] for this grammar.
        ///
        /// [Language]: https://docs.rs/tree-sitter/*/tree_sitter/struct.Language.html
        pub fn language() -> Language {{
            unsafe {{ tree_sitter_{lang}() }}
        }}

        /// Get the commit hash or version of this grammar.
        ///
        /// Current version: `{version}`.
        pub const fn version() -> &'static str {{
            "{version}"
        }}
    "#
    )?;

    macro_rules! include {
        ($name:expr, $path:expr) => {{
            let path = $path.canonicalize().unwrap();
            let path = path.strip_prefix(env!("CARGO_MANIFEST_DIR")).unwrap();
            writeln!(output, "/// {}", $name)?;
            writeln!(
                output,
                "pub const {}: &str = include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}\"));",
                $name,
                path.display()
            )?;
        }};
    }

    if let Some(grammar) = sitter.grammar() {
        include!("GRAMMAR", grammar);
    }
    if let Some(node_types) = sitter.node_types() {
        include!("NODE_TYPES", node_types);
    }
    if let Some(highlight) = sitter.highlight() {
        include!("HIGHLIGHT_QUERY", highlight);
    }
    if let Some(injections) = sitter.injections() {
        include!("INJECTION_QUERY", injections);
    }
    if let Some(locals) = sitter.locals() {
        include!("LOCALS_QUERY", locals);
    }
    if let Some(tags) = sitter.tags() {
        include!("TAGS_QUERY", tags);
    }

    if ts_highlight() {
        let highlight = sitter
            .highlight()
            .map(|_| "HIGHLIGHT_QUERY")
            .unwrap_or("\"\"");
        let injection = sitter
            .injections()
            .map(|_| "INJECTION_QUERY")
            .unwrap_or("\"\"");
        let locals = sitter.locals().map(|_| "LOCALS_QUERY").unwrap_or("\"\"");

        writeln!(
            output,
            r#"
            use tree_sitter_highlight::HighlightConfiguration;

            /// Get the tree-sitter [HighlightConfiguration][] for this grammar.
            ///
            /// [HighlightConfiguration]: https://docs.rs/tree-sitter-highlight/*/tree_sitter_highlight/struct.HighlightConfiguration.html
            pub fn highlight() -> HighlightConfiguration {{
                HighlightConfiguration::new(
                    language(),
                    {highlight},
                    {injection},
                    {locals},
                ).unwrap()
            }}
        "#
        )?;
    }

    writeln!(output, "#[cfg(test)]\nmod tests {{")?;
    writeln!(
        output,
        r#"
        #[test]
        fn test_print_version() {{
            println!("{{}}", super::version());
        }}
        #[test]
        fn test_can_load_grammar() {{
            let mut parser = tree_sitter::Parser::new();
            parser
                .set_language(&super::language())
                .expect("Error loading {lang} language");
        }}
    "#
    )?;
    if ts_highlight() {
        writeln!(
            output,
            r#"
            #[test]
            fn test_can_create_highlight() {{
                let _ = super::highlight();
            }}
        "#
        )?;
    }
    writeln!(output, "}}")?;

    Ok(())
}

pub fn main() {
    let mut needs_cpp = false;

    for sitter in get_sitters() {
        if compile_sitter(&sitter) {
            needs_cpp = true;
        }

        write_parser(&sitter).unwrap();
    }

    if needs_cpp {
        static_link_with_cpp();
    }
}

fn static_link_with_cpp() {
    let compiler = cc::Build::new().cpp(true).get_compiler();

    for (name, file) in [("stdc++", "libstdc++.a"), ("c++", "libc++.a")] {
        let out = compiler
            .to_command()
            .args(["--print-file-name", file])
            .output()
            .unwrap()
            .stdout;
        let out = String::from_utf8(out).unwrap();
        let path = Path::new(out.trim());

        if path.is_relative() {
            continue;
        }

        if let Some(parent) = path.parent() {
            println!("cargo:rustc-link-search={}", parent.display());
            println!("cargo:rustc-link-lib=static={name}");
            break;
        }
    }
}
