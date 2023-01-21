import * as analyticsDialog from "./analytics-dialog.js";

const config = await (await fetch("media/config.json")).json();
const width = config.screen_width * config.sprite_scale;
const height = config.screen_height * config.sprite_scale;
const canvas = document.getElementById("glcanvas");
const canvasWrapper = document.getElementById("glcanvas-wrapper");

canvas.style = `width: ${width}px; height: ${height}px`;

function maybeScaleCanvas() {
    const { innerWidth, innerHeight } = window;
    let scale = 1.0;

    if (innerWidth < width || innerHeight < height) {
        scale = Math.min(innerWidth / width, innerHeight / height);
    }

    if (scale !== 1.0) {
        canvasWrapper.style = `transform-origin: top-left; transform: scale(${scale});`;
    } else {
        canvasWrapper.style = "";
    }
}

window.addEventListener("resize", maybeScaleCanvas);

maybeScaleCanvas();

await analyticsDialog.maybeShowDialog();

const DEBUG = false;

const MIN_USEFUL_RECORDING_BYTES = 25;

const windowSearchParams = new URLSearchParams(window.location.search);

const didUserConsentToAnalytics = analyticsDialog.hasUserGivenConsent();

const trackingTag = getAndCacheTrackingTag();

let version = "0.0.0";

let recordingBytes = {
    id: null,
    sent: 0,
    toSend: [],
    nextScheduledSend: null,
}

function isValidTrackingTag(tag) {
    if (!tag) {
        return false;
    }
    if (tag.length > 10 || !tag.match(/^[a-zA-Z0-9]+$/)) {
        return false;
    }
    return true;
}

function getAndCacheTrackingTag() {
    let tag = windowSearchParams.get("t");
    if (tag && !isValidTrackingTag(tag)) {
        tag = null;
    }
    if (tag) {
        try {
            window.localStorage.setItem("macroquad_fun_tracking_tag", tag);
        } catch (e) {
        }
    } else {
        try {
            tag = window.localStorage.getItem("macroquad_fun_tracking_tag");
        } catch (e) {
        }
    }
    return tag;
}

function scheduleSendRecordingBytes(ms) {
    if (recordingBytes.nextScheduledSend === null) {
        recordingBytes.nextScheduledSend = setTimeout(sendRecordingBytes, ms);
    }
}

function areUsefulRecordingBytesAvailable() {
    let minimumBytesToSend = 0;
    if (recordingBytes.sent === 0) {
        // We don't want to spam our analytics with tons of
        // useless sessions, so let's wait until we have at least
        // enough data to be useful before we send anything.
        minimumBytesToSend = MIN_USEFUL_RECORDING_BYTES;
    }
    return recordingBytes.toSend.length > minimumBytesToSend;
}

function getServerOrigin() {
    if (window.location.hostname === "localhost") {
        return "http://localhost:4001";
    }
    return "https://macroquad-fun.toolness.org";
}

async function sendRecordingBytes() {
    if (!areUsefulRecordingBytesAvailable()) {
        recordingBytes.nextScheduledSend = null;
        return;
    }
    let success = false;
    const data = new FormData();
    const numBytesToSend = recordingBytes.toSend.length;
    data.append("v", version);
    data.append("b", new Blob([new Uint8Array(recordingBytes.toSend)]));
    data.append("p", recordingBytes.sent);
    if (trackingTag && isValidTrackingTag(trackingTag)) {
        data.append("t", trackingTag);
    }
    if (recordingBytes.id !== null) {
        data.append("id", recordingBytes.id);
    }
    try {
        const response = await fetch(`${getServerOrigin()}/record`, {
            method: "POST",
            body: data,
        });
        if (response.ok) {
            const id = await response.text();
            recordingBytes.id = id;
            success = true;
        } else {
            if (DEBUG) {
                console.error("Error response when sending recording bytes", response);
            }
        }
    } catch (e) {
        if (DEBUG) {
            console.error("Error thrown when sending recording bytes", e);
        }
    }

    recordingBytes.nextScheduledSend = null;

    if (success) {
        recordingBytes.sent += numBytesToSend;
        recordingBytes.toSend = recordingBytes.toSend.slice(numBytesToSend);
        if (DEBUG) {
            console.log(`Sent ${numBytesToSend} bytes for session ${id}, total sent: ${recordingBytes.sent}`);
        }
        if (recordingBytes.toSend.length > 0) {
            scheduleSendRecordingBytes(100);
        }
    } else {
        // TODO: exponential backoff
        scheduleSendRecordingBytes(10_000);
    }
}

const isOggSupported = detectOggSupport();

function detectOggSupport() {
    if (windowSearchParams.get("disable_ogg")) {
        console.log("Disabling OGG support because 'disable_ogg' is set in URL.");
        return false;
    }
    const audio  = document.createElement("audio");
    if (typeof audio.canPlayType === "function") {
        return audio.canPlayType("audio/ogg") !== "";
    }
    return false;
}

if (!isOggSupported) {
    console.log("OGG is unsupported on this browser, disabling sound.");
}

miniquad_add_plugin({
    register_plugin(importObject) {
        importObject.env.does_browser_support_ogg = () => {
            return isOggSupported === true ? 1 : 0;
        };

        importObject.env.record_input = (ptr, len) => {
            if (!didUserConsentToAnalytics) {
                return;
            }
            const u8Array = new Uint8Array(wasm_memory.buffer, ptr, len);
            recordingBytes.toSend.push(...Array.from(u8Array));
            scheduleSendRecordingBytes(100);
        };

        importObject.env.init_version = (ptr) => {
            version = UTF8ToString(ptr);
            const throbber = document.getElementById("throbber");
            if (throbber) {
                throbber.parentNode.removeChild(throbber);
            }
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
