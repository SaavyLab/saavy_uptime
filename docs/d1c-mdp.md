# d1c Roadmap (Near Term)

We have the CLI skeleton, `d1c init`, migration replayer, schema dumping, and basic codegen loops working. Next we need to turn the prototypes into usable features so real projects can drop d1c into their Workers repo. Below is a short list of immediate priorities.

## 1. Parameter + schema inference

- Use the in-memory `rusqlite::Connection` (after migrations replay) to prepare each query and inspect `statement.column_count()`, `statement.column_name(i)`, and `statement.column_decltype(i)` so we can auto-generate row structs.
- Inspect `statement.parameter_count()` and expose either `parameter_name(i)` or parse named parameters (`:param`) directly to build function arguments. Fall back to `-- params` comments when we cannot discover names (SQLite doesn’t expose named parameters reliably).
- Deduplicate row struct definitions across queries (e.g., `GetMonitorRow` reused for both `:one` and `:many` queries).

## 2. Improve generated code

- Emit the full module once (header imports + row structs + functions) instead of printing to stdout.
- Add gating/comparison: `d1c gen --check` should fail if the generated file differs; `d1c gen` should write over `out_dir/module_name.rs`.
- Add the ability to run `rustfmt` (feature flag or CLI flag), or at least instruct users to run `cargo fmt` afterwards.
- Emit proper return types instead of `worker::Result<...>` placeholders, using the cardinality + inferred row struct.

## 3. Config + workflow polish

- Use `d1c.toml` for all commands; load defaults before prompting.
- Expand `d1c init` to create `queries_dir/example.sql`, `out_dir/mod.rs`, etc., to give users a starting point.
- Document the CLI flags (`d1c gen --config <path>`, `--check`, `--schema <path>`).
- Integrate with Taskfile (already done), but add `task d1c:gen` that runs `d1c gen --config ...`.

## 4. Parsing improvements

- Support additional query metadata (e.g., `-- doc: ...` for comments, `-- returns:auto` for auto-generated row structs).
- Update the parser to capture `MULTILINE SELECT` reliably (already working) and ignore SQL block comments (`/* ... */`) when looking for metadata.
- Add validation: ensure `:param` references are declared either via `-- params` or discovered automatically; emit helpful errors when cardinality + SQL mismatch (e.g., `:one` on an update).

## 5. Testing + docs

- Add unit tests for the parser, migration replayer, and codegen (e.g., tests under `tests/` that run queries against a sample schema).
- Document the CLI usage and the query format in `crates/d1c/README.md` (currently a pitch; we need “How to run” info, examples for `init/gen/dump-schema`, and expected output).
- Provide a sample Worker that uses the generated code (e.g., `examples/saavy-uptime`).

Once these five buckets are in place, d1c will be ready for early adoption inside Saavy Uptime. After that we can iterate on type inference, `serde` support, `RETURNING` statements, `progress` commands, etc.
