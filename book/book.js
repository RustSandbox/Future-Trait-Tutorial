// mdBook Documentation Generator
// This script enhances the mdBook with additional features and interactivity

window.addEventListener('load', function () {
    // Initialize code highlighting
    initCodeHighlighting();

    // Initialize search functionality
    initSearch();

    // Add navigation helpers
    initNavigation();

    // Add interactive examples
    initInteractiveExamples();
});

// Code highlighting with Prism.js
function initCodeHighlighting() {
    // Add copy button to code blocks
    document.querySelectorAll('pre code').forEach((block) => {
        const button = document.createElement('button');
        button.className = 'copy-button';
        button.textContent = 'Copy';
        button.addEventListener('click', () => {
            navigator.clipboard.writeText(block.textContent);
            button.textContent = 'Copied!';
            setTimeout(() => {
                button.textContent = 'Copy';
            }, 2000);
        });
        block.parentNode.insertBefore(button, block);
    });
}

// Search functionality
function initSearch() {
    const searchInput = document.createElement('input');
    searchInput.type = 'search';
    searchInput.placeholder = 'Search the book...';
    searchInput.className = 'search-input';

    const searchResults = document.createElement('div');
    searchResults.className = 'search-results';

    document.querySelector('.book-summary').prepend(searchInput);
    document.querySelector('.book-summary').appendChild(searchResults);

    searchInput.addEventListener('input', debounce((e) => {
        const query = e.target.value.toLowerCase();
        if (query.length < 2) {
            searchResults.innerHTML = '';
            return;
        }

        // Search through all content
        const results = searchContent(query);
        displaySearchResults(results, searchResults);
    }, 300));
}

// Navigation helpers
function initNavigation() {
    // Add "Back to Top" button
    const backToTop = document.createElement('button');
    backToTop.className = 'back-to-top';
    backToTop.textContent = '↑';
    backToTop.addEventListener('click', () => {
        window.scrollTo({ top: 0, behavior: 'smooth' });
    });
    document.body.appendChild(backToTop);

    // Add keyboard navigation
    document.addEventListener('keydown', (e) => {
        if (e.ctrlKey || e.metaKey) {
            switch (e.key) {
                case 'f':
                    e.preventDefault();
                    document.querySelector('.search-input').focus();
                    break;
                case 'h':
                    e.preventDefault();
                    window.history.back();
                    break;
            }
        }
    });
}

// Interactive examples
function initInteractiveExamples() {
    document.querySelectorAll('.interactive-example').forEach((example) => {
        const runButton = document.createElement('button');
        runButton.className = 'run-example';
        runButton.textContent = 'Run Example';

        const output = document.createElement('pre');
        output.className = 'example-output';

        runButton.addEventListener('click', async () => {
            output.textContent = 'Running...';
            try {
                const result = await runExample(example.dataset.example);
                output.textContent = result;
            } catch (error) {
                output.textContent = `Error: ${error.message}`;
            }
        });

        example.appendChild(runButton);
        example.appendChild(output);
    });
}

// Utility functions
function debounce(func, wait) {
    let timeout;
    return function executedFunction(...args) {
        const later = () => {
            clearTimeout(timeout);
            func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
}

function searchContent(query) {
    const results = [];
    document.querySelectorAll('.chapter').forEach((chapter) => {
        const text = chapter.textContent.toLowerCase();
        if (text.includes(query)) {
            results.push({
                title: chapter.querySelector('h1').textContent,
                content: text,
                url: chapter.dataset.url
            });
        }
    });
    return results;
}

function displaySearchResults(results, container) {
    container.innerHTML = results.map(result => `
        <div class="search-result">
            <a href="${result.url}">${result.title}</a>
            <p>${result.content.substring(0, 150)}...</p>
        </div>
    `).join('');
}

async function runExample(exampleId) {
    // This would be replaced with actual example execution logic
    return new Promise((resolve) => {
        setTimeout(() => {
            resolve('Example output would appear here');
        }, 1000);
    });
}

// Add custom styles
const style = document.createElement('style');
style.textContent = `
    .copy-button {
        position: absolute;
        top: 0.5rem;
        right: 0.5rem;
        padding: 0.25rem 0.5rem;
        background: #f0f0f0;
        border: 1px solid #ccc;
        border-radius: 3px;
        cursor: pointer;
    }
    
    .search-input {
        width: 100%;
        padding: 0.5rem;
        margin: 1rem 0;
        border: 1px solid #ccc;
        border-radius: 3px;
    }
    
    .search-results {
        position: absolute;
        top: 100%;
        left: 0;
        right: 0;
        background: white;
        border: 1px solid #ccc;
        border-radius: 3px;
        max-height: 300px;
        overflow-y: auto;
        z-index: 1000;
    }
    
    .back-to-top {
        position: fixed;
        bottom: 2rem;
        right: 2rem;
        width: 3rem;
        height: 3rem;
        border-radius: 50%;
        background: #4a90e2;
        color: white;
        border: none;
        cursor: pointer;
        font-size: 1.5rem;
        box-shadow: 0 2px 5px rgba(0,0,0,0.2);
    }
    
    .interactive-example {
        border: 1px solid #ccc;
        padding: 1rem;
        margin: 1rem 0;
        border-radius: 3px;
    }
    
    .run-example {
        padding: 0.5rem 1rem;
        background: #4a90e2;
        color: white;
        border: none;
        border-radius: 3px;
        cursor: pointer;
    }
    
    .example-output {
        margin-top: 1rem;
        padding: 1rem;
        background: #f5f5f5;
        border-radius: 3px;
    }
`;

document.head.appendChild(style); 