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
    clear_selection();
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
    console.log(selection_mode())
  }
})

function clear_selection() {
    elems.map((el) => el.classList.remove('button-select'));
    ui_selection_mode = false;
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

/**
 * Add a row to the document body.
 * @param {Node} node - The row to be added to the document body.
 */
function add_row(node) {
  let el_p = node.children[1]
  node.addEventListener("click", (event) => {
    if (!selection_mode()) {
      clear_selection()
      node.classList.add('button-select')
      el_p.classList.toggle('truncate')
    } else {
      node.classList.toggle('button-select');
    }
  })
  elems.push(node)
  scrollableArea.appendChild(node);
}

window.addEventListener("DOMContentLoaded", () => {
  scrollableArea = document.querySelector("#scrollableArea");
  search = document.querySelector("#search");
  selectButton = document.querySelector("#select");


  for (let step = 0; step < 5; step++) {
    let row = gen_row("Foo bar game", "org.foo.bar", "Zombie ipsum actually everyday carry plaid keffiyeh blue bottle wolf quinoa squid four loko glossier kinfolk woke. Plaid cliche cloud bread wolf, etsy humblebrag ennui organic fixie. Tousled sriracha vice VHS. Chillwave vape raw denim aesthetic flannel paleo, austin mixtape lo-fi next level copper mug +1 cred before they sold out. Prism pabst raclette gastropub.");
    add_row(row)
  }

  selectButton.addEventListener('click', (event) => {
    ui_selection_mode ^= true;
  })
});
