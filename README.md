# Tauri + Unison

This template should help get you started developing with Tauri and
Unison. This example has been built assuming the use of the
[tv](https://share.unison-lang.org/@dfreeman/tv) library.

## Developing

### Dependencies

* [Tauri](https://tauri.app/). Only tested so far with the `cargo install` method.
* [Unison](https://www.unison-lang.org/)

### Workflow

It's pretty manual right now.

1. Copy your version of UCM to `src-tauri/binaries/ucm-TARGET`, where `TARGET` is a rust target triple for your platform, like `aarch64-apple-darwin`. You can find this by running `rustc -vV` and looking for the `host:` field.
2. From inside UCM, develop your TV app as normal.
3. From inside UCM, run `compile app.serve main.uc`, where `app.serve` is the name of your entry function. It should be of type `'{IO, Exception} r`
4. Copy `main.uc` to `src-tauri/resources/main.uc`.
5. Run `cargo tauri dev` to open up a window and test.

To make changes to your app, close the window, and repeat steps 2-5.

### Refining

Consult the [Tauri docs](https://tauri.app/v1/guides/features/) for how to
customize the icons, change default window size, and more.

### Publishing

To publish for your current platform, run `cargo tauri build`.

### Recommended IDE Setup

If you don't already have a preference, try this setup:

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) + [Unison](https://marketplace.visualstudio.com/items?itemName=unison-lang.unison)
