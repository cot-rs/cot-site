// The SDK can be simulated, or supplied via POSTHOG_SDK_FILE for an offline SDK smoke test.
const assert = require('node:assert/strict')
const fs = require('node:fs')
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright')
const [disabledUrl, enabledUrl] = process.argv.slice(2)
if (!disabledUrl || !enabledUrl) throw new Error('Provide disabled and enabled server URLs')
const sdkSource = process.env.POSTHOG_SDK_FILE ? fs.readFileSync(process.env.POSTHOG_SDK_FILE, 'utf8') : `
window.posthog = {
  config: {}, consent: 'pending',
  init(token, config) { this.config = config; config.loaded(this) },
  set_config(config) { Object.assign(this.config, config) },
  get_config(key) { return this.config[key] },
  opt_in_capturing() { this.consent = 'granted' },
  opt_out_capturing() { this.consent = 'denied' },
  get_explicit_consent_status() { return this.consent },
  capture() {}
}`
const probe = `
const originalInit = window.posthog.init.bind(window.posthog)
window.posthog.init = (token, config) => {
  const loaded = config.loaded
  config.loaded = instance => {
    window.analyticsCheck = { calls: [], initial: { ...config } }
    for (const method of ['opt_in_capturing', 'opt_out_capturing', 'capture']) {
      const original = instance[method].bind(instance)
      instance[method] = (...args) => { window.analyticsCheck.calls.push([method, ...args]); return original(...args) }
    }
    loaded(instance)
  }
  return originalInit(token, config)
}`

