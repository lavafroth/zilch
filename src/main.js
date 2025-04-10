const { invoke } = window.__TAURI__.core;
const { listen, emit } = window.__TAURI__.event;
const { save } = window.__TAURI__.dialog;
import { Selection } from './selection.js';

// Thunks that call async Rust routines
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

async function scan() {
  await invoke("scan");
}
async function listPackages() {
  await invoke("list_packages").catch(handleUsbDisconnect);
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

const buttons = {
  select: null,
  uninstall: null,
  revert: null,
  disable: null,
};

let waitViewVisible = true;

const selection = new Selection()

// All package IDs seen in the lifetime of a USB session.
// Elements (rows) visible on screen with some additional metadata.
let elems = new Map();

function handleUsbDisconnect(error) {
  if (error.includes("USB Error: No such device") && !waitViewVisible) {

    console.debug('the usb device has been disconnected');
    waitViewVisible = true;
    waitView.classList.remove("pageFadeOut");
    waitView.classList.add("pageFadeIn");

    elems.clear(); // clear out the packages seen
    selection.clear(); // clear any previous selection
    scan(); // put the scan function on the event loop
  }
}

function generateElements(html) {
  const template = document.createElement('template');
  template.innerHTML = html.trim();
  return template.content.children;
}

async function promptSavePath() {
  const path = await save({
    filters: [
      {
        name: 'zilch configs',
        extensions: ['json', 'zilch'],
      },
    ],
  });
  console.log(path);
}

window.addEventListener("keydown", (event) => {
  if (event.key === "Control") {
    selection.ctrlIsHeld = true
    return;
  }
  if (!waitViewVisible && event.key === "s" && selection.ctrlIsHeld) {
    promptSavePath();
    return;
  }
  if (event.key === "Escape" && search === document.activeElement) {
    search.blur();
    return;
  }
  if (!waitViewVisible && event.key === "Escape") {
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
      selection.clear(buttons)
      paragraph.classList.toggle('truncate')
    }

    selection.toggle(row);

    node.classList.toggle('button-select');
    selection.updateButtons(buttons)
    statusModeUpdate()
}

function mouseHandler(row) {
  let node = row.node;
  // Don't collapse a row when the user is selecting
  // something from the description
  let mouseClicked = false;
  let mouseMovedInMe = false;

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
  listPackages()
  setInterval(() => {
    if (waitViewVisible) {
      return;
    }
    listPackages()
  }, 5000)
});

listen('packages-updated', (event) => {
  let packages = event.payload;
  let newPkgIds = new Set(packages.map(pkg => pkg.id));

  for (let [id, elem] of elems.entries()) {
    if (newPkgIds.has(id)) {
      continue
    }

    let nowUninstalled = elem
    if (nowUninstalled.disabled) {
      continue
    }

    nowUninstalled.disabled = true;
    nowUninstalled.node.classList.add('striped');
  }

  selection.updateButtons(buttons)

  let newPackage = false;
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
      newPackage = true;
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


  if (!scrollableArea.hasChildNodes() || newPackage) {
    scrollableArea.replaceChildren(...searchFilter(search.value));
  }

  
  if (waitViewVisible) {
    waitViewVisible = false;
    waitView.classList.remove("pageFadeIn");
    waitView.classList.add("pageFadeOut");
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
  buttons.select = document.querySelector("#select");
  buttons.uninstall = document.querySelector("#uninstall");
  buttons.disable = document.querySelector("#disable");
  buttons.revert = document.querySelector("#revert");

  buttons.select.addEventListener('click', (event) => {
    selection.uiRubberband ^= true;
    if (!selection.uiRubberband) {
      selection.clear(buttons)
    }
    statusModeUpdate();
  })

  buttons.uninstall.addEventListener('click', (event) => {
    if (selection.usable()) {
      uninstallPackages(selection.enabled().map(row => row.id));
    }
  })

  buttons.disable.addEventListener('click', (event) => {
    if (selection.usable()) {
      disablePackages(selection.enabled().map(row => row.id));
    }
  })

  buttons.revert.addEventListener('click', (event) => {
    if (selection.usable()) {
      revertPackages(selection.disabled().map(row => row.id));
    }
  })

  search.addEventListener('input', (event) => {
    scrollableArea.replaceChildren(...searchFilter(search.value));
  })

  scan();
});
