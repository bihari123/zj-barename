# zellij tab-bar (no "Zellij" prefix)

A fork of [Zellij](https://github.com/zellij-org/zellij)'s built-in `tab-bar`
plugin with two tweaks: the leading `Zellij` word is removed (so the bar shows
just the session name), and an optional right-aligned **HH:MM clock**.

```
before:   Zellij (my-session)  Tab #1  Tab #2
after:              (my-session)  Tab #1  Tab #2                    14:22
```

Based on Zellij **v0.44.3**.

## What was changed

Versus the upstream `tab-bar` plugin:

1. `src/line.rs` — the prefix string `" Zellij "` is replaced with `" "`, so the
   session name (`(name)`) still renders, just without the leading word.
2. `src/main.rs` — a one-time `request_permission(...)` for
   `ReadApplicationState` + `ChangeApplicationState` in `load()` (a file-loaded
   plugin isn't implicitly trusted like a built-in), so Zellij asks **once**.
3. `src/main.rs` — an optional right-aligned 24-hour clock, updated once a
   minute via a timer.

### Config (set in the layout's `plugin { … }` block)

| Key | Default | Meaning |
| --- | --- | --- |
| `clock` | `true` | Show the right-aligned `HH:MM` clock. Set `"false"` to hide it. |
| `utc_offset` | `+00:00` | Offset from UTC for the clock, e.g. `"+05:30"` (IST), `"-08:00"`. The plugin reads the system (UTC) clock and applies this offset — there is no timezone database, so pick the fixed offset for your zone. |

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
        plugin location="file:~/.config/zellij/plugins/tab-bar.wasm" {
            utc_offset "+05:30"   // your UTC offset for the clock; omit to disable via clock "false"
        }
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
