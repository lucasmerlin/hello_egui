(function () {
    // Check if we are already loaded
    if (window["__egui_webview_handle_command"]) {
        return;
    }

    const sendEvent = (data) => {
        window.ipc.postMessage(JSON.stringify(data));
    }

    const captureScreenshot = () => {
        htmlToImage.toPng(document.body, {
            width: window.innerWidth,
            height: window.innerHeight,
            style: {
                transform: `translate(${-window.scrollX}px, ${-window.scrollY}px)`,
                background: "white",
            },
        })
            .then(function (dataUrl) {
                console.log("Captured screenshot");
                sendEvent({
                    type: "Screenshot",
                    base64: dataUrl.replace("data:image/png;base64,", "")
                });
            })
            .catch(function (error) {
                console.error('oops, something went wrong!', error);
            });
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
        if (command.type === "Screenshot") {
            captureScreenshot();
        }
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
