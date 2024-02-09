# Tauri + Unison

This template may help get you started developing with Tauri and
Unison. This example has been built assuming the use of the
[tv](https://share.unison-lang.org/@dfreeman/tv) library.


## Licensing and attribution

The bundled example is from [@dreeman's tvGuide](https://share.unison-lang.org/@dfreeman/tvGuide).

This example is licensed under the
[CC0](https://creativecommons.org/public-domain/cc0/) - this places this work
as fully as possible into the public domain. Tauri and Unison are covered by
their own licenses. If you fork this code, you should modify the license and
authorship information as you like.


## How it works

[Tauri](https://tauri.app/) is a toolkit that lets developers package web-based software as a
native-looking application. If you're familiar with Electron, it's similar to
that.

[Unison](https://www.unison-lang.org/) is a programming language with a whole bunch of cool ideas.

This project bundles a copy of the `ucm` Unison executable/runtime along with
`main.uc`, which is copy of the compiled Unison code. When the user launches
your app, in the background we boot up the Unison-based server, wait for it to
to be ready, and then display a web view. We connect over port 8080 (hard-coded).

TODO: Can we connect over a local socket, or at least dynamically choose an open port?

## Developing

### Dependencies

* [Tauri](https://tauri.app/). Only tested so far with the `cargo install` method.
* [Unison](https://www.unison-lang.org/)

### Workflow

Run `cargo tauri dev` to build and bring up your application in Tauri. You can
change the fields in `Cargo.toml` under `package.metadata.unison_tauri` to
point to the project you want to use, adjust the UCM version, etc.

Currently, this isn't well set up for interactive development - I'd recommend using
[hotswap](https://share.unison-lang.org/@dfreeman/hotswap) until you get your
app behaving as expected, and then work on the Tauri bundling.


#### Old Workflow

If the above doesn't work for you, here's the manual process:

1. Copy your version of UCM to `src-tauri/binaries/ucm-TARGET`, where `TARGET` is a rust target triple for your platform, like `aarch64-apple-darwin`. You can find this by running `rustc -vV` and looking for the `host:` field.
2. From inside UCM, develop your TV app as normal.
3. From inside UCM, run `compile app.serve main.uc`, where `app.serve` is the name of your entry function. It should be of type `'{IO, Exception} r`
4. Copy `main.uc` to `src-tauri/resources/main.uc`.
5. Run `cargo tauri dev` to open up a window and test.

To make changes to your app, close the window, and repeat steps 2-5.

You may want to use [hotswap](https://share.unison-lang.org/@dfreeman/hotswap),
which gives a lower-friction development loop here.

TODO: Improve automation here - possibly by expanding the build.rs script.

### Refining your app

Consult the [Tauri docs](https://tauri.app/v1/guides/features/) for how to
customize the icons, change default window size, and more.

### Publishing your app

To publish for your current platform, run `cargo tauri build`.

TODO: Set up Github Actions to build for other platforms.

### Recommended IDE Setup

If you don't already have a preference, try this setup:

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) + [Unison](https://marketplace.visualstudio.com/items?itemName=unison-lang.unison)
