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

class Selection {
  constructor() {
    this.disabled = 0
    this.enabled = 0
  }

  total() {
    return this.disabled + this.enabled
  }

  usable() {
    return !(this.disabled > 0 && this.enabled > 0)
  }

  updateButtons() {
    // console.log(this)
    if (!this.usable()) {
      uninstallButton.classList.add('invisible')
      disableButton.classList.add('invisible')
      return
    }

    if (this.disabled > 0) {
      uninstallButton.innerHTML = revertSvg;
      uninstallButton.classList.remove('invisible')
      disableButton.classList.add('invisible');
      return
    }

    if (this.enabled > 0) {
      uninstallButton.innerHTML = trashSvg;
      uninstallButton.classList.remove('invisible')
      disableButton.classList.remove('invisible')
      return
    }
  }

  isRubberband() {
    return ctrl_is_held || ui_selection_mode
  }
}

var selection = new Selection()

// same as the keys of `elems`, sacrificing space complexity
// so that taking subset difference is better than O(n)
const package_ids = new Set();
var elems = new Map();

function generateElements(html) {
  const template = document.createElement('template');
  template.innerHTML = html.trim();
  return template.content.children;
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
  selection.enabled = 0;
  selection.disabled = 0;
  selection.updateButtons()
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

const disableSvg = `
<svg class="w-8 h-8 fill-none" viewBox="-2 -2 36 36">
  <path d="M0 0h32v32H0z" />
  <path class="fill-zinc-300"
    d="M16 0c8.837 0 16 7.163 16 16s-7.163 16-16 16S0 24.837 0 16 7.163 0 16 0zm0 2C8.268 2 2 8.268 2 16s6.268 14 14 14 14-6.268 14-14S23.732 2 16 2zm2.828 9.757a1 1 0 0 1 1.415 1.415l-7.071 7.07a1 1 0 0 1-1.415-1.414z"
    fill-rule="nonzero" />
</svg>
`

function toggle_row_focus(row) {
    let node = row.node
    let paragraph = node.children[1]
    if (!selection.isRubberband()) {
      clear_selection()
      row.selected ^= true;
      if (row.disabled) {
        selection.disabled = 1;
        selection.enabled = 0;
      } else {
        selection.disabled = 0;
        selection.enabled = 1;
      }
      node.classList.add('button-select')
      paragraph.classList.toggle('truncate')
    } else {
      row.selected ^= true;
      let delta = row.selected ? 1: -1

      if (row.disabled) {
        selection.disabled += delta;
      } else {
        selection.enabled += delta;
      }

      node.classList.toggle('button-select');
    }
    selection.updateButtons()
    status_selection_toggle(selection.isRubberband())
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
    if (selection.total() != 0) {
      let n_selected = selection.total()
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

  const new_pkg_set = new Set(packages.map(pkg => pkg.id));

  let set_difference = package_ids.difference(new_pkg_set);
  for (let pkg of set_difference) {
    if (!elems.has(pkg)) {
      continue
    }

    let nowUninstalled = elems.get(pkg) // must be defined
    if (nowUninstalled.disabled) {
      continue
    }

    nowUninstalled.disabled = true;
    nowUninstalled.node.classList.add('striped');
    if (nowUninstalled.selected) {
      selection.disabled += 1
      selection.enabled -= 1
    }
  }

  selection.updateButtons()

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

    if (selection.usable()) {
      uninstallPackages(uninstallPkgList);
    }
  })

  search.addEventListener('input', (event) => {
    scrollableArea.replaceChildren(...searchFilter(search.value));
  })

  scan();
});
