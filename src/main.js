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
  await emit("uninstall", pkgs);
}

async function revertPackages(pkgs) {
  await emit("revert", pkgs);
}

async function disablePackages(pkgs) {
  await emit("disable", pkgs);
}

let scrollableArea;
let statusEl;
let waitView;
let search;

// buttons
let selectButton;
let uninstallButton;
let revertButton;
let disableButton;

var waitViewVisible = true;

class Selection {
  constructor() {
    this.sel = new Set();
    this.ctrlIsHeld = false
    this.uiRubberband = false
  }

  total() {
    return this.sel.size
  }

  enabled() {
    return Array.from(this.sel.values()).filter(row => !row.disabled)
  }

  disabled() {
    return Array.from(this.sel.values()).filter(row => row.disabled)
  }

  toggle(row) {
    this.sel.delete(row) || this.sel.add(row)
  }

  usable() {
    let howManyDisabled = this.disabled().length;
    return howManyDisabled == 0 || howManyDisabled == this.sel.size
  }

  clear() {
    for (let row of this.sel) {
        row.node?.classList.remove('button-select')
    }
    selection.sel.clear()
    selection.updateButtons()
    selection.uiRubberband = false
  }

  updateButtons() {
    if (!this.usable()) {
      uninstallButton.classList.add('hidden')
      revertButton.classList.add('hidden')
      disableButton.classList.add('hidden')
      return
    }

    if (this.disabled().length) {
      revertButton.classList.remove('hidden')
      uninstallButton.classList.add('hidden')
      disableButton.classList.add('hidden');
      return
    }

    revertButton.classList.add('hidden')
    uninstallButton.classList.remove('hidden')
    disableButton.classList.remove('hidden')
  }

  isRubberband() {
    return this.ctrlIsHeld || this.uiRubberband
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
    selection.ctrlIsHeld = true
    return;
  }
  if (event.key === "s" && selection.ctrlIsHeld) {
    console.log('Save')
    return;
  }
  if (event.key === "Escape" && search === document.activeElement) {
    search.blur();
    return;
  }
  // TODO: decrese the scope of this conditional
  if (event.key === "Escape") {
    selection.clear()
    statusModeUpdate()
    return;
  }
  if ('s/'.includes(event.key) && search !== document.activeElement) {
    event.preventDefault()
    search.focus()
  }
})

window.addEventListener("keyup", (event) => {
  if (event.key === "Control") {
    selection.ctrlIsHeld = false
  }
})

/**
 * Create a collapsable accordion row for a given package.
 * @param {string} name - Display name of the package
 * @param {string} packageId - Reverse domain identifier for a package
 * @param {string} description - What the knowledgebase has to say about the package
 */
function newRow(name, packageId, description) {
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

function toggleRowFocus(row) {
    let node = row.node
    let paragraph = node.children[1]
    if (!selection.isRubberband()) {
      selection.clear()
      paragraph.classList.toggle('truncate')
    }

    selection.toggle(row);

    node.classList.toggle('button-select');
    selection.updateButtons()
    statusModeUpdate()
}

function mouseHandler(row) {
  let node = row.node;
  // Don't collapse a row when the user is selecting
  // something from the description
  var mouseClicked = false;
  var mouseMovedInMe = false;

  node.addEventListener('mousedown', (event) => {
    mouseClicked = true;
  })
  node.addEventListener('mousemove', (event) => {
    mouseMovedInMe ||= mouseClicked
  })
  node.addEventListener('mouseup', (event) => {
    if (mouseClicked && !mouseMovedInMe) {
      toggleRowFocus(row)
    }
    mouseClicked = false;
    mouseMovedInMe = false;
  })
}

function statusModeUpdate() {
  if (selection.isRubberband()) {
    if (selection.total()) {
      statusEl.innerText = `${selection.total()} Selected`
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
  setInterval(() => {listPackages()}, 5000)
});

listen('packages-updated', (event) => {
  let packages = event.payload;

  const new_pkg_set = new Set(packages.map(pkg => pkg.id));

  for (let pkg of package_ids.difference(new_pkg_set)) {
    if (!elems.has(pkg)) {
      continue
    }

    let nowUninstalled = elems.get(pkg) // must be defined
    if (nowUninstalled.disabled) {
      continue
    }

    nowUninstalled.disabled = true;
    nowUninstalled.node.classList.add('striped');
  }

  selection.updateButtons()

  var new_package = false;
  for (let pkg of packages) {

    // When the node does not exist
    if (!elems.has(pkg.id)) {
      let node = newRow(pkg.name, pkg.id, "Zombie ipsum actually everyday carry plaid keffiyeh blue bottle wolf quinoa squid four loko glossier kinfolk woke. Plaid cliche cloud bread wolf, etsy humblebrag ennui organic fixie. Tousled sriracha vice VHS. Chillwave vape raw denim aesthetic flannel paleo, austin mixtape lo-fi next level copper mug +1 cred before they sold out. Prism pabst raclette gastropub.")
      let row = {
        id: pkg.id,
        name: pkg.name,
        node: node,
        disabled: false,
      };

      elems.set(pkg.id, row);
      mouseHandler(row);
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
      row.name = pkg.name;
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
    let inView = []
    let searchQueryLowerCase = query.toLowerCase()

    for (let [id, row] of elems.entries()) {
      if (searchQueryLowerCase.length === 0 || id.toLowerCase().includes(searchQueryLowerCase) || row.name?.toLowerCase().includes(searchQueryLowerCase)) {
        inView.push(row.node);
      }
    }
  return inView
}

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
    selection.uiRubberband ^= true;
    if (!selection.uiRubberband) {
      selection.clear()
    }
    statusModeUpdate();
  })

  uninstallButton.addEventListener('click', (event) => {
    if (selection.usable()) {
      uninstallPackages(selection.enabled().map(row => row.id));
    }
  })

  disableButton.addEventListener('click', (event) => {
    if (selection.usable()) {
      disablePackages(selection.enabled().map(row => row.id));
    }
  })

  revertButton.addEventListener('click', (event) => {
    if (selection.usable()) {
      revertPackages(selection.disabled().map(row => row.id));
    }
  })

  search.addEventListener('input', (event) => {
    scrollableArea.replaceChildren(...searchFilter(search.value));
  })

  scan();
});
