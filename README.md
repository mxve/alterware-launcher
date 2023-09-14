# AlterWare Launcher

### [AlterWare.dev](https://alterware.dev)

##### IW4x | IW4-SP | IW5-Mod | IW6-Mod | S1-Mod

---

#### Installation

1. Download the [latest alterware-launcher.exe](https://github.com/mxve/alterware-launcher/releases/latest/download/alterware-launcher.exe)
2. Place alterware-launcher.exe in the game directory
3. Run alterware-launcher.exe, after updating the game will launch automatically

---

#### Command line arguments

- ```iw4-sp```, ```iw4x```, ```iw5-mod```, ```iw6-mod```, ```s1-mod```
  - Skip automatic detection and launch the specified game
- ```--update```, ```-u```
  - Only update the game, don't launch it
- ```--skip-launcher-update```
  - Don't update the launcher
- ```--bonus```
  - Download bonus content
- ```--force```, ```-f```
  - Force file hash recheck

Example: ```alterware-launcher.exe iw4x --bonus -u```

Some arguments can be set in alterware-launcher.json, args generally override the values of the config.

---

#### Support

Visit the [AlterWare Forum](https://forum.alterware.dev/) or [Discord](https://discord.gg/2ETE8engZM) for support.

---

#### Building from Source

- [Install Rust](https://rustup.rs/)
- Clone the repository
- Run ```cargo build --release```
- The executable will be located in ```target/release```

---

### Note for server owners:
When the launcher updates itself it needs to be restarted. It will return exit code 201 in this case.

```
@echo off
:loop
start /wait alterware-launcher.exe update
if %errorlevel% equ 201 (
    goto loop
)
```