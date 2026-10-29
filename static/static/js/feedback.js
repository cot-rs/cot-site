// "Was this helpful?" widget, shown in the right sidebar on wide screens and at
// the end of the chapter on narrow ones. Both copies share one state.
//
// Picking a face sends `docs_feedback` straight away and locks the faces. A sad
// face also opens an optional comment box, which sends `docs_feedback_comment`
// with the same `feedback_id`; leaving it empty keeps just the rating. Nothing
// is stored on the device: the rating only lives for this page load.
(() => {
  const version = document.currentScript?.dataset.version
  const widgets = document.querySelectorAll('[data-page-feedback]')
  // posthog.js is only loaded when analytics is enabled for the site.
  if (!widgets.length || !window.cotAnalytics) return

  const MAX_COMMENT_LENGTH = 1000
  let feedback = null

  function send(event, extra) {
    window.cotAnalytics.sendFeedback(event, {
      feedback_id: feedback.id,
      rating: feedback.rating, // 'good' | 'okay' | 'bad'
      page: window.location.pathname,
      version,
      ...extra
    })
  }

  function newId() {
    return window.crypto?.randomUUID?.() ?? `${Date.now()}-${Math.random().toString(16).slice(2)}`
  }

  function setStatus(text) {
    widgets.forEach(widget => { widget.querySelector('[data-feedback-status]').textContent = text })
  }

  function rate(rating, activeWidget) {
    if (feedback) return
    feedback = { id: newId(), rating }
    send('docs_feedback')

    for (const widget of widgets) {
      widget.querySelectorAll('[data-feedback-rating]').forEach(face => {
        const chosen = face.dataset.feedbackRating === rating
        face.disabled = true
        face.setAttribute('aria-pressed', String(chosen))
        // Bootstrap fades disabled buttons; keep the chosen face fully visible.
        face.classList.toggle('opacity-100', chosen)
        face.classList.toggle('bg-body-secondary', chosen)
      })
      if (rating === 'bad') widget.querySelector('[data-feedback-comment]').hidden = false
    }

    if (rating === 'bad') activeWidget.querySelector('textarea').focus()
    else setStatus('Thanks for your feedback!')
  }

  function sendComment(activeWidget) {
    const comment = activeWidget.querySelector('textarea').value.trim().slice(0, MAX_COMMENT_LENGTH)
    if (!comment) {
      activeWidget.querySelector('textarea').focus()
      return
    }
    send('docs_feedback_comment', { comment })
    widgets.forEach(widget => { widget.querySelector('[data-feedback-comment]').hidden = true })
    setStatus('Thanks for your feedback!')
  }

  for (const widget of widgets) {
    widget.hidden = false
    widget.addEventListener('click', event => {
      const face = event.target.closest('[data-feedback-rating]')
      if (face) rate(face.dataset.feedbackRating, widget)
      else if (event.target.closest('[data-feedback-send]')) sendComment(widget)
    })
  }
})()
