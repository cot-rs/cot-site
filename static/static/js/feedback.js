(() => {
  const version = document.currentScript?.dataset.version
  const widgets = document.querySelectorAll('[data-page-feedback]')
  // posthog.js is only loaded when analytics is enabled for the site.
  if (!widgets.length || !window.cotAnalytics) return

  const maxCommentLen = 1000
  const docsFeedbackComment = "docs_feedback_comment"
  const docsFeedback = "docs_feedback"

  let feedback = null

  function send(event, extra) {
    window.cotAnalytics.sendFeedback(event, {
      feedback_id: feedback.id,
      rating: feedback.rating,
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
    send(docsFeedback)

    for (const widget of widgets) {
      // custom.scss highlights the face with aria-pressed="true".
      widget.querySelectorAll('[data-feedback-rating]').forEach(face => {
        face.disabled = true
        face.setAttribute('aria-pressed', String(face.dataset.feedbackRating === rating))
      })
      widget.querySelector('[data-feedback-comment]').hidden = false
    }

    activeWidget.querySelector('textarea').focus()
  }


  function sendComment(activeWidget) {
    const comment = activeWidget.querySelector('textarea').value.trim().slice(0, maxCommentLen)
    if (!comment) {
      activeWidget.querySelector('textarea').focus()
      return
    }
    send(docsFeedbackComment, { comment })
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
    // ctrl+Enter (cmd+Enter on macOS) sends the comment.
    widget.addEventListener('keydown', event => {
      if (event.key === 'Enter' && (event.ctrlKey || event.metaKey) && event.target.matches('textarea')) {
        event.preventDefault()
        sendComment(widget)
      }
    })
  }
})()
