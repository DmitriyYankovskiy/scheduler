const { invoke } = window.__TAURI__.core;

document.addEventListener("DOMContentLoaded", function () {
    console.log("st");
    const fileSelector = document.querySelector("#fileSelector");
    const fileInput = document.querySelector("#fileInput");
    const costField = document.querySelector("#costField");

    const optimizeButton = document.querySelector("#optimizeButton");
    const downloadButton = document.querySelector("#downloadButton");
    const resultsBlock = document.querySelector("#resultsBlock");

    async function optimizeFile() {
        console.log(await fileInput.files[0].text());

        let cost = await invoke("optimize_schedule", {
            aging: Number(document.querySelector(".aging-input").value),
            shuffling: document.querySelector("#checkboxShuffling").checked,
            greedily: document.querySelector("#checkboxGreedily").checked,
        });

        costField.innerHTML = cost;
        if (cost == 0) {
            downloadButton.classList.add("succesful");
        } else {
            downloadButton.classList.remove("succesful");
        }
    }

    function enableResultsBlock() {
        optimizeButton.style = "display: none";
        resultsBlock.style = "";
    }
    function disableResultsBlock() {
        optimizeButton.style = "";
        resultsBlock.style = "display: none";
        downloadButton.classList.remove("succesful");
    }

    fileSelector.addEventListener("click", async function () {
        fileInput.click();
        disableResultsBlock();
    });
    fileInput.addEventListener("change", async function () {
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
        await invoke("select_file", { file: await fileInput.files[0].text() });
    });

    optimizeButton.addEventListener("click", async (e) => {
        e.preventDefault();
        enableResultsBlock();
        optimizeFile();
    });
    document.querySelector("#updateButton").addEventListener("click", (e) => {
        e.preventDefault();
        optimizeFile();
    });
    downloadButton.addEventListener("click", async (e) => {
        e.preventDefault();
        await invoke("download_file");
    });
});
