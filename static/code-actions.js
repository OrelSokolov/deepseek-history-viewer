// Code block actions: Copy and Download
// Wrapped in IIFE to avoid global variable conflicts
(function() {
    console.log('=== CODE ACTIONS SCRIPT LOADED ===');
    console.log('window.__TAURI__:', typeof window.__TAURI__);
    console.log('window.__TAURI_INTERNALS__:', typeof window.__TAURI_INTERNALS__);

    // Wait for Tauri to be ready
    function initCodeActions() {
        // Check if running in Tauri
        const isInTauri = window.__TAURI__ !== undefined;
        
        console.log('=== CODE ACTIONS INITIALIZED ===');
        console.log('isInTauri:', isInTauri);
        
        if (isInTauri) {
            console.log('Available Tauri modules:', Object.keys(window.__TAURI__));
            if (window.__TAURI__.clipboard) {
                console.log('Clipboard API:', Object.keys(window.__TAURI__.clipboard));
            }
            if (window.__TAURI__.dialog) {
                console.log('Dialog API:', Object.keys(window.__TAURI__.dialog));
            }
            if (window.__TAURI__.fs) {
                console.log('FS API:', Object.keys(window.__TAURI__.fs));
            }
        }
        
        // Handle Copy buttons
        document.querySelectorAll('.copy-btn').forEach(button => {
            console.log('Found copy button:', button);
            button.addEventListener('click', async function() {
                const wrapper = this.closest('.code-block-wrapper');
                const highlight = wrapper.querySelector('.highlight');
                const code = highlight.getAttribute('data-code');
                const originalText = this.innerHTML;
                
                console.log('=== COPY CLICKED ===');
                console.log('Code length:', code.length);
                console.log('isInTauri:', isInTauri);
                
                try {
                    if (isInTauri && window.__TAURI__.clipboard && window.__TAURI__.clipboard.writeText) {
                        // Use Tauri clipboard API
                        console.log('Using Tauri clipboard API');
                        await window.__TAURI__.clipboard.writeText(code);
                    } else {
                        // Use browser clipboard API
                        console.log('Using browser clipboard API');
                        await navigator.clipboard.writeText(code);
                    }
                    
                    // Visual feedback
                    this.innerHTML = '<svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M13 4L6 11L3 8" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>Copied!';
                    this.style.color = '#10b981';
                    
                    setTimeout(() => {
                        this.innerHTML = originalText;
                        this.style.color = '';
                    }, 2000);
                } catch (err) {
                    console.error('=== COPY ERROR ===');
                    console.error('Error:', err);
                    console.error('Message:', err.message);
                    console.error('Stack:', err.stack);
                    
                    this.innerHTML = '<svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M8 1V8M8 11V13" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>Error';
                    this.style.color = '#ef4444';
                    
                    setTimeout(() => {
                        this.innerHTML = originalText;
                        this.style.color = '';
                    }, 2000);
                }
            });
        });
        
        // Handle Download buttons
        document.querySelectorAll('.download-btn').forEach(button => {
            console.log('Found download button:', button);
            button.addEventListener('click', async function() {
                const wrapper = this.closest('.code-block-wrapper');
                const highlight = wrapper.querySelector('.highlight');
                const code = highlight.getAttribute('data-code');
                const lang = highlight.getAttribute('data-lang') || 'txt';
                const originalText = this.innerHTML;
                
                console.log('=== DOWNLOAD CLICKED ===');
                console.log('Lang:', lang);
                console.log('Code length:', code.length);
                console.log('isInTauri:', isInTauri);
                
                // Determine file extension
                const extensions = {
                    'python': 'py',
                    'javascript': 'js',
                    'typescript': 'ts',
                    'rust': 'rs',
                    'go': 'go',
                    'java': 'java',
                    'cpp': 'cpp',
                    'c': 'c',
                    'ruby': 'rb',
                    'php': 'php',
                    'swift': 'swift',
                    'kotlin': 'kt',
                    'elixir': 'ex',
                    'shell': 'sh',
                    'bash': 'sh',
                    'sql': 'sql',
                    'html': 'html',
                    'css': 'css',
                    'json': 'json',
                    'yaml': 'yaml',
                    'xml': 'xml',
                    'markdown': 'md',
                };
                
                const ext = extensions[lang.toLowerCase()] || 'txt';
                const filename = `code.${ext}`;
                
                console.log('Filename:', filename);
                
                try {
                    if (isInTauri && window.__TAURI__.dialog && window.__TAURI__.fs) {
                        console.log('=== USING TAURI APIs ===');
                        console.log('window.__TAURI__.dialog:', window.__TAURI__.dialog);
                        console.log('window.__TAURI__.fs:', window.__TAURI__.fs);
                        
                        // Use Tauri dialog and fs APIs
                        const dialogSave = window.__TAURI__.dialog.save;
                        const fsWriteTextFile = window.__TAURI__.fs.writeTextFile;
                        
                        console.log('save function:', dialogSave);
                        console.log('writeTextFile function:', fsWriteTextFile);
                        
                        console.log('Calling save dialog...');
                        
                        // Show save dialog
                        const filePath = await dialogSave({
                            defaultPath: filename,
                            filters: [{
                                name: 'Code File',
                                extensions: [ext]
                            }]
                        });
                        
                        console.log('Save dialog returned:', filePath);
                        
                        if (filePath) {
                            console.log('Writing file to:', filePath);
                            // Write file
                            await fsWriteTextFile(filePath, code);
                            console.log('File written successfully');
                            
                            // Visual feedback
                            this.innerHTML = '<svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M13 4L6 11L3 8" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>Saved!';
                            this.style.color = '#10b981';
                        } else {
                            console.log('User cancelled save dialog');
                        }
                    } else {
                        console.log('=== USING BROWSER DOWNLOAD ===');
                        // Use browser download
                        const blob = new Blob([code], { type: 'text/plain;charset=utf-8' });
                        const url = URL.createObjectURL(blob);
                        const a = document.createElement('a');
                        a.href = url;
                        a.download = filename;
                        document.body.appendChild(a);
                        a.click();
                        document.body.removeChild(a);
                        URL.revokeObjectURL(url);
                        
                        // Visual feedback
                        this.innerHTML = '<svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M13 4L6 11L3 8" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>Downloaded!';
                        this.style.color = '#10b981';
                    }
                    
                    setTimeout(() => {
                        this.innerHTML = originalText;
                        this.style.color = '';
                    }, 2000);
                } catch (err) {
                    console.error('=== DOWNLOAD ERROR ===');
                    console.error('Error:', err);
                    console.error('Message:', err.message);
                    console.error('Stack:', err.stack);
                    
                    this.innerHTML = '<svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M8 1V8M8 11V13" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>Error';
                    this.style.color = '#ef4444';
                    
                    setTimeout(() => {
                        this.innerHTML = originalText;
                        this.style.color = '';
                    }, 2000);
                }
            });
        });
        
        console.log('=== CODE ACTIONS SETUP COMPLETE ===');
    }

    // Initialize when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initCodeActions);
    } else {
        // DOM already loaded
        initCodeActions();
    }
})();
