const { invoke } = window.__TAURI__.core;

// let fileInputEl;
// let pathInputEl;
// let resultEl;

// async function send_file() {
//     console.log(await fileInputEl.files[0].text());
//     result.textContent = await invoke("work_with", {
//         file: await fileInputEl.files[0].text(),
//     });
// }

// window.addEventListener("DOMContentLoaded", () => {
//     fileInputEl = document.querySelector("#file-input");
//     resultEl = document.querySelector("#result");
//     document.querySelector("#file-form").addEventListener("submit", (e) => {
//         e.preventDefault();
//         send_file();
//     });
// });

document.addEventListener("DOMContentLoaded", function () {
    console.log("st");
    const fileSelector = document.querySelector("#fileSelector");
    const fileInput = document.querySelector("#fileInput");
    fileSelector.addEventListener("click", function () {
        fileInput.click();
    });
    fileInput.addEventListener("change", function () {
        if (this.files && this.files[0]) {
            const fileName = this.files[0].name;
            const previousFileInfo = fileSelector.querySelector(".file-info");
            if (previousFileInfo) {
                fileSelector
                    .querySelector("div.text-center")
                    .removeChild(previousFileInfo);
            }
            const fileInfo = document.createElement("p");
            fileInfo.textContent = fileName;
            fileInfo.className = "file-info";
            fileSelector.querySelector("div.text-center").appendChild(fileInfo);
        }
    });

    async function send_file() {
        console.log(await fileInput.files[0].text());
        /*result.textContent = */ await invoke("work_with", {
            file: await fileInput.files[0].text(),
        });
    }

    document.querySelector("#optimizeButton").addEventListener("click", (e) => {
        e.preventDefault();
        send_file();
    });
});
