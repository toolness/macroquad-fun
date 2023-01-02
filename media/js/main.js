const config = await (await fetch("media/config.json")).json();
const width = config.screen_width * config.sprite_scale;
const height = config.screen_height * config.sprite_scale;
const canvas = document.getElementById("glcanvas");

canvas.style = `width: ${width}px; height: ${height}px`;

miniquad_add_plugin({
    register_plugin(importObject) {
        importObject.env.record_input = (ptr, len) => {
            const u8Array = new Uint8Array(wasm_memory.buffer, ptr, len);
            console.log("TODO: RECORD INPUT", u8Array);
        };

        importObject.env.init_version = (ptr) => {
            const version = UTF8ToString(ptr);
            console.log("TODO: STORE VERSION", version);
        };
    }
})

load("target/wasm32-unknown-unknown/release/macroquad-fun.wasm");
