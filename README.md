<img src="./assets/azalea-logo.png"/>

Azalea is a compositor agnostic desktop shell for Wayland written in Rust.

## Compatibility

It runs on any compositor that implements wlroots' [Layer Shell
](https://wayland.app/protocols/wlr-layer-shell-unstable-v1) protocol.

- Hyprland
- Sway
- KWin
- Niri
- Wayfire
- [Others](https://wayland.app/protocols/wlr-layer-shell-unstable-v1#compositor-support)

## Crates
- `azalea`: main application
- `azalea-core`: Core library (cli, config parser, sockets, window management, etc)
- `azalea-log`: Wrappers around glib's log functions
- `azalea-service`: Service traits
- `azalea-derive`: Derive macros
- `azalea-shell`: Implementation of services and widgets

> [!TIP]
> Cargo docs: https://enzohideo.github.io/azalea-desktop-shell/docs/azalea/

## Nix

This project uses nix to package and test Azalea on different wayland
compositors.

### Development shell

```sh
nix shell
cargo run daemon start
```

### Build and run

```sh
nix run
```

### NixOS VM Tests

Testing azalea on different wayland compositors

```
# Automatized tests, screenshots are saved to ./result
nix build .#test

# Interactive tests
nix run .#test.driverInteractive
```

> [!NOTE]
> This uses a lot of resources to spin up the VM

## Without nix

It's possible to build the project without nix, but you'd need to install all
the dependencies yourself.

https://github.com/enzohideo/azalea-desktop-shell/blob/554a7c7858464344c8c5ad852ce8156ab1ef8c0f/nix/package.nix#L17-L40

## Useful resources

This project uses `gtk4` (relm4) and `gtk4-layer-shell`, but there are a few
other libraries that could have been used:

- `qt` + `layer-shell-qt` (poor Rust support, but it works great on C++)
- `iced` + `iced-layershell` (pure Rust library, but relatively poor performance)

Here are a few useful resources for learning `gtk4` and `relm4`.

- https://gtk-rs.org/gtk4-rs/stable/latest/book/
- https://relm4.org/book/stable/

There are also high level libraries made specifically for building desktop
shell components:

- [ags](https://aylur.github.io/ags/) / [astal](https://aylur.github.io/astal/) (gtk4)
- [quickshell](https://quickshell.org/) (qt)
- [Azalea](https://enzohideo.github.io/azalea-desktop-shell/docs/azalea/) (this project)

## Other interesting Wayland protocols

Here are some interesting protocols that are worth exploring:

- [ext-session-lock-v1](https://wayland.app/protocols/ext-session-lock-v1) (useful for lock screens, widely supported)
- [ext-workspace-v1](https://wayland.app/protocols/ext-workspace-v1) (useful for workspace managers, but it's fairly recent, so support is limited)
