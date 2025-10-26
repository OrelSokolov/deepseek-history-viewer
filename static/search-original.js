// Blazing fast search engine for chat conversations
class ChatSearch {
    constructor() {
        this.conversations = [];
        this.normalizedConversations = [];
        this.initialized = false;
        this.maxResults = 50; // Limit results for performance
    }

    // Load and preprocess search index
    async init() {
        try {
            const response = await fetch('/assets/data/search_index.json');
            this.conversations = await response.json();
            
            // Precompute normalized versions once
            this.normalizedConversations = this.conversations.map(conv => ({
                id: conv.id,
                title: conv.title,
                titleLower: conv.title.toLowerCase(),
                contentLower: conv.content.toLowerCase(),
                url: `/conversations/${conv.id}/`,
                inserted_at: conv.inserted_at,
                content: conv.content
            }));
            
            this.initialized = true;
            console.log(`🚀 Search index loaded: ${this.conversations.length} conversations`);
        } catch (error) {
            console.error('Failed to load search index:', error);
        }
    }

    // Fast search with early exit
    search(query) {
        if (!this.initialized || !query) return [];

        const queryLower = query.toLowerCase();
        const results = [];
        
        // Early exit after maxResults
        for (let i = 0; i < this.normalizedConversations.length && results.length < this.maxResults; i++) {
            const conv = this.normalizedConversations[i];
            const titleMatch = conv.titleLower.includes(queryLower);
            const contentMatch = !titleMatch && conv.contentLower.includes(queryLower);

            if (titleMatch || contentMatch) {
                results.push({
                    id: conv.id,
                    title: conv.title,
                    url: conv.url,
                    inserted_at: conv.inserted_at,
                    titleMatch: titleMatch,
                    content: conv.content,
                    queryLower: queryLower
                });
            }
        }

        // Sort: title matches first, then by date
        results.sort((a, b) => {
            if (a.titleMatch && !b.titleMatch) return -1;
            if (!a.titleMatch && b.titleMatch) return 1;
            return new Date(b.inserted_at) - new Date(a.inserted_at);
        });

        return results;
    }

    // Get snippet around the match (lazy - called only when needed)
    getSnippet(content, queryLower, contextLength = 80) {
        if (!content) return '';

        const contentLower = content.toLowerCase();
        const index = contentLower.indexOf(queryLower);
        
        if (index === -1) return '';

        const start = Math.max(0, index - contextLength);
        const end = Math.min(content.length, index + queryLower.length + contextLength);
        
        let snippet = content.substring(start, end);
        
        if (start > 0) snippet = '...' + snippet;
        if (end < content.length) snippet = snippet + '...';
        
        return snippet;
    }
}

// Initialize search
const chatSearch = new ChatSearch();
let searchTimeout = null;
let cachedRegex = null;
let cachedQuery = '';

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', async () => {
    await chatSearch.init();
    
    const searchInput = document.getElementById('searchInput');
    const searchResults = document.getElementById('searchResults');
    const conversationsList = document.getElementById('conversationsList');

    if (!searchInput || !searchResults || !conversationsList) return;

    // Handle search input with optimized debounce
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

        // Optimized debounce (100ms)
        searchTimeout = setTimeout(() => {
            const startTime = performance.now();
            const results = chatSearch.search(query);
            const searchTime = performance.now() - startTime;
            
            displaySearchResults(results, query);
            console.log(`⚡ Search completed in ${searchTime.toFixed(2)}ms (${results.length} results)`);
        }, 100);
    });

    // Optimized display with DocumentFragment
    function displaySearchResults(results, query) {
        if (results.length === 0) {
            searchResults.innerHTML = '<div class="search-no-results">Ничего не найдено</div>';
            searchResults.style.display = 'block';
            conversationsList.style.display = 'none';
            return;
        }

        // Cache regex for highlighting
        if (cachedQuery !== query) {
            cachedQuery = query;
            cachedRegex = new RegExp(`(${escapeRegex(query)})`, 'gi');
        }

        // Use DocumentFragment for better performance
        const fragment = document.createDocumentFragment();
        
        results.forEach(result => {
            const item = document.createElement('a');
            item.href = result.url;
            item.className = 'search-result-item';
            
            const title = document.createElement('div');
            title.className = 'search-result-title';
            title.innerHTML = highlightMatch(result.title, cachedRegex);
            item.appendChild(title);
            
            // Lazy snippet generation only if not title match
            if (!result.titleMatch) {
                const snippet = chatSearch.getSnippet(result.content, result.queryLower);
                if (snippet) {
                    const snippetDiv = document.createElement('div');
                    snippetDiv.className = 'search-result-snippet';
                    snippetDiv.innerHTML = highlightMatch(snippet, cachedRegex);
                    item.appendChild(snippetDiv);
                }
            }
            
            fragment.appendChild(item);
        });

        // Single DOM update
        searchResults.innerHTML = '';
        searchResults.appendChild(fragment);
        searchResults.style.display = 'block';
        conversationsList.style.display = 'none';
    }

    // Highlight matched text with cached regex
    function highlightMatch(text, regex) {
        if (!text) return text;
        return text.replace(regex, '<mark>$1</mark>');
    }

    // Escape special regex characters
    function escapeRegex(str) {
        return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    }

    // Optional: Add keyboard navigation (Ctrl+K to focus search)
    document.addEventListener('keydown', (e) => {
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
