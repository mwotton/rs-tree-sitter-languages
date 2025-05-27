#![doc = include_str!("../README.md")]

macro_rules! add_parser {
    ($lang:ident, $feature:expr, $path:expr) => {
        /// Tree-Sitter parser for this grammar.
        #[cfg(feature = $feature)]
        pub mod $lang {
            include!(concat!(env!("OUT_DIR"), $path));
        }
    };
}

add_parser!(bash, "bash", "/lang_bash.rs");
add_parser!(c, "c", "/lang_c.rs");
add_parser!(cpp, "cpp", "/lang_cpp.rs");
add_parser!(css, "css", "/lang_css.rs");
add_parser!(d, "d", "/lang_d.rs");
add_parser!(go, "go", "/lang_go.rs");
add_parser!(haskell, "haskell", "/lang_haskell.rs");
add_parser!(html, "html", "/lang_html.rs");
add_parser!(java, "java", "/lang_java.rs");
add_parser!(javascript, "javascript", "/lang_javascript.rs");
add_parser!(json, "json", "/lang_json.rs");
add_parser!(lua, "lua", "/lang_lua.rs");
add_parser!(markdown, "markdown", "/lang_markdown.rs");
add_parser!(python, "python", "/lang_python.rs");
add_parser!(rust, "rust", "/lang_rust.rs");
add_parser!(toml, "toml", "/lang_toml.rs");
add_parser!(tsx, "typescript-tsx", "/lang_tsx.rs");
add_parser!(typescript, "typescript-typescript", "/lang_typescript.rs");
add_parser!(vim, "vim", "/lang_vim.rs");
add_parser!(yaml, "yaml", "/lang_yaml.rs");
add_parser!(elixir, "elixir", "/lang_elixir.rs");
add_parser!(erlang, "erlang", "/lang_erlang.rs");
add_parser!(perl, "perl", "/lang_perl.rs");
add_parser!(php, "php", "/lang_php.rs");
add_parser!(ruby, "ruby", "/lang_ruby.rs");
add_parser!(vbscript, "vbscript", "/lang_vbscript.rs");
