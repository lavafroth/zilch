# Zilch

Purge Android bloat with confidence.

> [!WARNING]  
> This app is not production ready, some features are missing.

## Features

### Simple UI

- Click on app entries to select them
- Double click to expand
- Clear multi-selection by pressing `Escape`
- Press `S` or `/` to search apps (not yet implemented)
- Save the current configuration with `Ctrl` + `S` (not yet implemented)
- Switch categories with `Ctrl` + `Tab` (not yet implemented)
- Extracts app labels via package manager API
- Architecture independent, works with any Android device over USB
- Accidentally removed apps can be restored via the revert button

### Not yet implemented

- [ ] Save button + shortcut
- [ ] Recommendation categories (WIP)
- [ ] Make uninstall and disable options depend on Android SDK

### Build from source

```sh
nix develop
cargo run
```
