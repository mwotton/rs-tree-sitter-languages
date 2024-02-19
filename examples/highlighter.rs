use html_escape::encode_text;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::{env::args, fs::read_to_string};
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

const HIGHLIGHT_NAMES: &[&str; 19] = &[
    "attribute",
    "constant",
    "function.builtin",
    "function",
    "keyword",
    "operator",
    "property",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "string",
    "string.special",
    "tag",
    "type",
    "type.builtin",
    "comment",
    "variable",
    "variable.builtin",
    "variable.parameter",
];

lazy_static! {
    static ref FILETYPES: HashMap<&'static str, &'static str> = HashMap::from([
        ("md", "markdown"),
        ("markdown", "markdown"),
        ("rs", "rust"),
        ("toml", "toml"),
        ("js", "javascript"),
        ("ts", "javascript"),
        ("html", "html"),
        ("vue", "html"),
        ("tera", "html"),
        ("css", "css"),
        ("c", "c"),
        ("cc", "c"),
        ("cpp", "cpp"),
        ("sh", "shells"),
        ("bash", "shells"),
        ("zsh", "shells"),
        ("lua", "lua"),
        ("py", "python"),
        ("yml", "yaml"),
        ("go", "go"),
        ("haskell", "haskell"),
        ("d", "d"),
        ("java", "java"),
        ("vim", "vim"),
    ]);
    static ref CONFIGS: HashMap<&'static str, HighlightConfiguration> = HashMap::from(
        [
            ("vim", pepegsitter::vim::highlight()),
            ("rust", pepegsitter::rust::highlight()),
            ("toml", pepegsitter::toml::highlight()),
            ("javascript", pepegsitter::javascript::highlight()),
            ("typescript", pepegsitter::typescript::highlight()),
            ("html", pepegsitter::html::highlight()),
            ("css", pepegsitter::css::highlight()),
            ("c", pepegsitter::c::highlight()),
            ("cpp", pepegsitter::cpp::highlight()),
            ("shells", pepegsitter::bash::highlight()),
            ("shells", pepegsitter::bash::highlight()),
            ("lua", pepegsitter::lua::highlight()),
            ("python", pepegsitter::python::highlight()),
            ("yaml", pepegsitter::yaml::highlight()),
            ("go", pepegsitter::go::highlight()),
            ("haskell", pepegsitter::haskell::highlight()),
            ("d", pepegsitter::d::highlight()),
            ("java", pepegsitter::java::highlight()),
            ("markdown", pepegsitter::markdown::highlight()),
        ]
        .map(|(key, mut val)| {
            val.configure(HIGHLIGHT_NAMES);
            (key, val)
        })
    );
}

/// An example file highlighter supporting [CONFIGS] filetypes. Run eg on itself with :
/// `cargo r --example=highlighter -- examples/highlighter.rs > highlighter.html`
fn main() {
    let arguments: Vec<_> = args().into_iter().collect();
    if arguments.len() != 2 {
        panic!("\nSyntax: highlighter text_file");
    }
    let file_name = arguments[1].clone();
    let text_content = read_to_string(&file_name).expect("readable file in text_file");
    let mut highlighted_text = highlight(&file_name, &text_content);
    highlighted_text = format!(
        r#"
<!DOCTYPE html>
<html>
    <head>
        <meta charset="UTF-8">
        <title>{file_name}</title>
        {STYLE}
    </head>
    <body>
        <pre>
{highlighted_text}
        </pre>
    </body>
</html>
    "#
    );
    println!("{highlighted_text}");
}

/// Highlight `text` using `filename`'s extension to guess content type.
/// The output is some html using [HIGHLIGHT_NAMES] span classes.
/// See [STYLE] for some basic styling.
pub fn highlight(filename: &str, text: &str) -> String {
    let mut highlighter = Highlighter::new();
    let extension = filename.split(".").last().unwrap();

    let Some(filetype) = FILETYPES.get(extension) else {
        eprintln!(
            " > highlighting: unrecognized extension '{}' with file '{}'.",
            extension, filename
        );

        return encode_text(&text).into_owned();
    };

    eprintln!(" > highlighting file {filename:?} with type {filetype:?}");
    let highlights = highlighter
        .highlight(
            CONFIGS.get(filetype).unwrap(),
            text.as_bytes(),
            None,
            |injected| {
                eprintln!(" > highlighting injected content with type {injected:?}");

                CONFIGS.get(injected)
            },
        )
        .unwrap();

    let mut highlighted_text = String::new();
    for event in highlights {
        match event.unwrap() {
            HighlightEvent::Source { start, end } => {
                highlighted_text =
                    format!("{}{}", highlighted_text, encode_text(&text[start..end]));
            }
            HighlightEvent::HighlightStart(s) => {
                highlighted_text = format!(
                    "{}<span class=\"{}\">",
                    highlighted_text,
                    HIGHLIGHT_NAMES[s.0].replace(".", " ")
                );
            }
            HighlightEvent::HighlightEnd => {
                highlighted_text = format!("{}</span>", highlighted_text);
            }
        }
    }

    highlighted_text
}

const STYLE: &str = r#"
        <style>
        body {
            background-color: var(--bg);
            color: var(--fg);
        }
        .comment {
            color: var(--other);
        }
        .attribute {
            color: var(--fg-less);
        }
        .constant {
            color: var(--info);
        }
        .function.builtin {
            color: var(--fg-less);
        }
        .function {
            color: var(--fg-less);
        }
        .keyword {
            color: var(--fg-less);
        }
        .operator {
            color: var(--fg-less);
        }
        .property {
            color: var(--fg-less);
        }
        .punctuation {
            color: var(--fg-lesser);
        }
        .punctuation.bracket {
            color: var(--fg-lesser);
        }
        .punctuation.delimiter {
            color: var(--fg-lesser);
        }
        .string {
            color: var(--info);
        }
        .string.special {
            color: var(--special);
        }
        .tag {
            color: var(--fg-less);
        }
        .type {
            color: var(--fg-less);
        }
        .type.builtin {
            color: var(--fg-less);
        }
        body {
            --fg: black;
            --fg-less: #777;            /* dark grey */
            --fg-lesser: #aaa;          /* light grey */
            --bg-less: #f3f3f3;         /* lighter grey */
            --bg: white;                /* white */
            --ok: #79d907;              /* green */
            --err: #e51426;             /* red */
            --warn: #ee5e12;            /* orange */
            --info: #0060df;            /* blue */
            --other: #03b5b5;           /* cyan */
            --special: #9b1ddf;         /* magenta */
            --caret: #EE4EB8;           /* pink */
        }
        @media(prefers-color-scheme: dark) {
            body {
                --fg: white;
                --fg-less: #ccc;        /* lighter grey */
                --fg-lesser: #777;      /* light grey */
                --bg-less: #222323;     /* dark grey */
                --bg: black;            /* black */
                --ok: #A4CC35;          /* green */
                --err: #FF4050;         /* red */
                --warn: #F28144;        /* orange */
                --info: #9cd6ff;        /* cyan */
                --special: #CC78FA;     /* magenta */
                --caret: #F553BF;       /* pink */
            }
        }
        </style>
"#;
