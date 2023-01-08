## Introduction

This is a prototype for a non-violent 2D platformer using the [macroquad][] Rust library, with character art by [LuizMelo][].

The goals for this project were:

- I wanted more experience writing Rust for game development.

- I wanted to tinker with non-violent game and level design. I was particularly intrigued by [LDtk](https://ldtk.io/) and wanted to play with it.

- I liked LuizMelo's art and wanted to make something with it.

[macroquad]: https://macroquad.rs/

## Architectural notes

Originally, this game used a somewhat object oriented architecture. But because Rust doesn't have many object oriented affordances, I quickly ran into problems.

This led me to re-watching Catherine West's RustConf 2018 closing keynote, [Using Rust For Game Development](https://kyren.github.io/2018/09/14/rustconf-talk.html), and reading her associated blog post, which fortunately described a journey much like the one I was taking: developing a Rust game in OO and running into lots of issues.

At this point, at of the end of 2022, the architecture has, with the immense help of West's materials, evolved into an architecture that is essentially an "array of structs" ECS--what West calls a bare minimum ECS system. It lacks the cache locality optimizations afforded by a "struct of arrays" style ECS.

It also uses a HashMap for all the entities instead of a vector with generational indices, which is probably not great for performance, but the actual game isn't demanding enough for this to be an issue right now.

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

When users start the web version, they're prompted to submit their gameplay data for analytics purposes. If they consent, the related data is sent to a server as they play.

The `server` subdirectory contains a simple server that receives this data and writes it to disk.

To build and run the server:

```
cd server
cargo run
```

Accessing the web version at `localhost` will automatically use this server.

Note that accessing the web version at any _other_ hostname (including an IP address, even `127.0.0.1`) will cause the game to submit any data to my personal analytics server hosted at `macroquad-fun.toolness.org`. Ideally, this should be made more configurable.

## Credits

I, Atul Varma, wrote the code and designed the environment art.

The rest of the graphics are by LuizMelo:

- [Huntress](https://luizmelo.itch.io/huntress) by LuizMelo (CC0 license)
- [Monsters Creatures Fantasy](https://luizmelo.itch.io/monsters-creatures-fantasy) by LuizMelo (CC0 license)

Code from late December 2022 onward was created with the assistance of GitHub Copilot.
