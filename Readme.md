# Spin Clean Plugin

## Install

`spin plugin install --url https://raw.githubusercontent.com/skabber/spin-clean-plugin/main/clean/clean.json`

## Usage

Create a `spin-clean.toml` in the same directory as your `spin.toml`.
This file should contain a list of components with a `[component.clean]` command per component.

```toml
[[component]]
id = "spin-id-1"
[component.clean]
command = "cargo clean"

[[component]]
id = "spin-id-2"
[component.clean]
command = "rm -rf ./build"
```

Then run:

`spin clean`

## Building

```bash
git clone https://github.com/skabber/spin-clean-plugin.git
cd spin-clean-plugin
git submodule update --init
cd clean
cargo build
```

## TODO

- [x] Build Action
- [x] Finish clean/clean.json
- [ ] Add flag to delete .spin/
