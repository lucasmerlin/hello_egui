(function () {
    // Check if we are already loaded
    if (window["__egui_webview_handle_command"]) {
        return;
    }

    const sendEvent = (event) => {
        window.ipc.postMessage(JSON.stringify({
            event,
            __egui_webview: true,
        }));
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
                let event = new MouseEvent("click", {
                    bubbles: true,
                    cancelable: true,
                    clientX: x,
                    clientY: y,
                });
                element.dispatchEvent(event);
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
