mod classify;
mod emit;
mod spec;

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("codegen") => {
            let mut check = false;
            for arg in &args[1..] {
                if arg == "--check" {
                    check = true;
                } else {
                    eprintln!("unrecognised argument: {arg}");
                    eprintln!("usage: cargo xtask codegen [--check]");
                    std::process::exit(2);
                }
            }
            codegen(check)
        }
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
    if !check {
        // --check is a read-only verification gate; only create the directory
        // when we are actually about to write generated files into it.
        std::fs::create_dir_all(&out_dir).unwrap();
    }
    for (file, content) in &files {
        let formatted = rustfmt(&root, file, content);
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

    // files the generator no longer produces must not linger. Only entries that
    // are BOTH a regular file AND have a `.rs` extension are considered ours to
    // manage: this directory can also contain things we do not own (.DS_Store,
    // editor swapfiles, stray subdirectories) which must never be reported as
    // stale or deleted. Do not "simplify" this filter away.
    if let Ok(read_dir) = std::fs::read_dir(&out_dir) {
        for entry in read_dir {
            let entry = entry.unwrap();
            let name = entry.file_name().into_string().unwrap();
            let is_rs_file = entry.file_type().unwrap().is_file() && name.ends_with(".rs");
            if is_rs_file && !files.contains_key(&name) {
                if check {
                    stale.push(format!("{name} (orphaned)"));
                } else {
                    std::fs::remove_file(out_dir.join(&name)).unwrap();
                    println!("removed orphaned {name}");
                }
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
fn rustfmt(workspace_root: &Path, file: &str, source: &str) -> String {
    let mut child = Command::new("rustfmt")
        .args(["--edition", "2024"])
        .current_dir(workspace_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| match e.kind() {
            std::io::ErrorKind::NotFound => panic!("rustfmt not found on PATH"),
            _ => panic!("failed to spawn rustfmt: {e}"),
        });
    child.stdin.take().unwrap().write_all(source.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        panic!("rustfmt rejected generated code for {file}:\n{stderr}");
    }
    String::from_utf8(out.stdout).unwrap()
}
