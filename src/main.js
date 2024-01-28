const { invoke } = window.__TAURI__.tauri;
const { readDir, BaseDirectory } = window.__TAURI__.fs;
const { resourceDir } = window.__TAURI__.path;

/*

async function launch(name) {
  let url = await invoke("launch", {name: name});
  window.location.href = url;
}
async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  //greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

let resourceDirPath = await resourceDir();
let entries = await readDir(resourceDirPath + "/resources/");


function createHtmlForEntry(entry) {
  let div = document.createElement("div");
  div.classList.add("row");
  let text = document.createTextNode(entry.name);
  let button = document.createElement("button");
  button.data = "hi";
  button.innerHTML = "Launch";
  button.onclick = function() {
    launch(entry.name);
  }
  div.appendChild(text);
  div.appendChild(button);
  return div;
}
  
console.log("entries are:", entries);
let container = document.getElementById("demo-apps")
for (const entry of (entries)) {
  let div = createHtmlForEntry(entry);
  container.appendChild(div);
}

*/
