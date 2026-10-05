const assert = require('node:assert/strict')
const fs = require('node:fs')
const vm = require('node:vm')
const source = fs.readFileSync(new URL('../../static/static/js/posthog.js', `file://${__filename}`), 'utf8')

function run({ hostname = 'cot.rs', allowLocalhost = 'false', privacy = {}, stored = null, storageBlocked = false } = {}) {
  const scripts = [], calls = [], listeners = {}
  const store = new Map(stored ? [['cot-cookie-consent-v1', stored]] : [])
  function element(action) {
    return {
      dataset: { consent: action }, hidden: true, checked: false, disabled: false, open: false, events: {},
      addEventListener(event, callback) { this.events[event] = callback },
      showModal() { this.open = true }, close() { this.open = false; this.events.close?.() },
      closest() { return null }, focus() { this.focused = true }, click() { this.events.click?.() }
    }
  }
  const elements = Object.fromEntries(['cookie-banner', 'cookie-preferences', 'cookie-analytics', 'cookie-privacy-signal', 'cookie-preferences-form'].map(id => [id, element()]))
  const buttons = Object.fromEntries(['accept', 'reject', 'customize', 'close'].map(action => [action, element(action)]))
  const window = {
    location: { hostname }, scrollX: 0, scrollY: 0,
    addEventListener: (event, callback) => { listeners[event] = callback },
    scrollTo: (x, y) => { window.scrollX = x; window.scrollY = y }
  }
  const main = element()
  const context = {
    window, navigator: privacy,
    localStorage: {
      getItem: key => { if (storageBlocked) throw new Error('Blocked'); return store.get(key) || null },
      setItem: (key, value) => { if (storageBlocked) throw new Error('Blocked'); store.set(key, value) }
    },
    document: {
      currentScript: { dataset: { projectToken: 'phc_test', apiHost: 'https://eu.i.posthog.com', allowLocalhost } },
      getElementById: id => elements[id],
      querySelectorAll: selector => selector === '[data-consent]' ? Object.values(buttons) : [],
      querySelector: selector => selector === 'main' ? main : buttons.customize,
      createElement: () => ({}), head: { appendChild: script => scripts.push(script) }
    }
  }
  vm.runInNewContext(source, context)
  const sdk = {
    config: {}, init(token, config) { this.config = config; config.loaded(this) },
    set_config(config) { Object.assign(this.config, config) },
    opt_in_capturing() { calls.push('accept') }, opt_out_capturing() { calls.push('reject') },
    capture(event) { calls.push(event) }
  }
  function load() { window.posthog = sdk; scripts.at(-1).onload() }
  return { scripts, calls, elements, buttons, store, sdk, load, listeners, main }
}

const pending = run()
assert.equal(pending.elements['cookie-banner'].hidden, false)
assert.equal(pending.scripts.length, 0)
pending.buttons.customize.click()
assert.equal(pending.elements['cookie-preferences'].open, true)
assert.equal(pending.store.size, 0)
pending.buttons.close.click()
assert.equal(pending.scripts.length, 0)
pending.buttons.reject.click()
assert.equal(pending.scripts[0].src, 'https://eu-assets.i.posthog.com/static/array.js')
pending.load()
assert.equal(pending.sdk.config.cookieless_mode, 'on_reject')
assert.equal(pending.sdk.config.disable_persistence, true)
assert.equal(pending.calls.filter(call => call === '$pageview').length, 1)
pending.buttons.accept.click()
assert.equal(pending.calls.includes('accept'), true)
assert.equal(pending.sdk.config.disable_session_recording, false)
assert.equal(pending.main.focused, true)
pending.buttons.reject.click()
assert.equal(pending.sdk.config.disable_session_recording, true)
assert.equal(pending.calls.filter(call => call === '$pageview').length, 1)

for (const hostname of ['localhost', '127.0.0.1', '[::1]', 'docs.localhost']) {
  const blocked = run({ hostname }); blocked.buttons.accept.click()
  assert.equal(blocked.scripts.length, 0)
  const allowed = run({ hostname, allowLocalhost: 'true' }); allowed.buttons.accept.click()
  assert.equal(allowed.scripts.length, 1)
}
for (const privacy of [{ globalPrivacyControl: true }, { doNotTrack: '1' }]) {
  const blocked = run({ privacy }); blocked.buttons.reject.click()
  assert.equal(blocked.scripts.length, 0)
  assert.equal(blocked.buttons.accept.disabled, true)
}
for (const stored of ['bad json', JSON.stringify({ version: 1, analytics: true, replay: true, expires: 0 })]) {
  assert.equal(run({ stored }).elements['cookie-banner'].hidden, false)
}
const blockedStorage = run({ storageBlocked: true })
blockedStorage.buttons.reject.click()
blockedStorage.load()
assert.equal(blockedStorage.sdk.config.disable_persistence, true)
console.log('Consent state and privacy checks passed')
