(function () {
    // Check if we are already loaded
    if (window["__egui_webview_handle_command"]) {
        return;
    }

    const sendEvent = (data) => {
        window.ipc.postMessage(JSON.stringify(data));
    }

    document.addEventListener("focus", (e) => {
        sendEvent({
            type: "Focus",
            target: e.target.id,
        });
    });

    document.addEventListener("blur", (e) => {
        sendEvent({
            type: "Blur",
            target: e.target.id,
        });
    });

    window["__egui_webview_handle_command"] = (command) => {
        if (command.type === "Click") {
            // convert from physical to logical pixels
            let x = command.x * window.devicePixelRatio;
            let y = command.y * window.devicePixelRatio;
            let element = document.elementFromPoint(x, y);
            if (element) {
                element.click();
            }
        }
        if (command.type === "Back") {
            window.history.back();
        }
        if (command.type === "Forward") {
            window.history.forward();
        }
    }
})();
