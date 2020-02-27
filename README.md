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

Additional dependencies (the bot still works without these)

+ ffmpeg (for music related commands)
+ youtube-dl (for listening music on youtube, even it hasn't supported yet tho)

```bash
sudo pacman -S ffmpeg python
python install youtube-dl
```
(*I use Arch btw*)<br>

Then clone this repo, navigate into it and do  

```bash
cargo build --release --bin tomoka_rs
```
*You may see some warning about unused variable stuff due to the incomplete `radio` command*
 
The whole process should be 
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo pacman -S ffmpeg python # Change this to your package manager
python install youtube-dl
git clone https://gitlab.com/tmokenc/tomoka-rs
cd tomoka-rs
cargo build --release --bin tomoka_rs
```
## Usage

Add your bot token to the environment

```bash
export DISCORD_TOKEN="your-extremely-secret-token"
```

Then run it

```bash
cargo run --release --bin tomoka_rs
```

*There are a bunch of useless binaries in the codebase, so we must specific the bot binary here*

## Structure

The structure of this codebase (kind of mess though) <br>
(*Navigate into each folder to read its README*) <br>
(*Still nothing yet ~~thanks to my laziness~~*)
+ [Discord](discord) (The main bot source code)
+ [Server](server) (The web server for the bot, but nothing is there yet)
+ [Db](db) (Database related stuff)
+ [Magic](magic) (All the magic in one place)
+ [Requester](requester) (Request to external API)
+ [Pokemon](pokemon) (just there for the furute of Discord Pokemon Battle game)
+ [Bins](bins) (useless binaries)

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
- [ ] Web server 
    - [ ] GitLab webhook
    - [ ] Web documentation

## Support

Feel free to give me a DM on discord `@tmokenc#0001`, a guy with extremely cute Kanna avatar.

## Contributing

Pull requests are welcome. Bugs fix and new commands are extremely welcome. But please don't change the exist functionalities

## License
[MIT](https://choosealicense.com/licenses/mit/)
