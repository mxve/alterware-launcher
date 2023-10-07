# IW4x
[github.com/iw4x/iw4x-client#command-line-arguments](https://github.com/iw4x/iw4x-client#command-line-arguments)

| Argument                | Description                                    |
|:------------------------|:-----------------------------------------------|
| `-tests`                | Perform unit tests.                            |
| `-entries`              | Print to the console a list of every asset as they are loaded from zonefiles. |
| `-stdout`               | Redirect all logging output to the terminal iw4x is started from, or if there is none, creates a new terminal window to write log information in. |
| `-console`              | Allow the game to display its own separate interactive console window. |
| `-dedicated`            | Starts the game as a headless dedicated server. |
| `-bigminidumps`         | Include all code sections from loaded modules in the dump. |
| `-reallybigminidumps`   | Include data sections from all loaded modules in the dump. |
| `-dump`                 | Write info of loaded assets to the raw folder as they are being loaded. |
| `-nointro`              | Skip game's cinematic intro.                   |
| `-version`              | Print IW4x build info on startup.              |
| `-nosteam`              | Disable friends feature and do not update Steam about the game's current status just like an invisible mode. |
| `-unprotect-dvars`      | Allow the server to modify saved/archive dvars. |
| `-zonebuilder`          | Start the interactive zonebuilder tool console instead of starting the game. |
| `-disable-notifies`     | Disable "Anti-CFG" checks |
| `-disable-mongoose`     | Disable Mongoose HTTP server |
| `-disable-rate-limit-check` | Disable RCOn rate limit checks |
| `+<command>`            | Execute game command (ex. `+set net_port 1337`)|


# S1-Mod, IW6-Mod
| Argument                | Description                                    |
|:------------------------|:-----------------------------------------------|
| `-headless`             | Use system console                             |
| `-dedicated`            | Dedicated server                               |
| `-singleplayer`         | Start singleplayer; Skip launcher              |
| `-multiplayer`          | Start multiplayer; Skip launcher               |
| `+<command>`            | Execute game command (ex. `+set net_port 1337`)|
