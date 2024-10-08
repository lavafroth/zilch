const { invoke } = window.__TAURI__.core;

// async function greet() {
//   // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
//   greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
// }

var ctrl_is_held = false;
var ui_selection_mode = false;

let scrollableArea;
let selectButton;
let search;
let statusEl;

var bitvec = [];

var n_selected = 0;
var elems = [];

function generateElements(html) {
  const template = document.createElement('template');
  template.innerHTML = html.trim();
  return template.content.children;
}

function selection_mode() {
  return ctrl_is_held || ui_selection_mode
}

window.addEventListener("keydown", (event) => {
  // console.log(event)
  if (event.key === "Control") {
    ctrl_is_held = true
    console.log(selection_mode())
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
    n_selected = 0
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
    elems.map((el) => el.classList.remove('button-select'))
    ui_selection_mode = false
    n_selected = 0

    for (let step = 0; step < bitvec.length; step++) {
      bitvec[step] = 0;
    }
}

/**
 * Create a collapsable accordion row for a given package.
 * @param {string} name - Display name of the package
 * @param {string} packageId - Reverse domain identifier for a package
 * @param {string} description - What the knowledgebase has to say about the package
 */
function gen_row(name, packageId, description) {
  let templ = `<div id="accordion" class="button">

    <div>
      <span class="select-text">${name}</span>
      <span class="select-text text-zinc-400">${packageId}</span>
    </div>

    <p class="font-light truncate select-text">${description}</p>
  </div>`;

  return generateElements(templ)[0]
}

function toggle_row_focus(node, i) {
    let paragraph = node.children[1]
    if (!selection_mode()) {
      status_selection_toggle(false)
      clear_selection()
      bitvec[i] ^= true;
      n_selected += bitvec[i] ? 1: -1
      node.classList.add('button-select')
      paragraph.classList.toggle('truncate')
    } else {
      bitvec[i] ^= true;
      n_selected += bitvec[i] ? 1: -1
      node.classList.toggle('button-select');
      status_selection_toggle(true);
    }
}

function mouse_handler(node, i) {
  // Don't collapse a row when the user is selecting
  // something from the description
  var mouse_clicked = false;
  var mouse_moved_in_me = false;

  node.addEventListener('mousedown', (event) => {
    mouse_clicked = true;
  })
  node.addEventListener('mousemove', (event) => {
    if (mouse_clicked) {
      mouse_moved_in_me = true;
    }
  })
  node.addEventListener('mouseup', (event) => {
    if (mouse_clicked && !mouse_moved_in_me) {
      toggle_row_focus(node, i)
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

/**
 * Add a row to the document body.
 * @param {Node} node - The row to be added to the document body.
 */
function add_row(node, i) {
  mouse_handler(node, i)
  elems.push(node)
  scrollableArea.appendChild(node);
}

window.addEventListener("DOMContentLoaded", () => {
  scrollableArea = document.querySelector("#scrollableArea");
  search = document.querySelector("#search");
  statusEl = document.querySelector("#status");
  selectButton = document.querySelector("#select");


  for (let step = 0; step < 5; step++) {
    let row = gen_row("Foo bar game", "org.foo.bar", "Zombie ipsum actually everyday carry plaid keffiyeh blue bottle wolf quinoa squid four loko glossier kinfolk woke. Plaid cliche cloud bread wolf, etsy humblebrag ennui organic fixie. Tousled sriracha vice VHS. Chillwave vape raw denim aesthetic flannel paleo, austin mixtape lo-fi next level copper mug +1 cred before they sold out. Prism pabst raclette gastropub.");
    bitvec.push(0);
    add_row(row, step)
  }

  selectButton.addEventListener('click', (event) => {
    ui_selection_mode ^= true;
    if (!ui_selection_mode) {
      clear_selection()
    }
    status_selection_toggle(ui_selection_mode);
  })
});
