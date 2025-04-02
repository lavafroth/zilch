# Zilch

A vanilla tauri app to rid Android devices of software bloat.  
Keep none of 'em, none, nada, zilch.

![What the app looks like](assets/preview.png)

> [!WARNING]  
> This app is not production ready, some features are missing.

### Features

#### Simple yet powerful UI

The UI is somewhat inspired by file managers. Clicking a row will select it as
well as toggle the collapsed state to display extra information.

Multiple entries can be selected by `Ctrl` clicking them, just like in file
managers. Alternatively, one can use the multi-select button in the bottom
action row to achieve the same. You can always cancel a selection by hitting escape.

#### Intuitive keyboard shortcuts

- Press `S` or `/` to jump to the search field. Inspired by [docs.rs](https://docs.rs).
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

### Features not implemented yet

- [ ] Save button + shortcut
- [ ] Knowledge base
- [ ] Recommendation categories
- [ ] Make uninstall and disable options depend on Android SDK

### Build from source

```sh
nix develop
tailwindcss --output src/styles.css --input src/input.css
cargo tauri dev
```

### How is this associated with Universal Android Debloater?

I plan to import their knowledge-base in the future.
I have been a contributor to the original repo as well as the now maintained fork.

I started this project because of the following reasons:
- The current UI written in iced-rs is clunky due to limitations of the library.
- UAD requires the ADB binary whereas we use our custom implementation of ADB written in pure Rust.
