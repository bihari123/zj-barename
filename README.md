# zellij tab-bar (no "Zellij" prefix)

A minimal fork of [Zellij](https://github.com/zellij-org/zellij)'s built-in
`tab-bar` plugin. It is **pixel-identical to the default tab bar** with one
change: the leading `Zellij` word is removed, so the bar shows just the session
name.

```
before:   Zellij (my-session)  Tab #1  Tab #2
after:              (my-session)  Tab #1  Tab #2
```

Based on Zellij **v0.44.3**.

## What was changed

Only two edits versus the upstream `tab-bar` plugin:

1. `src/line.rs` — the prefix string `" Zellij "` is replaced with `" "`, so the
   session name (`(name)`) still renders, just without the leading word.
2. `src/main.rs` — a one-time `request_permission(...)` for
   `ReadApplicationState` + `ChangeApplicationState` is added in `load()`.
   The built-in plugin doesn't need this because built-ins are implicitly
   trusted; a plugin loaded from a file is not, so this makes Zellij ask for
   permission **once** (then it's remembered) instead of on every tab change.

## Build

Requires a Rust toolchain and the `wasm32-wasip1` target.

```bash
./build.sh            # builds target/wasm32-wasip1/release/tab-bar.wasm
./build.sh --install  # also copies it to ~/.config/zellij/plugins/tab-bar.wasm
```

## Install manually

Copy the built `tab-bar.wasm` into your Zellij plugins directory:

```bash
cp target/wasm32-wasip1/release/tab-bar.wasm ~/.config/zellij/plugins/tab-bar.wasm
```

Then use it in a layout in place of the built-in `tab-bar`. A drop-in default
layout (`~/.config/zellij/layouts/default.kdl`):

```kdl
layout {
    pane size=1 borderless=true {
        plugin location="file:~/.config/zellij/plugins/tab-bar.wasm"
    }
    pane
    pane size=1 borderless=true {
        plugin location="status-bar"
    }
}
```

Start a **new** session to see it (existing sessions keep the old bar until
restarted). On first load Zellij will ask to grant the plugin permission — press
`y` once.

## Version compatibility

Zellij's plugin ABI can change between releases. This is built against the
`zellij-tile` / `zellij-tile-utils` `0.44.3` crates. If you upgrade Zellij to a
version with a different plugin ABI, bump those versions in `Cargo.toml`
(and re-apply the two edits if upstream `tab-bar` source changed), then rebuild.

## License

MIT. This is a derivative work of the Zellij `tab-bar` plugin; see
[`LICENSE`](./LICENSE) for the original Zellij copyright and attribution.
