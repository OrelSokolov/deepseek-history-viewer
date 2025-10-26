// Blazing fast ngram substring search using Rust Tantivy backend
class RustSearch {
    constructor() {
        this.apiUrl = '/api/search';
        this.debounceMs = 100; // Faster realtime search
        this.timeout = null;
        this.cache = new Map();
        this.minQueryLength = 2; // Ngram tokenizer требует минимум 2 символа
    }

    async search(query, limit = 50) {
        if (!query.trim()) return [];

        // Check cache
        const cacheKey = query.toLowerCase();
        if (this.cache.has(cacheKey)) {
            return this.cache.get(cacheKey);
        }

        try {
            const start = performance.now();
            const url = `${this.apiUrl}?q=${encodeURIComponent(query)}&limit=${limit}`;
            const response = await fetch(url);
            
            if (!response.ok) throw new Error(`Search failed: ${response.status}`);

            const data = await response.json();
            const totalTime = performance.now() - start;
            
            console.log(`🚀 Search: "${query}" → ${data.total} results in ${totalTime.toFixed(1)}ms (server: ${data.time_ms}ms)`);
            
            const results = data.results.map(r => ({
                id: r.conversation_id,
                title: r.title,
                url: `/conversations/${r.conversation_id}/`,
                snippet: r.snippet,
                score: r.score
            }));

            // Cache results
            this.cache.set(cacheKey, results);
            if (this.cache.size > 100) {
                const firstKey = this.cache.keys().next().value;
                this.cache.delete(firstKey);
            }

            return results;
        } catch (error) {
            console.error('Search error:', error);
            return [];
        }
    }
}

const search = new RustSearch();

document.addEventListener('DOMContentLoaded', () => {
    const searchInput = document.getElementById('searchInput');
    const searchResults = document.getElementById('searchResults');
    const conversationsList = document.getElementById('conversationsList');

    if (!searchInput) return;

    // Search input handler
    searchInput.addEventListener('input', (e) => {
        const query = e.target.value.trim();

        clearTimeout(search.timeout);

        if (!query) {
            searchResults.innerHTML = '';
            searchResults.style.display = 'none';
            conversationsList.style.display = 'block';
            return;
        }

        // Don't search for too short queries
        if (query.length < search.minQueryLength) {
            searchResults.innerHTML = '<div class="search-loading">Введите хотя бы 2 символа для ngram поиска...</div>';
            searchResults.style.display = 'block';
            conversationsList.style.display = 'none';
            return;
        }

        searchResults.innerHTML = '<div class="search-loading">🔍 Поиск...</div>';
        searchResults.style.display = 'block';
        conversationsList.style.display = 'none';

        search.timeout = setTimeout(async () => {
            const results = await search.search(query);
            displayResults(results, query);
        }, search.debounceMs);
    });

    function displayResults(results, query) {
        if (!results.length) {
            searchResults.innerHTML = '<div class="search-no-results">Ничего не найдено</div>';
            return;
        }

        const fragment = document.createDocumentFragment();
        
        results.forEach(r => {
            const item = document.createElement('a');
            item.href = r.url;
            item.className = 'search-result-item';
            
            const title = document.createElement('div');
            title.className = 'search-result-title';
            title.innerHTML = highlight(r.title, query);
            item.appendChild(title);
            
            if (r.snippet) {
                const snippet = document.createElement('div');
                snippet.className = 'search-result-snippet';
                snippet.innerHTML = highlight(r.snippet, query);
                item.appendChild(snippet);
            }
            
            fragment.appendChild(item);
        });

        searchResults.innerHTML = '';
        searchResults.appendChild(fragment);
    }

    function highlight(text, query) {
        const regex = new RegExp(`(${escapeRegex(query)})`, 'gi');
        return text.replace(regex, '<mark>$1</mark>');
    }

    function escapeRegex(str) {
        return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    }

    // Keyboard shortcuts
    document.addEventListener('keydown', (e) => {
        // Ctrl/Cmd + K
        if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
            e.preventDefault();
            searchInput.focus();
            searchInput.select();
        }
        
        // Escape
        if (e.key === 'Escape' && document.activeElement === searchInput) {
            searchInput.value = '';
            searchInput.dispatchEvent(new Event('input'));
        }
    });
});
