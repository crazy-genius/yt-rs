# generator

Code generator for the [`yt-rs`](../yt-rs) YouTrack API client. It reads the
YouTrack OpenAPI document and emits all **218** Rust model types — as committed,
deterministic, `rustfmt`-clean source — into `crates/yt-rs/src/models/generated/`.

The generated code is checked into the repository; this crate is the tool that
produces it, not a build dependency of `yt-rs`.

## Usage

Run through the workspace alias (defined in `.cargo/config.toml`):

```sh
cargo xtask codegen          # regenerate crates/yt-rs/src/models/generated/
cargo xtask codegen --check  # verify the committed output is up to date (CI gate)
```

`--check` exits non-zero when the checked-in files differ from a fresh
generation or when a stray `.rs` file has been orphaned. It is wired into
`make lint` so staleness fails the build. Unknown arguments are rejected rather
than silently falling through to a destructive write.

Input: `docs/openapi.json` (repository root). Output: 8 domain modules
(`activity`, `admin`, `agile`, `article`, `common`, `issue`, `project`, `user`)
plus `mod.rs`, ~6,200 lines total.

## How it works

The pipeline is four small modules, each independently unit-tested (31 tests):

| Module | Responsibility |
| --- | --- |
| `spec.rs` | Parse the OpenAPI JSON; resolve `allOf` inheritance by inlining; classify every property into a small `PropType` (string / i32 / i64 / bool / json / ref / array / string-enum). |
| `classify.rs` | Identify the **20 polymorphic roots** (schemas with a `discriminator.mapping`); assign every schema to a domain module via an explicit table (an unassigned schema is a hard error); compute which schemas are referenced directly as field types. |
| `emit.rs` | Emit the Rust source: plain structs, string enums with lossless `Other(String)` fallbacks, the root enums, and their accessors. |
| `main.rs` | CLI, `rustfmt` formatting via subprocess, atomic write / `--check` staleness comparison, orphan cleanup. |

### Type mapping highlights

- Every field is `Option<T>` with `#[serde(skip_serializing_if = "Option::is_none")]` — the spec declares nothing required.
- Direct `$ref` fields are `Option<Box<T>>` (the spec has reference cycles); array elements are never boxed.
- The **20 roots** become a forward-compatible pair: an untagged `Known(<Root>Kind) / Unknown(serde_json::Value)` wrapper over a `#[serde(tag = "$type")]` enum, so a `$type` from a newer YouTrack degrades to `Unknown` instead of failing the whole parse. The root's own payload is `<Root>Data`. Accessors are generated for the properties every variant shares.
- Of the **143** discriminator-mapping variants, `$type` is kept only on the **49** that are also referenced directly as a field type (reachable standalone, where serde has no enum to supply the tag) and stripped on the other **94**; the **75** non-variant schemas keep it. This reachability rule is what makes values round-trip losslessly in both directions.

Determinism is structural — `BTreeMap`/`BTreeSet` throughout, no `HashMap` — so regeneration is byte-identical and `--check` never flaps.

## Refreshing the spec

Replace `docs/openapi.json`, run `cargo xtask codegen`, and run the test suite.
The generator asserts its invariants (schema/root/variant counts, the dual-use
split, domain coverage) and will fail loudly on anything it does not recognise —
a new schema with no domain assignment, an unexpected property type, or a
discriminator shape it has not seen — rather than emitting silently-wrong code.

---

<sub>🤖 This crate was implemented with [Claude Code](https://claude.com/claude-code) (Opus 4.8), test-driven and reviewed task-by-task.</sub>
