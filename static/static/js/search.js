export async function initSearch(pagefind, currentVersion) {
    const searchInput = document.getElementById('cot-search');
    const searchResults = document.getElementById('search-results');
    const searchResultsList = document.getElementById('search-results-list');
    const searchQuerySpan = document.getElementById('search-query');
    const searchClose = document.getElementById('search-close');
    const mainContent = document.querySelector('article');
    const cotToc = document.getElementsByClassName('cot-toc');

    searchInput.addEventListener('input', async event => {
        const query = event.target.value.trim();
        if (query.length > 0) {
            await performSearch(query);
        } else {
            resetSearch();
        }
    });

    searchClose.addEventListener('click', () => {
        searchInput.value = '';
        resetSearch();
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

        searchResultsList.innerHTML = '';
        if (search.results.length === 0) {
            searchResultsList.innerHTML = '<p>No results found.</p>';
        } else {
            for (const result of search.results.slice(0, 10)) {
                const data = await result.data();
                const resultItem = document.createElement('div');
                resultItem.classList.add('mb-4');

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

    function resetSearch() {
        mainContent.classList.remove('d-none');
        for (const toc of cotToc) {
            toc.classList.remove('invisible', 'd-none', 'd-lg-block');
        }
        searchResults.classList.add('d-none');
    }
}
