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
var elements_in_view = [];

let scrollableArea;
let statusEl;
let waitView;
let search;

// buttons
let selectButton;
let uninstallButton;
let disableButton;

var packages = [];

var waitViewVisible = true;
var n_selected = 0;

var disabledCount = 0
var enabledCount = 0

// same as the keys of `elems`, sacrificing space complexity
// so that taking subset difference is better than O(n)
const package_ids = new Set();
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
  enabledCount = 0
  disabledCount = 0
  uninstallButton.innerHTML = trashSvg
  uninstallButton.classList.remove('bg-zinc-800')
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
      <span class="select-text text-zinc-400 break-all">${packageId}</span>
    </div>

    <p class="font-light truncate select-text">${description}</p>
  </div>`;

  return generateElements(templ)[0]
}

const trashSvg = `
<svg class="w-8 h-8 stroke-zinc-300" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path
              d="m18 6-.8 12.013c-.071 1.052-.106 1.578-.333 1.977a2 2 0 0 1-.866.81c-.413.2-.94.2-1.995.2H9.994c-1.055 0-1.582 0-1.995-.2a2 2 0 0 1-.866-.81c-.227-.399-.262-.925-.332-1.977L6 6M4 6h16m-4 0-.27-.812c-.263-.787-.394-1.18-.637-1.471a2 2 0 0 0-.803-.578C13.938 3 13.524 3 12.694 3h-1.388c-.829 0-1.244 0-1.596.139a2 2 0 0 0-.803.578c-.243.29-.374.684-.636 1.471L8 6"
              stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
`;

const revertSvg = `
    <svg class="w-8 h-8 stroke-zinc-300" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path d="M4 10h13a4 4 0 0 1 4 4v0a4 4 0 0 1-4 4h-5" stroke-width="1.3" stroke-linecap="round"
        stroke-linejoin="round" />
      <path d="m7 6-4 4 4 4" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
`;
const confusedSvg = `
<svg class="w-8 h-8 stroke-zinc-300" viewBox="100 100 200 200" fill="none" xmlns="http://www.w3.org/2000/svg">
<path d="M154.604 194.87C152.978 186.642 153.347 178.343 153.347 170.013" stroke="currentColor" stroke-opacity="0.9" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>
<path d="M242.371 185.992C244.005 178.565 242.371 170.603 242.371 162.91" stroke="currentColor" stroke-opacity="0.9" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>
<path d="M143.005 259.779C154.701 272.112 172.594 288.123 187.201 279.067C201.808 270.012 206.715 251.336 224.438 251.336C242.161 251.336 250.719 260.993 258.639 274.76" stroke="currentColor" stroke-opacity="0.9" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>
<path d="M154.605 118C142.523 122.278 132.988 130.833 126 143.664" stroke="currentColor" stroke-opacity="0.9" stroke-width="12" stroke-linecap="round" stroke-linejoin="round"/>
<path d="M239.933 123.799C253.241 132.353 264.298 136.631 273.106 136.631" stroke="currentColor" stroke-opacity="0.9" stroke-width="12" stroke-linecap="round" stroke-linejoin="round"/>
</svg>
`;

function toggle_row_focus(row) {
    let node = row.node
    let paragraph = node.children[1]
    if (!selection_mode()) {
      clear_selection()
      row.selected ^= true;
      n_selected += row.selected ? 1: -1
      uninstallButton.innerHTML = row.disabled ? revertSvg : trashSvg;
      if (row.disabled) {
        disabledCount = 1
        enabledCount = 0
      } else {
        disabledCount = 0
        enabledCount = 1
      }
      node.classList.add('button-select')
      paragraph.classList.toggle('truncate')
    } else {
      row.selected ^= true;
      n_selected += row.selected ? 1: -1

      if (row.disabled) {
        disabledCount += row.selected ? 1: -1
      } else {
        enabledCount += row.selected ? 1: -1
      }

      if (disabledCount > 0 && enabledCount > 0) {
        uninstallButton.classList.add('bg-zinc-800')
        uninstallButton.innerHTML = confusedSvg;
      } else if (disabledCount > 0) {
        uninstallButton.classList.remove('bg-zinc-800')
        uninstallButton.innerHTML = revertSvg;
      } else {
        uninstallButton.classList.remove('bg-zinc-800')
        uninstallButton.innerHTML = trashSvg;
      }
      node.classList.toggle('button-select');
    }
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
  packages = event.payload;

  var new_pkg_set = new Set();
  for (let pkg of packages) {
    new_pkg_set.add(pkg.id)
  }

  let set_difference = package_ids.difference(new_pkg_set);
  for (let pkg of set_difference) {
    if (elems.has(pkg)) {
      elems.get(pkg).disabled = true;
      elems.get(pkg)?.node.classList.add('striped');
    }
  }

  var new_package = false;
  for (let pkg of packages) {

    // When the node does not exist
    if (!elems.has(pkg.id)) {
      let node = gen_row(pkg.name, pkg.id, "Zombie ipsum actually everyday carry plaid keffiyeh blue bottle wolf quinoa squid four loko glossier kinfolk woke. Plaid cliche cloud bread wolf, etsy humblebrag ennui organic fixie. Tousled sriracha vice VHS. Chillwave vape raw denim aesthetic flannel paleo, austin mixtape lo-fi next level copper mug +1 cred before they sold out. Prism pabst raclette gastropub.")
      let row = {
        name: pkg.name,
        node: node,
        selected: false,
        disabled: false,
      };

      elems.set(pkg.id, row);
      mouse_handler(row);
      new_package = true;
      
      package_ids.add(pkg.id);
    } else {
      // it already exists
      let row = elems.get(pkg.id);
      row.disabled = false;
      // remove the striped background if the app was previously disabled or uninstalled
      row.node.classList.remove('striped');
    }

    let row = elems.get(pkg.id)
    let node = row.node;
    // the name is not set in the frontend
    if (row.name === null && pkg.name !== null) {
      row.name ??= pkg.name;
      node.children[0].children[0].replaceWith(generateElements(`<span class="select-text">${pkg.name}</span>`)[0]);
    }
  }


  if (!scrollableArea.hasChildNodes() || new_package) {
    scrollableArea.replaceChildren(...searchFilter(search.value));
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

function searchFilter(query) {
    var local_elements_in_view = []
    let searchQueryLowerCase = query.toLowerCase()

    for (let [id, row] of elems.entries()) {
      if (searchQueryLowerCase.length === 0 || id.toLowerCase().includes(searchQueryLowerCase) || row.name?.toLowerCase().includes(searchQueryLowerCase)) {
        local_elements_in_view.push(row.node);
      }
    }
  return local_elements_in_view
}


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

    if (!(disabledCount > 0 && enabledCount > 0)) {
      uninstallPackages(uninstallPkgList);
    }
  })

  search.addEventListener('input', (event) => {
    scrollableArea.replaceChildren(...searchFilter(search.value));
  })

  scan();
});
