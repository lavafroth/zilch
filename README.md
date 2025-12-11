# Zilch

Rid Android devices of software bloat.  
Keep none, nada, zilch.

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

Correlating package names (`org.company.package`) to app labels
from the app drawer can be tricky. Zilch automatically parses APK files
from the device and displays their labels.

#### Architecture independent

Zilch works with any Android device built on any architecture as long as it can connect over USB.

#### Undo

Apps accidentally disabled or uninstalled via Zilch can be restored using the revert button.
Zilch will always backup your app before deletion.

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
