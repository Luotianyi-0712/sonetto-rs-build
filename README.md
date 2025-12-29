# This is just a compiled repository, please go to the [original repository](https://github.com/Yoshk4e/sonetto-rs).

Rust can't run on my computer, so I created this repository.

## Current supported version: **3.1 (non-steam)**
PS was made to use the pc [version](https://download.bluepoch.com/en/Reverse1999_v3.1.0.38_en.exe) of the game.

## Thks
-Yoshk4e

## Use

Download latest build from [releases page](https://github.com/Luotianyi-0712/sonetto-rs-build/releases/tag/latest)

Actually, you only need to move the `sdkserver` executable into the `gameserver` folder. The `data/` folder is already included in the build, so no additional setup is required. Ensure your folder structure looks like this:
```text
.
├── sdkserver.exe
├── gameserver.exe
├── Config.toml
└── data/
```
- need to use the [sonetto patch](https://github.com/yoncodes/sonetto-patch) to make the game work with the server
- now open two terminals or command prompts

```bash
    .\sdkserver
```
```bash
    .\gameserver
```
- Login with email. **NOT REGISTER** if the account doesn't exist it will be created automatically
![login image](/images/r99-email.png)


