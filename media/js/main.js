const config = await (await fetch("media/config.json")).json();
const width = config.screen_width * config.sprite_scale;
const height = config.screen_height * config.sprite_scale;
const canvas = document.getElementById("glcanvas");

canvas.style = `width: ${width}px; height: ${height}px`;

let version = "0.0.0";

let recordingBytes = {
    id: null,
    sent: 0,
    toSend: [],
    nextScheduledSend: null,
}

function scheduleSendRecordingBytes(ms) {
    if (recordingBytes.nextScheduledSend === null) {
        recordingBytes.nextScheduledSend = setTimeout(sendRecordingBytes, ms);
    }
}

async function sendRecordingBytes() {
    console.log("Sending recording bytes", recordingBytes);
    if (recordingBytes.toSend.length === 0) {
        recordingBytes.nextScheduledSend = null;
        return;
    }
    let success = false;
    try {
        const data = new FormData();
        const numBytesToSend = recordingBytes.toSend.length;
        data.append("v", version);
        data.append("b", new Blob([new Uint8Array(recordingBytes.toSend)]));
        data.append("p", recordingBytes.sent);
        // TODO: Add tracking tag.
        if (recordingBytes.id !== null) {
            data.append("id", recordingBytes.id);
        }
        const response = await fetch("http://localhost:4001/record", {
            method: "POST",
            body: data,
        });
        if (response.ok) {
            const id = await response.text();
            recordingBytes.id = id;
            recordingBytes.sent += numBytesToSend;
            recordingBytes.toSend = recordingBytes.toSend.slice(numBytesToSend);
            console.log(`Sent ${numBytesToSend} bytes for session ${id}, total sent: ${recordingBytes.sent}`);
            success = true;
        } else {
            console.error("Error sending recording bytes", response);
        }
    } catch (e) {
        console.error("Error sending recording bytes", e);
    }

    recordingBytes.nextScheduledSend = null;

    if (success) {
        if (recordingBytes.toSend.length > 0) {
            scheduleSendRecordingBytes(100);
        }
    } else {
        // TODO: exponential backoff
        scheduleSendRecordingBytes(10_000);
    }
}

miniquad_add_plugin({
    register_plugin(importObject) {
        importObject.env.record_input = (ptr, len) => {
            const u8Array = new Uint8Array(wasm_memory.buffer, ptr, len);
            recordingBytes.toSend.push(...Array.from(u8Array));
            scheduleSendRecordingBytes(100);
        };

        importObject.env.init_version = (ptr) => {
            version = UTF8ToString(ptr);
        };
    }
})

window.addEventListener("blur", () => {
    wasm_exports.set_blurred(1);
});

window.addEventListener("focus", () => {
    wasm_exports.set_blurred(0);
});

load("target/wasm32-unknown-unknown/release/macroquad-fun.wasm");
