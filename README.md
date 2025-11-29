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
- `azalea-shell`: Implementation of the services and widgets

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
nix run .#test
```

> [!NOTE]
> This uses a lot of resources to spin up the VM

## Without nix

It's possible to build the project without nix, but you'd need to install all
the dependencies yourself.

- gtk4
- gtk4-layer-shell
- openssl
- dbus
- alsa
- clang
- mold
