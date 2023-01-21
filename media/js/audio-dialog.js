export async function maybeShowDialog() {
    // https://stackoverflow.com/a/31732310
    const isSafari = navigator.vendor && navigator.vendor.indexOf('Apple') > -1;

    // We only need to display this dialog on Safari; other browsers will play
    // audio fine without it.
    if (!isSafari) {
        return;
    }

    const dialog = document.getElementById("audio-dialog");

    if (!dialog.showModal) {
        return;
    }

    dialog.querySelector('button[value="yes"]').addEventListener("click", () => {
        audio_init();
    });

    return new Promise((resolve) => {
        dialog.showModal();

        dialog.addEventListener("close", () => {
            resolve();
        });
    });
}
