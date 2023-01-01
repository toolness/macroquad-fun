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

## Credits

- [Huntress](https://luizmelo.itch.io/huntress) by LuizMelo (CC0 license)
- [Monsters Creatures Fantasy](https://luizmelo.itch.io/monsters-creatures-fantasy) by LuizMelo (CC0 license)
