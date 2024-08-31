# AlterWare Launcher

### [AlterWare.dev](https://alterware.dev)

##### IW4x | IW4-SP | IW5-Mod | IW6-Mod | S1-Mod

![GitHub tag (with filter)](https://img.shields.io/github/v/tag/mxve/alterware-launcher?filter=!v*-pre&style=flat-square&label=Latest%20release&labelColor=F3F8FF&color=E26EE5) ![GitHub (Pre-)Release Date](https://img.shields.io/github/release-date-pre/mxve/alterware-launcher?style=flat-square&label=Release%20date&labelColor=F3F8FF&color=E26EE5) ![GitHub all releases](https://img.shields.io/github/downloads/mxve/alterware-launcher/total?style=flat-square&label=Total%20downloads&labelColor=F3F8FF&color=E26EE5)

---

**Only legitimate copies of the games are supported. If you don't own the game, please buy it.**

---

## Installation

### Windows

1. Download the game from [Steam](https://store.steampowered.com/).
2. Download the [latest alterware-launcher.exe](https://github.com/mxve/alterware-launcher/releases/latest/download/alterware-launcher.exe).
3. Place `alterware-launcher.exe` in the game directory.
4. Run `alterware-launcher.exe`. After updating, the game will launch automatically.

### Debian/Ubuntu

1. Follow the instructions provided in this [forum post](https://forum.alterware.dev/t/linux-debian-ubuntu-amd64-arm64-install-the-alterware-launcher-using-our-apt-repository/1117) to install the alterware-launcher using our self-hosted APT repository.
2. Once installed, run the launcher to automatically download the client files. For updates, use the terminal with the `--update` command line argument.

---

#### Command line arguments

- ```iw4-sp```, ```iw4x```, ```iw5-mod```, ```iw6-mod```, ```s1-mod```
  - Skip automatic detection and launch the specified game
    - This should always be the first argument if used
- ```--help```
  - Print help
- ```--update```, ```-u```
  - Only update the game, don't launch it
- ```--skip-launcher-update```
  - Don't update the launcher
- ```--bonus```
  - Download bonus content
- ```--force```, ```-f```
  - Force file hash recheck
- ```--path```, ```-p```
  - Set the game path
    - Do not include a trailing backslash in the path
- ```--pass```
  - Pass additional arguments to the game
    - See [client-args.md](client-args.md)
- ```--version```, ```-v```
  - Print the launcher version
- ```--ignore-required-files```
  - Install client even if required files are missing
- ```--skip-redist```
  - Skip redistributable installation
- ```--redist```
  - (Re-)install redistributables

Example: ```alterware-launcher.exe iw4x --bonus -u --path "C:\Games\IW4x" --pass "-console"```

Some arguments can be set in alterware-launcher.json, args generally override the values of the config.

---

#### Config file
alterware-launcher.json

- ```update_only```
  - See --update
  - Default: false
- ```skip_self_update```
  - See --skip-launcher-update
  - Default: false
- ```download_bonus_content```
  - See --bonus
  - Default: false
- ```ask_bonus_content```
  - Ask the user if they want to download bonus content
  - Default: true; false after asking
- ```force_update```
  - See --force
  - Default: false
- ```args```
  - See --pass
  - Default: ""
- ```use_https```
  - Use HTTPS for downloads
  - Default: true
- ```skip_redist```
  - Skip redistributable installation
  - Default: false

---

#### Support

Visit the [AlterWare Forum](https://forum.alterware.dev/) or [Discord](https://discord.gg/2ETE8engZM) for support.

---

#### Building from Source

- [Install Rust](https://rustup.rs/)
- [Linux/unix] Make sure perl is installed<sup>1</sup>
- Clone the repository
- Run ```cargo build --release```
- The executable will be located in ```target/release```

---

### Note for server owners:
When the launcher updates itself __on Windows__ it will restart by spawning a new console. If you are automating this process, you should probably use ```--skip-launcher-update``` and download the latest launcher yourself from [here](https://github.com/mxve/alterware-launcher/releases/latest/download/alterware-launcher.exe).

The linux build does __not__ update itself.

---

<sup>1</sup> [Required for OpenSSL](https://docs.rs/openssl/latest/openssl/#vendored)
