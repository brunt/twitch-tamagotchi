[![Ko-fi](https://img.shields.io/badge/Ko--fi-Support%20me-orange?logo=kofi)](https://ko-fi.com/ubruntu65048)

# Twitch Tamagotchi

Implementation of a [Tamagotchi](https://en.wikipedia.org/wiki/Tamagotchi) that interacts via Twitch chat commands

* minimal configuration required: a `.env` file specifying a webserver port and twitch channel name
* responds to twitch chat but does not pollute chat with bot spam
* minimal resource requirements: uses about a dozen megabytes of RAM running on a raspberry pi

![Image](/media/tamagotchi.png)

## How to Use
* compile the code by installing [rust](https://www.rust-lang.org/) and running `cargo build --release`
* Configure `.env` file with desired channel name and port for webserver
* Run the compiled binary either locally or on a different pc if you like
* In OBS or other streaming software, create a browser source pointing to the ip address of the computer where `twitch-tamagotchi` is running, and the port specified in the `.env` file
* Type commands in the specified twitch channel's chat and watch the tamagotchi react to commands

_Note: Fonts may need to be installed for emoji support, I had to run `sudo apt install fonts-noto-color-emoji` on my raspberry pi for some emojis to show._