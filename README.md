# Zilch

Purge Android bloat with confidence.

![How the app currently looks](./assets/Screenshot_20251223_113343.png)

## Features

- Single compiled binary, just download and run
- No dependency on external ADB tooling
- Click on app entries to select them
- Double click to expand
- Clear multi-selection by pressing `Escape`
- Extracts app labels via package manager API
- Architecture independent, works with any Android device over USB
- Accidentally removed apps can be restored via the revert button
- Recommendation categories (borrowed from UAD)
- Press `S` or `/` or `Ctrl` `F` to search apps
- Save the current state of packages on the phone with `Ctrl` `S`
- Uninstall and disable operations are agnostic to Android version

## Installation

### From binary releases

Precompiled binaries are available under the releases tab.

### Build from source

```sh
nix develop
cargo run
```
