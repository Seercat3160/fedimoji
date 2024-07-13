
# fedimoji

A system to display custom emojis in Minecraft, using a server-provided resource pack.
It works with completely Vanilla clients, as long as they accept the resource pack.

On the server side, it currently requires use of the Styled Chat fabric mod with some configuration.
Of course, the server must also be configured to present the resource pack to clients.

A companion server-only mod is planned, which will add some commands for searching and previewing the available emoji
without having to send one in chat just to see it. That mod will probably not remove the need for Styled Chat, as that would be
much more of a hassle and isn't really needed.

## How it works

TL;DR: We assign a codepoint in Unicode's Private Use Areas to each emoji, and put all the emoji images in one image file
so Minecraft can use it. We add to the definition of the default Minecraft font so that it displays these characters using the emoji.
Styled Chat is configured to allow people to type (for example) `:neofox:` and have it be mapped to the correct special character when sent.

This repository contains a tool to take a bunch of emoji packs as input and output all the files needed to actually put it in Minecraft.

It resizes all the emoji to 8x8 pixels - that's the resolution they're displayed at in game to fit with normal text, so why store them any larger
(clients need to download this when joining, remember) - and combines them into one image. This font bitmap is 16 glyphs wide (same as the Minecraft fonts,
I don't know if there's any reason for this to be honest) and as many tall as is needed.

To each emoji it assigns a unique Unicode codepoint in the range `U+E000` to `U+F8FF`, `U+F0000` to `U+FFFFD`, or `U+100000` to `U+10FFFD`.
It then makes a set of strings that tell Minecraft which codepoints map to where in the provided image. Each string represents a new row in the image,
and each character is what character is to use the glyph in the corresponding position in the image. Blank parts of the image are `\u0000`, as each string
must define the same number of glyphs.

## How to use it

At the moment, this process isn't fully automated. You can't just provide a few URLs or zip files of emoji packs and have it just work.
Maybe in the future.

**Note**: This probably uses a lot of RAM while generating the glyph map, given it loads all the files into memory and processes them there.

For now, here are the steps:

1. Just dump a bunch of square PNGs in `./emoji/` with the file names being the names they'll be used as in-game.
2. Run the program. `cargo run --release` (`--release` for performance reasons) should do. If it gives any errors or advice, take note of those.
3. Arrange the output files in the following way.

I've provided a template for the resource pack (`./template-resource-pack/`), containing everything that isn't specific to your input. Make a copy of that (or don't).
You'll be putting some of the generated files in there to make it yours. Further docs assume it's been copied to `./pack/`, such that `./pack/pack.mcmeta` exists.

The following files will be output into `./out/`, and **existing files there will be overwritten.**

### `emoji.png`

This is the font bitmap, an atlas of all the custom glyphs.

It is copied to `./pack/assets/fedimoji/textures/font/emoji.png`.

### `emoji.json`

This is the definition of the font provider, which tells Minecraft how to display the custom glyphs.

It is copied to `./pack/assets/fedimoji/font/include/emoji.json`.

### `fedimoji.json`

This tells the Styled Chat mod how to map emoji names (like `:neofox:`) to character codepoints.

You will need to perform some configuration of Styled Chat.

First, copy `fedimoji.json` from the output directory to `config/fedimoji.json` in your Minecraft server.

Next, add an emoticon entry to your Styled Chat config:

- Assuming you're using the `default` style (adapt this yourself if not),
add the entry `"$default:from_file:fedimoji.json": "${emoji}"` to `default.emoticons`.
- I would also recommend changing `auto_completion.emoticons` to `true` so that people can tab-complete the emoji names in chat.
- I'd also remove the `"$emojibase:builtin:joypixels"` entry under `emoticons`, and perhaps all the others if you want, so that people can only use the emoji you've added. They'll look way better, anyway.

Here's a JSON snippet of those changes, for clarity:

```json
{
    "auto_completion": {
        "emoticons": true
    },
    "default": {
        "emoticons": {
            "$default:from_file:fedimoji.json": "${emoji}"
        }
    }
}
```

### Resource Pack Preparation

You need to turn the `./pack/` dir (or whatever you used) into a ZIP file, and configure your server to give it to clients.

Before you do that, however, you can make any changes you want to the pack. Here's my recommendations:

- Edit the `LICENSE` file I have in there in the template, crediting all the artists whose work you're redistributing.
Remember, you'll be serving this resource pack file to everyone who joins your server.
- Edit the `description` key in `pack.mcmeta` to something you want.
- Change the `pack.png`.

## Conclusion

Sorry for the lacklustre documentation (and general UX), I'm mostly making this for myself. It got enough fedi interaction, though, that I thought I should at least write something about how to use it. And besides, I'm assuming anyone doing this knowns enough about doing admin stuff that they can figure it out.
