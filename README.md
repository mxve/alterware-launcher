# alterware-launcher

1. Download [latest release](https://github.com/mxve/alterware-launcher/releases/latest/download/alterware-launcher-x86_64-pc-windows-msvc.zip)
2. Unpack the archive and place alterware-launcher.exe in the game directory
3. Run alterware-launcher.exe, after updating the game will launch automatically

---

- Passing ```iw4-sp```, ```iw5-mod```, ```iw6-mod``` or ```s1-mod``` as the first argument will skip automatic game detection
- Passing ```update``` will stop the launcher from launching the game
- ```skip-launcher-update``` skips self-update

### Note for server owners:
When the launcher updates itself it needs to be restarted. It will return exit code 201 in this case.