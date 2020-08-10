# tomoka rs

A ~~buggy~~ Discord bot written in [Rust](https://rust-lang.org) <br>
This is a personal project for my own habit. <br>
There are **a lot of** typo, grammar errors, bad naming, redundant code, etc... <br> 
And **MOST** of the functionalities is for my own server, eg. __RGB__.

## Installation

Since it is written purely in Rust, you have to [install Rust](https://www.rust-lang.org/tools/install) with version >1.39 <br>
If you are running Linux, it will be as easy as 
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Then clone this repo, navigate into it
```bash
git clone https://gitlab.com/tmokenc/tomoka-rs
cd tomoka-rs
```
And then build the bot
```bash
cargo build --release
```
*You may see some warning about unused variable stuff due to the incomplete `radio` command*
 
The whole process should be 
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://gitlab.com/tmokenc/tomoka-rs
cd tomoka-rs
cargo build --release
```
## Usage

Add your bot token to the environment

```bash
export DISCORD_TOKEN="your-extremely-secret-token"
```

Then run it

```bash
cargo run --release
```

## Todo

- [ ] Music
    - [ ] Youtube playblack
    - [ ] Radio
- [ ] Games
    - [x] TouhouMusicQuest
    - [ ] TicTacToe
    - [ ] Chess
    - [ ] Tetris
    - [ ] PokemonBattle (maybe it never done though)
- [ ] Pokemon
    - [x] Smogon strategy
    - [ ] Pokemon info
    - [ ] Ability
    - [x] Nature
    - [ ] Move
- [ ] Utility
    - [ ] Time
        - [x] Display time in multiple timezone
        - [ ] Configurable timezones
        - [ ] Lunar time
    - [ ] E-hentai
        - [x] Gallery information
        - [ ] Read
        - [ ] Downloader
    - [ ] Encode
    - [ ] Decode
- [ ] Better guild option
- [ ] Better documentation

## Support

Feel free to give me a DM on discord `@tmokenc#0001`, a guy with extremely cute Kanna avatar.

## Contributing

Pull requests are welcome. Bugs fix and new commands are extremely welcome. But please don't change the exist functionalities

## License
[MIT](https://choosealicense.com/licenses/mit/)
