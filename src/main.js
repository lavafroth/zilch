const { invoke } = window.__TAURI__.core;
const { listen, emit } = window.__TAURI__.event;

// Thunks that call async Rust routines
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
async function scan() {
  await invoke("scan");
}
async function listPackages() {
  await invoke("list_packages");
}

async function uninstallPackages(pkgs) {
  await emit("uninstall",  pkgs );
}

var ctrl_is_held = false;
var ui_selection_mode = false;

let scrollableArea;
let statusEl;
let waitView;
let search;

// buttons
let selectButton;
let uninstallButton;
let disableButton;
let revertButton;

var packages = [];

var waitViewVisible = true;
var n_selected = 0;
const elems = new Map();

function generateElements(html) {
  const template = document.createElement('template');
  template.innerHTML = html.trim();
  return template.content.children;
}

function selection_mode() {
  return ctrl_is_held || ui_selection_mode
}

window.addEventListener("keydown", (event) => {
  if (event.key === "Control") {
    ctrl_is_held = true
    return;
  }
  if (event.key === "s" && ctrl_is_held) {
    console.log('Save')
    return;
  }
  if (event.key === "Escape" && search === document.activeElement) {
    search.blur();
    return;
  }
  // TODO: decrese the scope of this conditional
  if (event.key === "Escape") {
    clear_selection()
    status_selection_toggle(false)
    return;
  }
  if ((event.key === "s" || event.key === "/") && search !== document.activeElement) {
    event.preventDefault()
    search.focus()
  }
})

window.addEventListener("keyup", (event) => {
  if (event.key === "Control") {
    ctrl_is_held = false
  }
})

function clear_selection() {
  for (let row of elems.values()) {
      row.selected = false;
      row.node?.classList.remove('button-select')
  }
  n_selected = 0
  ui_selection_mode = false
}

/**
 * Create a collapsable accordion row for a given package.
 * @param {string} name - Display name of the package
 * @param {string} packageId - Reverse domain identifier for a package
 * @param {string} description - What the knowledgebase has to say about the package
 */
function gen_row(name, packageId, description) {
  if (name === null) {
    name = `<div class="w-32 h-4 rounded bg-zinc-400 animate-pulse inline-block"></div>`;
  } else {
    name = `<span class="select-text">${name}</span>`;
  };
  let templ = `<div id="accordion" class="button">
  
    <div>
      ${name}
      <span class="select-text text-zinc-400">${packageId}</span>
    </div>

    <p class="font-light truncate select-text">${description}</p>
  </div>`;

  return generateElements(templ)[0]
}

function toggle_row_focus(row) {
    let node = row.node
    let paragraph = node.children[1]
    if (!selection_mode()) {
      clear_selection()
      node.classList.add('button-select')
      paragraph.classList.toggle('truncate')
    } else {
      node.classList.toggle('button-select');
    }
    row.selected ^= true;
    n_selected += row.selected ? 1: -1
    status_selection_toggle(selection_mode())
}

function mouse_handler(row) {
  let node = row.node;
  // Don't collapse a row when the user is selecting
  // something from the description
  var mouse_clicked = false;
  var mouse_moved_in_me = false;

  node.addEventListener('mousedown', (event) => {
    mouse_clicked = true;
  })
  node.addEventListener('mousemove', (event) => {
    mouse_moved_in_me ||= mouse_clicked
  })
  node.addEventListener('mouseup', (event) => {
    if (mouse_clicked && !mouse_moved_in_me) {
      toggle_row_focus(row)
    }
    mouse_clicked = false;
    mouse_moved_in_me = false;
  })
}

function status_selection_toggle(is_select) {
  if (is_select) {
    if (n_selected != 0) {
      statusEl.innerText = `${n_selected} Selected`
    } else {
      statusEl.innerText = "Selection Mode"
    }
    statusEl.classList.add('status-select')
    statusEl.classList.remove('status-normal')
  } else {
    statusEl.innerText = "Normal Mode"
    statusEl.classList.remove('status-select')
    statusEl.classList.add('status-normal')
  }
}

listen('device-ready', (event) => {
  setInterval( () => {listPackages()}, 5000)
});

listen('packages-updated', (event) => {
  var elements_in_view = [];
  packages = event.payload;
  for (let pkg of packages) {

    // When the node does not exist
    if (!elems.has(pkg.id)) {
      let node = gen_row(pkg.name, pkg.id, "Zombie ipsum actually everyday carry plaid keffiyeh blue bottle wolf quinoa squid four loko glossier kinfolk woke. Plaid cliche cloud bread wolf, etsy humblebrag ennui organic fixie. Tousled sriracha vice VHS. Chillwave vape raw denim aesthetic flannel paleo, austin mixtape lo-fi next level copper mug +1 cred before they sold out. Prism pabst raclette gastropub.")
      let row = {
        name: pkg.name,
        node: node,
        selected: false,
      };

      elems.set(pkg.id, row);
      mouse_handler(row);
    }

    // it already exists
    let row = elems.get(pkg.id)
    let node = row.node;
    // the name is not set in the frontend
    if (row.name === null && pkg.name !== null) {
      row.name ??= pkg.name;
      node.children[0].children[0].replaceWith(generateElements(`<span class="select-text">${pkg.name}</span>`)[0]);
    }
    elements_in_view.push(node);

  }


  if (!scrollableArea.hasChildNodes()) {
    scrollableArea.replaceChildren(...elements_in_view);
  }

  
  if (waitViewVisible) {
    waitViewVisible = false;
    waitView.classList.remove("pageFadeIn");
    waitView.classList.add("pageFadeOut");
  // } else {
  //   waitViewVisible = true;
  //   waitView.classList.remove("pageFadeOut");
  //   waitView.classList.add("pageFadeIn");
  }

});


listen('indexing-packages', (event) => {
  let waitHeader =document.querySelector("#waitHeader");
  let waitDescription =document.querySelector("#waitDescription");
  waitHeader.innerText = "Indexing packages";
  waitDescription.innerText = "Indexing packages";
});

window.addEventListener("DOMContentLoaded", () => {
  scrollableArea = document.querySelector("#scrollableArea");
  search = document.querySelector("#search");
  statusEl = document.querySelector("#status");
  waitView = document.querySelector("#waitView");
  selectButton = document.querySelector("#select");
  uninstallButton = document.querySelector("#uninstall");
  disableButton = document.querySelector("#disable");
  revertButton = document.querySelector("#revert");

  selectButton.addEventListener('click', (event) => {
    ui_selection_mode ^= true;
    if (!ui_selection_mode) {
      clear_selection()
    }
    status_selection_toggle(ui_selection_mode);
  })

  uninstallButton.addEventListener('click', (event) => {
    var uninstallPkgList = [];
    for (let pkg of packages) {
      let row = elems.get(pkg.id);
      if (row.selected) {
        uninstallPkgList.push(pkg.id);
      }
    }
    uninstallPackages(uninstallPkgList);
  })

  search.addEventListener('input', (event) => {
    var local_elements_in_view = []
    let searchQueryLowerCase = search.value.toLowerCase()

    for (let pkg of packages) {
      let row = elems.get(pkg.id);
      if (searchQueryLowerCase.length === 0 || pkg.id.toLowerCase().includes(searchQueryLowerCase) || row.name?.toLowerCase().includes(searchQueryLowerCase)) {
        local_elements_in_view.push(row.node);
      }
    }

    scrollableArea.replaceChildren(...local_elements_in_view);
  })

  scan();
});
