# fledge-plugin-envcheck

A [fledge](https://github.com/CorvidLabs/fledge) WASM plugin that validates `.env` files against their example templates (`.env.example`, `.env.sample`, `.env.template`).

Catches configuration drift before it becomes a runtime error: missing keys, empty values, and undocumented extras.

## Install

```sh
fledge plugins install corvid-agent/fledge-plugin-envcheck
```

## Usage

```sh
fledge plugins run envcheck
```

The plugin scans the project root and subdirectories for `.env.example`, `.env.sample`, or `.env.template` files, then compares each against its corresponding `.env`. It reports:

- **Missing keys** -- defined in the template but absent from `.env`
- **Empty values** -- key present in `.env` but set to an empty string
- **Extra keys** -- present in `.env` but not defined in any template

Common directories like `node_modules`, `target`, and `vendor` are skipped automatically.

## Capabilities

This is a WASM plugin (sandboxed, cross-platform). It requires only `filesystem="project"` for read access to `.env` files. No network, exec, or store access is needed.

## Build

Requires `wasm32-wasip1` target:

```sh
rustup target add wasm32-wasip1
cargo build --target wasm32-wasip1 --release
```

## License

MIT
