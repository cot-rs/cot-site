// How long the query must stay unchanged before it counts as one search.
// Pagefind searches as you type, so without this every keystroke would be sent.
const ANALYTICS_IDLE_MS = 1000;
const MAX_QUERY_LENGTH = 100;
// Make sure events sent right before a page navigation are not lost.
const BEACON = { transport: 'sendBeacon' };

export async function initSearch(pagefind, currentVersion) {
    const searchInput = document.getElementById('cot-search');
    const searchResults = document.getElementById('search-results');
    const searchResultsList = document.getElementById('search-results-list');
    const searchQuerySpan = document.getElementById('search-query');
    const searchClose = document.getElementById('search-close');
    const mainContent = document.querySelector('article');
    const cotToc = document.getElementsByClassName('cot-toc');

    let pendingSearch = null;
    let analyticsTimer = null;
    let lastSentQuery = null;
    let lastResultCount = 0;

    // PostHog is loaded only after the visitor makes a consent choice, and
    // posthog.js drops these events for visitors who rejected analytics.
    function track(event, properties, options) {
        window.posthog?.capture(event, properties, options);
    }

    function normalizeQuery(query) {
        return query.trim().toLowerCase().replace(/\s+/g, ' ').slice(0, MAX_QUERY_LENGTH);
    }

    function scheduleSearchEvent(query, resultCount) {
        pendingSearch = { query: normalizeQuery(query), resultCount };
        clearTimeout(analyticsTimer);
        analyticsTimer = setTimeout(() => flushSearchEvent('idle'), ANALYTICS_IDLE_MS);
    }

    function flushSearchEvent(trigger, options) {
        clearTimeout(analyticsTimer);
        if (!pendingSearch) return;
        const { query, resultCount } = pendingSearch;
        pendingSearch = null;
        if (query.length < 2 || query === lastSentQuery) return;
        lastSentQuery = query;
        track('docs_search', {
            query,
            result_count: resultCount,
            trigger, // 'idle' | 'enter' | 'blur' | 'result_click' | 'cleared' | 'closed'
            version: currentVersion,
            page: window.location.pathname,
        }, options);
    }

    searchInput.addEventListener('input', async event => {
        const query = event.target.value.trim();
        if (query.length > 0) {
            await performSearch(query);
        } else {
            flushSearchEvent('cleared');
            resetSearch();
        }
    });

    searchInput.addEventListener('keydown', event => {
        if (event.key === 'Enter') flushSearchEvent('enter');
    });

    // Clicking a result blurs the input first, and the page navigates right after.
    searchInput.addEventListener('blur', () => flushSearchEvent('blur', BEACON));

    searchClose.addEventListener('click', () => {
        flushSearchEvent('closed');
        if (lastSentQuery) {
            track('docs_search_closed', {
                query: normalizeQuery(searchInput.value),
                result_count: lastResultCount,
                version: currentVersion,
            });
        }
        searchInput.value = '';
        resetSearch();
    });

    searchResultsList.addEventListener('click', event => {
        const link = event.target.closest('a');
        if (!link) return;
        flushSearchEvent('result_click', BEACON);
        const item = link.closest('[data-result-position]');
        track('docs_search_result_clicked', {
            query: normalizeQuery(searchInput.value),
            position: item ? Number(item.dataset.resultPosition) : null,
            is_sub_result: link.closest('.search-result-sub-item') !== null,
            is_docs_rs_fallback: link.hasAttribute('data-docs-rs-fallback'),
            destination: link.getAttribute('href'),
            result_count: lastResultCount,
            version: currentVersion,
        }, BEACON);
    });

    async function performSearch(query) {
        mainContent.classList.add('d-none');
        for (const toc of cotToc) {
            toc.classList.add('invisible', 'd-none', 'd-lg-block');
        }
        searchResults.classList.remove('d-none');
        searchQuerySpan.textContent = query;

        const DEBOUNCE_DELAY = 50; // ms
        const search = await pagefind.debouncedSearch(query, {
            filters: {version: currentVersion}
        }, DEBOUNCE_DELAY);
        if (search === null) {
            // a more recent search call has been made, nothing to do
            return;
        }

        lastResultCount = search.results.length;
        scheduleSearchEvent(query, lastResultCount);

        searchResultsList.innerHTML = '';
        if (search.results.length === 0) {
            searchResultsList.appendChild(renderNoResults(query));
        } else {
            for (const [index, result] of search.results.slice(0, 10).entries()) {
                const data = await result.data();
                const resultItem = document.createElement('div');
                resultItem.classList.add('mb-4');
                resultItem.dataset.resultPosition = String(index + 1);

                resultItem.innerHTML = `
                    <h3><a href="${data.url}">${data.meta.title}</a></h3>
                    ${data.sub_results.map(data => `
                       <div class="search-result-sub-item">
                         <a href="${data.url}">${data.title}</a>
                         <p class="hit-content">${data.excerpt}</p>
                       </div>
                    `).join('')}
                `;
                searchResultsList.appendChild(resultItem);
            }
        }
    }

    // Same mapping as the guide's own API links (cot-site-macros rendering.rs):
    // "v0.6" -> "0.6" (docs.rs resolves it to the newest 0.6.x), and the
    // unreleased "master" guide -> the latest release.
    function docsRsVersion(version) {
        const match = /^v(\d+\.\d+)$/.exec(version);
        return match ? match[1] : 'latest';
    }

    // The guide index has no API items, so point those searches at docs.rs.
    function renderNoResults(query) {
        const wrapper = document.createElement('div');
        const message = document.createElement('p');
        message.textContent = 'No results found in the guide.';
        const fallback = document.createElement('p');
        const link = document.createElement('a');
        link.href = `https://docs.rs/cot/${docsRsVersion(currentVersion)}/cot/?search=${encodeURIComponent(query)}`;
        link.dataset.docsRsFallback = '';
        link.textContent = `Search the API reference on docs.rs for "${query}"`;
        fallback.appendChild(link);
        wrapper.append(message, fallback);
        return wrapper;
    }

    function resetSearch() {
        clearTimeout(analyticsTimer);
        pendingSearch = null;
        mainContent.classList.remove('d-none');
        for (const toc of cotToc) {
            toc.classList.remove('invisible', 'd-none', 'd-lg-block');
        }
        searchResults.classList.add('d-none');
    }
}
