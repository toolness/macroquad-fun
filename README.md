## Introduction

This is a prototype for a non-violent 2D platformer using the [macroquad][] Rust library, with character art by [LuizMelo][].

Presently the game consists of a single level which requires the player to collect gems. Once the player has collected all the gems, there is nothing left to do (they have effectively won the game).

[The game can be played online here.](https://toolness.github.io/macroquad-fun/)

## Project goals

The goals for this project were:

- I wanted more experience writing Rust for game development (see the [architecture](#architecture) section for some learnings).

- I wanted to tinker with non-violent game and level design. I was particularly intrigued by [LDtk](https://ldtk.io/) and wanted to play with it (see the [level design process](#level-design-process) section for some learnings).

- I liked LuizMelo's art and wanted to make something with it.

[macroquad]: https://macroquad.rs/
[luizmelo]: https://luizmelo.itch.io/

## Quick start

### Desktop

```
cargo run
```

### Web

```
cargo install basic-http-server

sh ./build-wasm.sh

basic-http-server .
```

Then open your web browser to http://localhost:4000/.

Whenever you make a code change, you'll need to re-run `sh ./build-wasm.sh`.

### Distribution

To distribute the Web build, run:

```
sh ./dist-wasm.sh
```

Then copy the `dist` directory to a static file server.

### Deployment

To deploy the `dist` directory to GitHub Pages, run:

```
npm run deploy
```

### Web analytics server

When users start the web version, they're prompted to submit their gameplay data for analytics purposes. If they consent, the related data is sent to a server as they play (for more details on what's in the data, see the [analytics](#analytics) section).

The `server` subdirectory contains a simple server that receives this data and writes it to disk.

To build and run the server:

```
cd server
cargo run
```

Accessing the web version at `localhost` will automatically use this server.

Note that accessing the web version at any _other_ hostname (including an IP address, even `127.0.0.1`) will cause the game to submit any data to my personal analytics server hosted at `macroquad-fun.toolness.org`. Ideally, this should be made more configurable.

## Architecture

Originally, this game used an object-oriented architecture. But because Rust doesn't have many affordances for object-oriented programming, I quickly ran into problems.

This led me to re-watching Catherine West's RustConf 2018 closing keynote and reading her associated blog post, [Using Rust For Game Development](https://kyren.github.io/2018/09/14/rustconf-talk.html), which fortunately described a journey much like the one I was taking: developing a Rust game in OO and running into lots of issues.

At this point, at of the end of 2022, the architecture has, with the immense help of West's materials, evolved into an architecture that is essentially an "array of structs" ECS--what West calls a bare minimum ECS system. It lacks the cache locality optimizations afforded by a "struct of arrays" style ECS.

It also uses a HashMap for all the entities instead of a vector with generational indices, which is probably not great for performance, but the actual game isn't demanding enough for this to be an issue right now.

Good entrypoints for understanding the architecture can be found in [`level_runtime.rs`](./src/level_runtime.rs) and [`entity.rs`](./src/entity.rs).

## Analytics

The web version of the game uses no third-party analytics services: instead, data is sent to a custom Rust server. The value of the `t` querystring argument, if any, is associated with a random UUID for each playthrough, along with the event data described below.

### Event data

Internally, game logic runs at a fixed 60 frames per second, and only has a few buttons: left, right, and jump.

The game uses a compact [postcard][]-based recording format that logs button up/down events with the frame number they occurred at. This allows playthroughs to be recorded at the cost of a few hundred bytes, and played back with full fidelity.

### More details

For more details on recording and playing back sessions on the desktop version, run the game's executable with the `--help` flag.

For more details on the implementation details of the recording format, see [`recorder.rs`](./src/recorder.rs).

[postcard]: https://docs.rs/postcard/latest/postcard/

## Level design process

I approached the design of the game's level with the following process:

1. I wrote out a text document as per Steve Lee's [How I design levels in text first, and why](https://www.youtube.com/watch?v=0FSssDWEFLc). This outlined the main mechanics I wanted to showcase, as well as the narrative beats.

2. Informed by the text document, I sketched out a general plan for the level on a sheet of graph paper.

3. I went into LDtk and created a level based on the sketch, and further iterated on the design within LDtk.

Designing the level and playing through it myself made me realize that some of the core mechanics needed tweaking. For example, the player jumped so far horizontally that it was very difficult for me to make parts of the level that couldn't simply be jumped to. This led me to reducing the run speed (which is what determines the horizontal jump distance).

Further tweaks were then made to the level based on playtesting.

## Credits

I, Atul Varma, wrote the code and designed the environment art and font.

The character art is by LuizMelo:

- [Huntress](https://luizmelo.itch.io/huntress) by LuizMelo (CC0 license)
- [Monsters Creatures Fantasy](https://luizmelo.itch.io/monsters-creatures-fantasy) by LuizMelo (CC0 license)

Code from late December 2022 onward was created with the assistance of GitHub Copilot.

## License

Everything in this repository that isn't provided by a third party is licensed under [CC0 1.0 Universal](./LICENSE.md) (public domain).
