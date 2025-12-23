# Zilch

Purge Android bloat with confidence.

![How the app currently looks](./assets/Screenshot_20251223_113343.png)

> [!WARNING]  
> This app is not production ready, some features are missing.

## Features

- Click on app entries to select them
- Double click to expand
- Clear multi-selection by pressing `Escape`
- Extracts app labels via package manager API
- Architecture independent, works with any Android device over USB
- Accidentally removed apps can be restored via the revert button
- Recommendation categories (borrowed from UAD)

### Not yet implemented

- Save button + shortcut
- Make uninstall and disable options depend on Android SDK
- Press `S` or `/` to search apps (not yet implemented)
- Save the current configuration with `Ctrl` + `S` (not yet implemented)
- Switch categories with `Ctrl` + `Tab` (not yet implemented)

### Build from source

```sh
nix develop
cargo run
```
