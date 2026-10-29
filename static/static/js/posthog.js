(() => {
  const configuration = document.currentScript?.dataset
  const banner = document.getElementById('cookie-banner')
  if (!configuration || !banner) return

  const storageKey = '__cot-cookie-consent-v1'
  const lifetime = 180 * 24 * 60 * 60 * 1000 // 180 days
  // Keep visits from local development out of the site's analytics.
  const local = ['localhost', '127.0.0.1', '[::1]', '::1'].includes(window.location.hostname)
    || window.location.hostname.endsWith('.localhost')
  // Global Privacy Control (GPC) and Do Not Track (DNT) are browser privacy signals.
  // We treat either as a full analytics opt-out.
  const privacySignal = navigator.globalPrivacyControl === true || navigator.doNotTrack === '1'
    || window.doNotTrack === '1'
  const trackingAllowed = !privacySignal && !local
  let preferences = readPreferences()
  let sdk = null
  let loading = false
  let pageviewMode = null
  let returnFocus = null
  let previousAnalytics = null

  function readPreferences() {
    try {
      const saved = JSON.parse(localStorage.getItem(storageKey))
      const now = Date.now()
      // Ignore expired choices and expiry dates beyond our 180-day limit.
      if (saved?.version === 1 && typeof saved.analytics === 'boolean'
        && typeof saved.expires === 'number'
        && saved.expires > now && saved.expires <= now + lifetime) return saved
    } catch { /* Storage may be blocked; ask for a choice again. */ }
    return null
  }

  function save(allowAnalytics) {
    preferences = {
      version: 1,
      analytics: !privacySignal && allowAnalytics,
      expires: Date.now() + lifetime
    }
    try { localStorage.setItem(storageKey, JSON.stringify(preferences)) } catch { /* The choice still works for this page. */ }
    banner.hidden = true
    focusPageWithoutScrolling()
    if (sdk) applyPreferences()
    else loadSdk()
  }

  function focusPageWithoutScrolling() {
    const x = window.scrollX
    const y = window.scrollY
    const target = returnFocus || document.querySelector('main')
    target?.focus({ preventScroll: true })
    returnFocus = null
    window.scrollTo(x, y)
  }

  function applyPreferences() {
    if (!sdk) return
    // Pause interaction capture and replay before we apply the new choice.
    sdk.set_config({ autocapture: false, disable_session_recording: true })
    // No choice, a browser privacy signal, or localhost blocks new events.
    if (!preferences || !trackingAllowed) {
      sdk.opt_out_capturing()
      previousAnalytics = null
      return
    }
    const consented = preferences.analytics
    // We only switch PostHog's consent mode when the choice changes.
    if (previousAnalytics !== consented) {
      if (consented) sdk.opt_in_capturing({ captureEventName: false })
      else sdk.opt_out_capturing()
      previousAnalytics = consented
    }
    if (consented) sdk.set_config({ autocapture: true, disable_session_recording: false })
    // A change from Reject to Allow gives this page a new consented pageview.
    if (pageviewMode !== consented) {
      sdk.capture('$pageview')
      pageviewMode = consented
    }
  }

  function loadSdk() {
    // Wait for a choice before downloading PostHog, even for cookieless visits.
    if (!preferences || !trackingAllowed || loading || sdk) return
    loading = true
    const script = document.createElement('script')
    script.async = true
    script.crossOrigin = 'anonymous'
    // PostHog serves the SDK from its asset host
    script.src = configuration.apiHost.replace('.i.posthog.com', '-assets.i.posthog.com') + '/static/array.js'
    script.onerror = () => { loading = false }
    script.onload = () => {
      loading = false
      if (!window.posthog) return
      // PostHog SDK options: https://posthog.com/docs/libraries/js/config
      window.posthog.init(configuration.projectToken, {
        api_host: configuration.apiHost,
        defaults: '2026-08-30',
        cookieless_mode: 'on_reject',
        person_profiles: 'never',
        persistence: 'localStorage+cookie',
        cross_subdomain_cookie: false,
        cookie_expiration: 180,
        // We send the pageview ourselves after applying the visitor's choice.
        capture_pageview: false,
        capture_pageleave: true,
        autocapture: false,
        capture_performance: true,
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
          // After Reject, only the cookieless pageview may leave the browser.
          if (!preferences.analytics && event.event !== '$pageview') return null
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
        case 'accept': save(true); break
        case 'reject': save(false); break
        case 'show-banner':
          returnFocus = button
          banner.hidden = false
          banner.querySelector(privacySignal ? '[data-consent="reject"]' : '[data-consent="accept"]')?.focus({ preventScroll: true })
          break
      }
    })
    if (privacySignal && button.dataset.consent === 'accept') button.disabled = true
  })
  window.addEventListener('storage', event => {
    // Apply consent changes made in another tab to this tab as well.
    if (event.key !== storageKey && event.key !== null) return
    preferences = readPreferences()
    banner.hidden = preferences !== null || privacySignal
    if (sdk) applyPreferences()
    else loadSdk()
  })

  // Page feedback is something the visitor deliberately submits, not tracking,
  // so it is sent whatever the analytics choice. Visitors who allowed analytics
  // send it through the SDK, so it joins their session. Everyone else sends one
  // request with a single-use random ID: nothing is stored on the device, and
  // the feedback can't be linked to any of the visitor's other events.
  function sendFeedback(event, properties) {
    if (local) {
      console.info('[feedback] not sent from a local host:', event, properties)
      return
    }
    if (sdk && trackingAllowed && preferences?.analytics) {
      sdk.capture(event, properties)
      return
    }
    fetch(`${configuration.apiHost}/i/v0/e/`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'omit',
      keepalive: true,
      body: JSON.stringify({
        api_key: configuration.projectToken,
        event,
        distinct_id: properties.feedback_id,
        properties: { ...properties, $process_person_profile: false }
      })
    }).catch(() => { /* Feedback is best effort. */ })
  }

  window.cotAnalytics = { sendFeedback }

  document.getElementById('cookie-privacy-signal').hidden = !privacySignal
  banner.hidden = preferences !== null || privacySignal
  loadSdk()
})()
