# pet-friend-dock

**Sulivan Minder** is a COSMIC desktop applet (Pop!_OS / COSMIC) written in Rust: a little dog that walks back and forth along your panel, runs when the CPU is busy, and pops up your reminders.

The applet lives in [`sulivan-minder/`](sulivan-minder/) and has no Python dependency.

## Features

- Animated dog sprites that walk along the dock and run faster when CPU usage is above a threshold (default 20%).
- Click the pet to open a popup where you enter a reminder and how many minutes from now it should fire.
- When a reminder is due, the popup opens and shows its message. Each reminder is one-shot: it is removed from the applet's own saved config when it fires. The system crontab is never touched.
- Reminders and settings persist via `cosmic-config`.

## Layout

```text
sulivan-minder/
├── Cargo.toml / build.rs / justfile
├── i18n/en/            # Fluent translations
├── resources/          # desktop entry, metainfo, icon
└── src/
    ├── main.rs         # entrypoint
    ├── app.rs          # AppModel: view, popup, subscriptions, reminders
    ├── pet.rs          # movement/animation logic (unit tested)
    ├── config.rs       # persisted config (cpu_threshold, reminders)
    ├── tracker/        # CPU usage sampling
    └── assets/         # dog sprites (64x64 PNG, facing left)
```

## Prerequisites

- Rust toolchain via [rustup](https://rustup.rs/) (edition 2024, so a recent stable)
- [just](https://github.com/casey/just) (optional but recommended)
- System development libraries. On Debian/Ubuntu/Pop!_OS:

```bash
sudo apt install build-essential pkg-config libxkbcommon-dev libwayland-dev \
  libinput-dev libudev-dev libseat-dev libgbm-dev libegl1-mesa-dev \
  libfontconfig-dev libfreetype-dev libssl-dev
```

## Build and run

```bash
cd sulivan-minder
just build-release     # or: cargo build --release
just install           # installs binary, desktop entry, metainfo and icon (uses sudo/prefix as needed)
```

Then add the applet from **COSMIC Settings → Desktop → Panel/Dock → Applets**.

Running the binary directly (`just run` / `cargo run`) requires a running COSMIC panel session, since it is a panel applet. Use `just uninstall` to remove it.

## Contributing

1. Fork and clone the repository, then create a branch: `git checkout -b feature/my-change`.
2. Make focused changes inside `sulivan-minder/`.
3. Before opening a PR, run from `sulivan-minder/`:

```bash
cargo fmt
cargo test            # unit tests (pet movement, CPU threshold)
just check            # clippy
cargo build
```

4. Open a Pull Request against `main` describing the motivation, changes and how you tested them.

Guidelines:

- Keep UI updates non-blocking: do blocking work (such as `/proc` sampling) in `spawn_blocking` tasks, as `CpuTracker` does.
- Keep pure logic (like `pet.rs`) free of UI types so it can be unit tested.
- User-facing strings go in `i18n/en/sulivan_minder.ftl` and are used via `fl!()`. To add a language, copy `i18n/en` to a new ISO 639-1 directory and translate.
- Do not modify or delete entries in the user's system crontab; reminders are app-managed.
- To replace the sprites, keep four same-size PNG frames facing left and update the list in `app.rs`.

See [`sulivan-minder/README.md`](sulivan-minder/README.md) for packaging and vendoring details.

## License

GPL-3.0. See [LICENSE](LICENSE).
