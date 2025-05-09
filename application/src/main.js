const { invoke } = window.__TAURI__.core;

let fileInputEl;
let pathInputEl;
let resultEl;

async function send_file() {
  console.log(await fileInputEl.files[0].text());
  result.textContent = await invoke("work_with", { file: await fileInputEl.files[0].text() });
}

window.addEventListener("DOMContentLoaded", () => {
  fileInputEl = document.querySelector("#file-input");
  resultEl = document.querySelector("#result");
  document.querySelector("#file-form").addEventListener("submit", (e) => {
    e.preventDefault();
    send_file();
  });
});
