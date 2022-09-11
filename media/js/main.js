const config = await (await fetch("media/config.json")).json();
const width = config.screen_width * config.sprite_scale;
const height = config.screen_height * config.sprite_scale;
const canvas = document.getElementById("glcanvas");

canvas.style = `width: ${width}px; height: ${height}px`;

load("target/wasm32-unknown-unknown/release/macroquad-fun.wasm");
