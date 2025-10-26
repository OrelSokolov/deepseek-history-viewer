// Blazing fast search using Rust backend (Tantivy)
class RustChatSearch {
    constructor() {
        this.apiUrl = '/api/search';
        this.maxResults = 50;
        this.cache = new Map(); // Cache search results
        this.cacheSize = 100;
    }

    // Fast search via Rust API
    async search(query, limit = this.maxResults) {
        if (!query) return [];

        // Check cache first
        const cacheKey = `${query}:${limit}`;
        if (this.cache.has(cacheKey)) {
            console.log('⚡ Cache hit for:', query);
            return this.cache.get(cacheKey);
        }

        try {
            const startTime = performance.now();
            const response = await fetch(`${this.apiUrl}?q=${encodeURIComponent(query)}&limit=${limit}`);
            
            if (!response.ok) {
                throw new Error(`Search failed: ${response.status}`);
            }

            const data = await response.json();
            const searchTime = performance.now() - startTime;
            
            console.log(`🚀 Rust search completed in ${searchTime.toFixed(2)}ms (server: ${data.time_ms}ms, ${data.total} results)`);
            
            // Transform results to match Jekyll format
            const results = data.results.map(r => ({
                id: r.conversation_id,
                title: r.title,
                url: `/conversations/${r.conversation_id}/`,
                inserted_at: r.date,
                snippet: r.snippet,
                score: r.score
            }));

            // Cache results
            this.cache.set(cacheKey, results);
            if (this.cache.size > this.cacheSize) {
                const firstKey = this.cache.keys().next().value;
                this.cache.delete(firstKey);
            }

            return results;
        } catch (error) {
            console.error('Search error:', error);
            return [];
        }
    }

    // Get snippet with highlighting
    getSnippet(snippet, query) {
        if (!snippet) return '';
        
        // Simple highlight - regex-based
        const regex = new RegExp(`(${escapeRegex(query)})`, 'gi');
        return snippet.replace(regex, '<mark>$1</mark>');
    }
}

// Escape special regex characters
function escapeRegex(str) {
    return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

// Initialize search
const chatSearch = new RustChatSearch();
let searchTimeout = null;

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', async () => {
    const searchInput = document.getElementById('searchInput');
    const searchResults = document.getElementById('searchResults');
    const conversationsList = document.getElementById('conversationsList');

    if (!searchInput || !searchResults || !conversationsList) {
        console.warn('Search elements not found');
        return;
    }

    console.log('🔍 Rust search initialized');

    // Handle search input with debounce
    searchInput.addEventListener('input', (e) => {
        const query = e.target.value.trim();

        // Clear previous timeout
        if (searchTimeout) clearTimeout(searchTimeout);

        // Instant clear if empty
        if (!query) {
            searchResults.innerHTML = '';
            searchResults.style.display = 'none';
            conversationsList.style.display = 'block';
            return;
        }

        // Debounce search (200ms for network request)
        searchTimeout = setTimeout(async () => {
            searchResults.innerHTML = '<div class="search-loading">🔍 Поиск...</div>';
            searchResults.style.display = 'block';
            conversationsList.style.display = 'none';

            const results = await chatSearch.search(query);
            displaySearchResults(results, query);
        }, 200);
    });

    // Display search results
    function displaySearchResults(results, query) {
        if (results.length === 0) {
            searchResults.innerHTML = '<div class="search-no-results">Ничего не найдено</div>';
            searchResults.style.display = 'block';
            conversationsList.style.display = 'none';
            return;
        }

        // Use DocumentFragment for better performance
        const fragment = document.createDocumentFragment();
        
        results.forEach(result => {
            const item = document.createElement('a');
            item.href = result.url;
            item.className = 'search-result-item';
            
            // Title with highlighting
            const title = document.createElement('div');
            title.className = 'search-result-title';
            title.innerHTML = chatSearch.getSnippet(result.title, query);
            item.appendChild(title);
            
            // Snippet from Rust backend
            if (result.snippet) {
                const snippetDiv = document.createElement('div');
                snippetDiv.className = 'search-result-snippet';
                snippetDiv.innerHTML = chatSearch.getSnippet(result.snippet, query);
                item.appendChild(snippetDiv);
            }

            // Score badge (optional)
            if (result.score) {
                const score = document.createElement('div');
                score.className = 'search-result-score';
                score.textContent = `Score: ${result.score.toFixed(2)}`;
                item.appendChild(score);
            }
            
            fragment.appendChild(item);
        });

        // Single DOM update
        searchResults.innerHTML = '';
        searchResults.appendChild(fragment);
        searchResults.style.display = 'block';
        conversationsList.style.display = 'none';
    }

    // Keyboard shortcuts
    document.addEventListener('keydown', (e) => {
        // Ctrl+K or Cmd+K to focus search
        if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
            e.preventDefault();
            searchInput.focus();
            searchInput.select();
        }
        
        // Escape to clear search
        if (e.key === 'Escape' && document.activeElement === searchInput) {
            searchInput.value = '';
            searchInput.dispatchEvent(new Event('input'));
            searchInput.blur();
        }
    });
});

// Auto-render KaTeX formulas on page load
document.addEventListener('DOMContentLoaded', () => {
    // Check if KaTeX is available
    if (typeof renderMathInElement !== 'undefined') {
        renderMathInElement(document.body, {
            delimiters: [
                {left: '$$', right: '$$', display: true},
                {left: '$', right: '$', display: false},
                {left: '\\[', right: '\\]', display: true},
                {left: '\\(', right: '\\)', display: false}
            ],
            throwOnError: false,
            trust: true
        });
        console.log('✅ KaTeX formulas rendered');
    }
});

