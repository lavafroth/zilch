export class Selection {
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

  clear(buttons) {
    for (let row of this.sel) {
        row.node?.classList.remove('button-select')
    }
    this.sel.clear()
    if (buttons) {
      this.updateButtons(buttons)
    }
    this.uiRubberband = false
  }

  updateButtons(buttons) {
    if (!this.usable()) {
      buttons.uninstall.classList.add('hidden')
      buttons.revert.classList.add('hidden')
      buttons.disable.classList.add('hidden')
      return
    }

    if (this.disabled().length) {
      buttons.revert.classList.remove('hidden')
      buttons.uninstall.classList.add('hidden')
      buttons.disable.classList.add('hidden');
      return
    }

    buttons.revert.classList.add('hidden')
    buttons.uninstall.classList.remove('hidden')
    buttons.disable.classList.remove('hidden')
  }

  isRubberband() {
    return this.ctrlIsHeld || this.uiRubberband
  }
}
