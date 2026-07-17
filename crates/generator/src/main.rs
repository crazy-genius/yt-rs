mod classify;
mod emit;
mod spec;

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("codegen") => codegen(args.iter().any(|a| a == "--check")),
        _ => {
            eprintln!("usage: cargo xtask codegen [--check]");
            std::process::exit(2);
        }
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
}

fn codegen(check: bool) {
    let root = workspace_root();
    let spec_path = root.join("docs/openapi.json");
    let out_dir = root.join("crates/yt-rs/src/models/generated");

    let raw = std::fs::read_to_string(&spec_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", spec_path.display()));
    let mut spec = spec::Spec::parse(&raw);
    classify::apply_overrides(&mut spec);
    let classified = classify::classify(&spec);
    let files = emit::emit_all(&spec, &classified);

    let mut stale = Vec::new();
    std::fs::create_dir_all(&out_dir).unwrap();
    for (file, content) in &files {
        let formatted = rustfmt(&root, content);
        let path = out_dir.join(file);
        if check {
            let on_disk = std::fs::read_to_string(&path).unwrap_or_default();
            if on_disk != formatted {
                stale.push(file.clone());
            }
        } else {
            std::fs::write(&path, formatted)
                .unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
        }
    }

    // files the generator no longer produces must not linger
    for entry in std::fs::read_dir(&out_dir).unwrap() {
        let name = entry.unwrap().file_name().into_string().unwrap();
        if !files.contains_key(&name) {
            if check {
                stale.push(format!("{name} (orphaned)"));
            } else {
                std::fs::remove_file(out_dir.join(&name)).unwrap();
                println!("removed orphaned {name}");
            }
        }
    }

    if check {
        if stale.is_empty() {
            println!("generated models are up to date");
        } else {
            eprintln!("generated models are stale, run `cargo xtask codegen`:");
            for f in stale {
                eprintln!("  {f}");
            }
            std::process::exit(1);
        }
    } else {
        println!("wrote {} files to {}", files.len(), out_dir.display());
    }
}

/// Format through the same rustfmt (and rustfmt.toml, via cwd) the repo uses,
/// so `cargo fmt-check` agrees with `--check` byte-for-byte.
fn rustfmt(workspace_root: &Path, source: &str) -> String {
    let mut child = Command::new("rustfmt")
        .args(["--edition", "2024"])
        .current_dir(workspace_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("rustfmt not found on PATH");
    child.stdin.take().unwrap().write_all(source.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "rustfmt rejected generated code:\n{source}");
    String::from_utf8(out.stdout).unwrap()
}
