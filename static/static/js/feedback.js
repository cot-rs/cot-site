(() => {
  const version = document.currentScript?.dataset.version
  const widgets = document.querySelectorAll('[data-page-feedback]')
  // posthog.js is only loaded when analytics is enabled for the site.
  if (!widgets.length || !window.cotAnalytics) return

  const MAX_COMMENT_LENGTH = 1000
  const DOCS_FEEDBACK_COMMENT_TAG = "docs_feedback_comment"
  const DOCS_FEEDBACK_TAG = "docs_feedback"

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

  // Random ID that links a rating to its optional comment.
  function newId() {
    return window.crypto?.randomUUID?.() ?? `${Date.now()}-${Math.random().toString(16).slice(2)}`
  }

  function setStatus(text) {
    widgets.forEach(widget => { widget.querySelector('[data-feedback-status]').textContent = text })
  }

  function rate(rating, activeWidget) {
    if (feedback) return
    feedback = { id: newId(), rating }
    send(DOCS_FEEDBACK_TAG)

    for (const widget of widgets) {
      widget.querySelectorAll('[data-feedback-rating]').forEach(face => {
        const chosen = face.dataset.feedbackRating === rating
        face.disabled = true
        face.setAttribute('aria-pressed', String(chosen))
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
    send(DOCS_FEEDBACK_COMMENT_TAG, { comment })
    widgets.forEach(widget => {
      widget.querySelector('[data-feedback-comment]').hidden = true
    })
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
