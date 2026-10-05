(() => {
  const configuration = document.currentScript?.dataset
  const banner = document.getElementById('cookie-banner')
  const dialog = document.getElementById('cookie-preferences')
  if (!configuration || !banner || !dialog) return

  const storageKey = 'cot-cookie-consent-v1'
  const lifetime = 180 * 24 * 60 * 60 * 1000
  const analytics = document.getElementById('cookie-analytics')
  const local = ['localhost', '127.0.0.1', '[::1]', '::1'].includes(window.location.hostname)
    || window.location.hostname.endsWith('.localhost')
  const privacySignal = navigator.globalPrivacyControl === true || navigator.doNotTrack === '1'
    || window.doNotTrack === '1'
  const trackingAllowed = !privacySignal && (!local || configuration.allowLocalhost === 'true')
  let preferences = readPreferences()
  let sdk = null
  let loading = false
  let pageviewCaptured = false
  let returnFocus = null
  let previousAnalytics = null

  document.querySelectorAll('.cookie-option details').forEach(details => {
    const summary = details.querySelector('summary')
    const content = details.querySelector('.cookie-accordion-content')
    let animation = null
    let closing = false
    if (!summary || !content) return

    summary.addEventListener('click', event => {
      event.preventDefault()
      const opening = !details.open || closing
      const currentHeight = content.getBoundingClientRect().height
      const currentOpacity = Number.parseFloat(getComputedStyle(content).opacity) || 0
      animation?.cancel()
      animation = null
      closing = false
      if (matchMedia('(prefers-reduced-motion: reduce)').matches) {
        details.open = opening
        return
      }
      details.open = true
      const targetHeight = opening ? content.scrollHeight : 0
      animation = content.animate([
        { height: `${currentHeight}px`, opacity: currentOpacity, transform: `translateY(${opening ? '-.25rem' : '0'})` },
        { height: `${targetHeight}px`, opacity: opening ? 1 : 0, transform: opening ? 'translateY(0)' : 'translateY(-.25rem)' }
      ], { duration: 260, easing: 'cubic-bezier(.2,.75,.25,1)' })
      closing = !opening
      animation.onfinish = () => {
        if (closing) details.open = false
        closing = false
        animation = null
      }
    })
  })

  function readPreferences() {
    try {
      const saved = JSON.parse(localStorage.getItem(storageKey))
      if (saved?.version === 1 && typeof saved.analytics === 'boolean'
        && typeof saved.expires === 'number'
        && saved.expires > Date.now() && saved.expires <= Date.now() + lifetime) return saved
    } catch { /* Blocked storage leaves preferences in memory for this page. */ }
    return null
  }

  function updateChoices() {
    analytics.disabled = privacySignal
    document.getElementById('cookie-privacy-signal').hidden = !privacySignal
  }

  function openPreferences(button) {
    returnFocus = button
    analytics.checked = !privacySignal && (preferences?.analytics || false)
    updateChoices()
    if (!dialog.open) dialog.showModal()
  }

  function closePreferences() {
    if (dialog.open) dialog.close()
  }

  function save(allowAnalytics) {
    preferences = {
      version: 1,
      analytics: !privacySignal && allowAnalytics,
      expires: Date.now() + lifetime
    }
    try { localStorage.setItem(storageKey, JSON.stringify(preferences)) } catch { /* Memory only. */ }
    banner.hidden = true
    closePreferences()
    if (!dialog.open) focusPageWithoutScrolling()
    if (sdk) applyPreferences()
    else loadSdk()
  }

  function focusPageWithoutScrolling() {
    const x = window.scrollX
    const y = window.scrollY
    document.querySelector('main')?.focus({ preventScroll: true })
    window.scrollTo(x, y)
  }

  function applyPreferences() {
    if (!sdk) return
    // A rejected analytics choice must stop an accepted session recording immediately.
    sdk.set_config({ disable_session_recording: true })
    if (!preferences || !trackingAllowed) {
      sdk.opt_out_capturing()
      previousAnalytics = null
      return
    }
    // PostHog's consent APIs switch between persistent and cookieless capture.
    sdk.set_config({ disable_persistence: !preferences.analytics })
    if (previousAnalytics !== preferences.analytics) {
      if (preferences.analytics) sdk.opt_in_capturing({ captureEventName: false })
      else sdk.opt_out_capturing()
      previousAnalytics = preferences.analytics
    }
    sdk.set_config({ autocapture: true, capture_performance: true })
    sdk.set_config({ disable_session_recording: !preferences.analytics })
    if (!pageviewCaptured) {
      sdk.capture('$pageview')
      pageviewCaptured = true
    }
  }

  function loadSdk() {
    if (!preferences || !trackingAllowed || loading || sdk) return
    loading = true
    const script = document.createElement('script')
    script.async = true
    script.crossOrigin = 'anonymous'
    script.src = configuration.apiHost.replace('.i.posthog.com', '-assets.i.posthog.com') + '/static/array.js'
    script.onerror = () => { loading = false }
    script.onload = () => {
      loading = false
      if (!window.posthog) return
      window.posthog.init(configuration.projectToken, {
        api_host: configuration.apiHost,
        defaults: '2026-05-30',
        cookieless_mode: 'on_reject',
        person_profiles: 'never',
        persistence: 'localStorage+cookie',
        cross_subdomain_cookie: false,
        cookie_expiration: 180,
        disable_persistence: !preferences?.analytics,
        // Consent is applied before the one explicit pageview.
        capture_pageview: false,
        capture_pageleave: true,
        autocapture: false,
        capture_performance: false,
        mask_all_text: false,
        mask_all_element_attributes: true,
        disable_session_recording: true,
        enable_recording_console_log: false,
        session_recording: {
          maskAllInputs: true,
          maskTextSelector: '.ph-sensitive, [data-ph-mask]',
          recordHeaders: false,
          recordBody: false,
          recordCrossOriginIframes: false
        },
        disable_surveys: true,
        capture_exceptions: false,
        capture_heatmaps: false,
        before_send: event => {
          if (!preferences || !trackingAllowed) return null
          return event
        },
        loaded: instance => {
          sdk = instance
          applyPreferences()
        }
      })
    }
    document.head.appendChild(script)
  }

  document.querySelectorAll('[data-consent]').forEach(button => {
    button.addEventListener('click', () => {
      switch (button.dataset.consent) {
        case 'accept': save(true, true); break
        case 'reject': save(false, false); break
        case 'customize': openPreferences(button); break
        case 'close': closePreferences(); break
      }
    })
    if (privacySignal && button.dataset.consent === 'accept') button.disabled = true
  })
  analytics.addEventListener('change', updateChoices)
  document.getElementById('cookie-preferences-form').addEventListener('submit', event => {
    event.preventDefault()
    save(analytics.checked)
  })
  dialog.addEventListener('close', () => {
    const target = returnFocus && !returnFocus.closest('[hidden]')
      ? returnFocus : document.querySelector('main')
    const x = window.scrollX
    const y = window.scrollY
    target?.focus({ preventScroll: true })
    window.scrollTo(x, y)
  })
  window.addEventListener('storage', event => {
    if (event.key !== storageKey && event.key !== null) return
    preferences = readPreferences()
    banner.hidden = preferences !== null
    if (dialog.open) {
      analytics.checked = preferences?.analytics || false
      updateChoices()
    }
    if (sdk) applyPreferences()
    else loadSdk()
  })

  banner.hidden = preferences !== null || privacySignal
  loadSdk()
})()
