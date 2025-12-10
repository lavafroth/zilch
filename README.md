# Zilch

A vanilla tauri app to rid Android devices of software bloat.  
Keep none of 'em, none, nada, zilch.

![What the app looks like](assets/preview.png)

> [!WARNING]  
> This app is not production ready, some features are missing.

### Features

#### Simple UI

- Click on app entries to select them and view details.
- `Ctrl` click or use the multi-select button to select multiple apps.
- Clear multi-selection by pressing `Escape`.
- Press `S` or `/` to search apps.
- Save the current configuration with `Ctrl` + `S` (not yet implemented)
- Switch categories with `Ctrl` + `Tab` (not yet implemented)

#### Extracts app names by parsing APK files on device

It isn't always obvious from the package names like `com.oppo.brjl` what app it correlates to
in the app drawer. Don't fret, we do the legwork for you. Zilch automatically tries to pull
the APK files if they exist on the device, parses them and displays their label.

#### Architecture independent

Zilch works with any Android device built on any architecture as long as it can connect over USB.

#### Undo

If you accidentally disabled or uninstalled a system app, selecting the package entry will
display the revert button in the bottom action row. Click it to undo the deletion and restore the app.

### Not yet implemented

- [ ] Save button + shortcut
- [ ] Recommendation categories (WIP)
- [ ] Make uninstall and disable options depend on Android SDK

### Build from source

```sh
nix develop
tailwindcss --output src/styles.css --input src/input.css
cargo tauri dev
```
