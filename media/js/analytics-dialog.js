const YES = "yes";

const NO = "no";

/**
 * Returns YES, NO, or an empty string if the dialog was cancelled via ESC, etc.
 */
async function getAnalyticsDialogResponse() {
    const dialog = document.getElementById("analytics-dialog");

    if (!dialog.showModal) {
        // Browser doesn't support <dialog>, just exit.
        return "";
    }

    return new Promise((resolve) => {
        dialog.showModal();

        dialog.addEventListener("close", () => {
            resolve(dialog.returnValue);
        });
    });
}

export async function maybeShowDialog() {
    if (getConsent()) {
        return;
    }

    const response = await getAnalyticsDialogResponse();

    try {
        window.localStorage.setItem("macroquad_fun_tracking_consent", response);
    } catch (e) {}
}

/**
 * Returns YES, NO, or an empty string if the user hasn't explicitly given or denied consent.
 */
function getConsent() {
    try {
        return window.localStorage.getItem("macroquad_fun_tracking_consent") || "";
    } catch (e) {
        return "";
    }
}

export function hasUserGivenConsent() {
    return getConsent() === YES;
}
