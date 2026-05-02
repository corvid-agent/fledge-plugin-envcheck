# fledge-plugin-envcheck

WASM plugin for fledge — validate `.env` files against `.env.example`, find missing or extra variables.

## Install

```sh
fledge plugins install corvid-agent/fledge-plugin-envcheck
```

## Run

```sh
fledge plugins run envcheck
```

Scans the project root (and subdirectories) for `.env.example`, `.env.sample`,
or `.env.template` files and compares them against the corresponding `.env`.
Reports missing keys, empty values, and extra keys not in the template.

## Details

This is a WASM plugin (sandboxed, cross-platform). It requires the
`filesystem="project"` capability to read `.env` files from the project
directory. No network or exec access is needed.

## License

MIT
