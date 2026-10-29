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
    const text = code.innerText.trim()
    await navigator.clipboard.writeText(text)
    setButtonState(button, COPIED_LABEL, 'true')
    trackCopy(button, text)
  } catch {
    setButtonState(button, ERROR_LABEL, 'false')
  }
}

function classifySnippet(text) {
  if (/cargo install cot-cli/.test(text)) return 'install_cli'
  if (/\bcot new\b/.test(text)) return 'new_project'
  if (/\bbacon\b|\bcargo run\b/.test(text)) return 'run_project'
  return 'code'
}

function trackCopy(button, text) {
  const blocks = Array.from(document.querySelectorAll('.code-block'))
  window.posthog?.capture('code_copied', {
    snippet_kind: classifySnippet(text),
    snippet_preview: text.slice(0, 80),
    snippet_index: blocks.indexOf(button.closest('.code-block')),
    method: 'button',
    page: window.location.pathname
  })
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
