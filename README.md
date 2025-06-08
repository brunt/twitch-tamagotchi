# Twitch Tamagotchi

Implementation of a [Tamagotchi](https://en.wikipedia.org/wiki/Tamagotchi) that interacts via Twitch chat commands

* minimal configuration required: a `.env` file specifying a webserver port and twitch channel name
* responds to twitch chat but does not pollute chat with bot spam
* minimal resource requirements: uses about a dozen megabytes of RAM running on a raspberry pi

![Image](/media/tamagotchi.png)

_Note: Fonts may need to be installed for emoji support, I had to run `sudo apt install fonts-noto-color-emoji` on my raspberry pi for some emojis to show._