;(async () => {
  const browser = await chromium.launch({ executablePath: process.env.CHROME_PATH, headless: true })
  let requests = 0
  let failSdk = false
  const errors = []
  async function context(options = {}) {
    const ctx = await browser.newContext(options)
    await ctx.route('**/*', route => {
      const url = new URL(route.request().url())
      if (url.hostname.endsWith('.posthog.com')) {
        requests++
        if (url.pathname.endsWith('/static/array.js')) {
          if (failSdk) return route.abort()
          return route.fulfill({ contentType: 'application/javascript', headers: { 'access-control-allow-origin': '*' }, body: sdkSource + '\n' + probe })
        }
        return route.fulfill({ contentType: 'application/json', headers: { 'access-control-allow-origin': '*' }, body: '{}' })
      }
      if (!['127.0.0.1', 'localhost'].includes(url.hostname)) return route.abort()
      return route.continue()
    })
    ctx.on('page', page => page.on('pageerror', error => errors.push(error.message)))
    return ctx
  }
  async function waitForConsent(page, consent) {
    await page.waitForFunction(value => window.posthog?.get_explicit_consent_status() === value, consent)
  }
  const bannerButton = (page, name) => page.locator('#cookie-banner').getByRole('button', { name, exact: true })
  try {
    const ctx = await context()
    const page = await ctx.newPage()
    await page.goto(disabledUrl)
    assert.equal(await page.locator('#cookie-banner').count(), 0)
    assert.equal(requests, 0)
    await page.goto(enabledUrl)
    assert.equal(await page.locator('#cookie-banner').isVisible(), true)
    assert.equal(requests, 0, 'No SDK or telemetry before a choice')
    assert.equal(await page.evaluate(() => localStorage.getItem('cot-cookie-consent-v1')), null)

    await bannerButton(page, 'Customize').click()
    assert.equal(await page.getByRole('switch', { name: 'strictly Necessary', exact: true }).isChecked(), true)
    assert.equal(await page.getByRole('switch', { name: 'strictly Necessary', exact: true }).isDisabled(), true)
    assert.equal(await page.getByRole('switch', { name: 'Analytics', exact: true }).count(), 1)
    assert.equal(await page.getByRole('checkbox').count(), 0)
    assert.equal(await page.locator('#cookie-analytics').isChecked(), false)
    assert.equal(await page.locator('#cookie-replay').count(), 0)
    assert.equal(await page.locator('#cookie-preferences details').count(), 2)
    await page.locator('#cookie-preferences summary').first().click()
    assert.equal(await page.locator('#cookie-preferences details').first().getAttribute('open'), '')
    await page.keyboard.press('Escape')
    assert.equal(requests, 0, 'Closing customization is not consent')

    await bannerButton(page, 'Reject all').click()
    await waitForConsent(page, 'denied')
    assert.equal(await page.evaluate(() => posthog.config.cookieless_mode), 'on_reject')
    assert.equal(await page.evaluate(() => posthog.config.disable_session_recording), true)
    assert.equal(await page.evaluate(() => posthog.config.disable_persistence), true)
    assert.equal((await ctx.cookies()).filter(cookie => cookie.name.startsWith('ph_')).length, 0)
    assert.equal(await page.evaluate(() => analyticsCheck.calls.filter(call => call[0] === 'capture' && call[1] === '$pageview').length), 1)
    await page.reload()
    await waitForConsent(page, 'denied')
    assert.equal(await page.locator('#cookie-banner').isVisible(), false)

    await page.getByRole('button', { name: 'Cookie settings', exact: true }).click()
    await page.locator('#cookie-analytics').check()
    await page.getByRole('button', { name: 'Save preferences', exact: true }).click()
    await waitForConsent(page, 'granted')
    assert.equal(await page.evaluate(() => posthog.config.disable_session_recording), false)
    assert.equal(await page.evaluate(() => posthog.config.disable_persistence), false)
    assert.equal(await page.evaluate(() => posthog.config.session_recording.maskAllInputs), true)
    assert.equal(await page.evaluate(() => posthog.config.mask_all_text), false)
    assert.equal(await page.evaluate(() => posthog.config.session_recording.maskTextSelector), '.ph-sensitive, [data-ph-mask]')
    assert.equal(await page.evaluate(() => analyticsCheck.calls.filter(call => call[0] === 'capture' && call[1] === '$pageview').length), 1, 'Preference changes do not duplicate pageviews')

    const other = await ctx.newPage()
    await other.goto(enabledUrl)
    await waitForConsent(other, 'granted')
    await page.getByRole('button', { name: 'Cookie settings', exact: true }).click()
    await page.locator('#cookie-preferences').getByRole('button', { name: 'Reject all', exact: true }).click()
    await waitForConsent(other, 'denied')
    assert.equal(await other.evaluate(() => posthog.config.disable_session_recording), true)
    assert.equal((await ctx.cookies()).filter(cookie => cookie.name.startsWith('ph_')).length, 0, 'Withdrawal removes analytics cookies across tabs')

    const fresh = await context({ viewport: { width: 375, height: 812 } })
    const mobile = await fresh.newPage()
    await mobile.goto(enabledUrl)
    await mobile.evaluate(() => document.fonts.ready)
    assert.equal(await mobile.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true)
    if (process.env.CONSENT_ARTIFACTS) {
      fs.mkdirSync(process.env.CONSENT_ARTIFACTS, { recursive: true })
      await mobile.screenshot({ path: process.env.CONSENT_ARTIFACTS + '/banner-mobile.png' })
    }
    await mobile.evaluate(() => scrollTo(0, Math.min(300, document.documentElement.scrollHeight - innerHeight)))
    const scrollBeforeAccept = await mobile.evaluate(() => scrollY)
    await bannerButton(mobile, 'Accept all').click()
    assert.equal(await mobile.evaluate(() => scrollY), scrollBeforeAccept, 'Accepting consent keeps the current scroll position')
    await waitForConsent(mobile, 'granted')
    assert.equal(await mobile.evaluate(() => posthog.config.disable_session_recording), false)
    await mobile.getByRole('button', { name: 'Cookie settings', exact: true }).click()
    await mobile.locator('#cookie-analytics').uncheck()
    assert.equal(await mobile.locator('#cookie-replay').count(), 0)
    await mobile.locator('#cookie-preferences details').evaluateAll(details => details.forEach(item => { item.open = true }))
    assert.equal(await mobile.locator('#cookie-preferences').evaluate(dialog => dialog.scrollWidth <= dialog.clientWidth), true)
    if (process.env.CONSENT_ARTIFACTS) await mobile.screenshot({ path: process.env.CONSENT_ARTIFACTS + '/preferences-mobile.png' })
    await fresh.close()

    const blocked = await context()
    // Isolate consent's storage fallback from the existing theme script,
    // which assumes localStorage is available.
    await blocked.route('**/static/js/color-modes.js*', route => route.fulfill({ contentType: 'application/javascript', body: '' }))
    await blocked.addInitScript(() => {
      Storage.prototype.getItem = () => { throw new Error('Storage blocked') }
      Storage.prototype.setItem = () => { throw new Error('Storage blocked') }
    })
    const blockedPage = await blocked.newPage()
    await blockedPage.goto(enabledUrl)
    await bannerButton(blockedPage, 'Reject all').click()
    await waitForConsent(blockedPage, 'denied')
    await blocked.close()

    const privacy = await context()
    await privacy.addInitScript(() => Object.defineProperty(navigator, 'globalPrivacyControl', { value: true }))
    const privacyPage = await privacy.newPage()
    const beforePrivacy = requests
    await privacyPage.goto(enabledUrl)
    await privacyPage.getByRole('button', { name: 'Cookie settings', exact: true }).click()
    assert.equal(await privacyPage.locator('#cookie-analytics').isDisabled(), true)
    assert.equal(requests, beforePrivacy)
    await privacy.close()

    failSdk = true
    const unavailable = await context()
    const failed = await unavailable.newPage()
    await failed.goto(enabledUrl)
    await bannerButton(failed, 'Accept all').click()
    await failed.locator('.navbar').getByRole('link', { name: 'FAQ', exact: true }).click()
    await failed.waitForURL('**/faq/')
    await unavailable.close()
    assert.deepEqual(errors, [])
    console.log('Consent, customization, persistence, withdrawal, mobile, privacy, and SDK-failure checks passed; no telemetry sent')
  } finally { await browser.close() }
})().catch(error => { console.error(error); process.exitCode = 1 })
