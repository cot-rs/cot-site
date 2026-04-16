const BUTTON_TEXT = 'Copy'
const COPY_LABEL = 'Copy'
const COPIED_LABEL = 'Copied'
const ERROR_LABEL = 'Failed to copy'

async function copyCode(button) {
  const container = button.closest('.code-block')
  const code = container?.querySelector('pre code')

  if (!code) {
    return
  }

  try {
    await navigator.clipboard.writeText(code.innerText)
    setButtonState(button, COPIED_LABEL, 'true')
  } catch {
    setButtonState(button, ERROR_LABEL, 'false')
  }
}

function setButtonState(button, label, copied) {
  button.textContent = label
  button.setAttribute('aria-label', label)
  button.setAttribute('title', label)
  button.dataset.copied = copied

  window.clearTimeout(button._copyResetTimer)
  button._copyResetTimer = window.setTimeout(() => {
    button.textContent = BUTTON_TEXT
    button.setAttribute('aria-label', COPY_LABEL)
    button.setAttribute('title', COPY_LABEL)
    delete button.dataset.copied
  }, 1600)
}

document.addEventListener('click', (event) => {
  const button = event.target.closest('[data-copy-code]')

  if (!button) {
    return
  }

  copyCode(button)
})
