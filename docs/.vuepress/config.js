import { defineUserConfig } from 'vuepress'
import { defaultTheme } from '@vuepress/theme-default'
import { backToTopPlugin } from '@vuepress/plugin-back-to-top'
import { mediumZoomPlugin } from '@vuepress/plugin-medium-zoom'
import { searchPlugin } from '@vuepress/plugin-search'

export default defineUserConfig({
    lang: 'en-US',
    title: 'Rust Async Programming with Future Trait',
    description: 'A comprehensive guide to asynchronous programming in Rust using the Future trait',

    theme: defaultTheme({
        logo: '/images/logo.png',
        repo: 'yourusername/rust-async-book',
        docsDir: 'docs',

        // Theme configuration
        colorMode: 'auto',
        colorModeSwitch: true,

        // Navbar configuration
        navbar: [
            { text: 'Home', link: '/' },
            { text: 'Guide', link: '/guide/' },
            { text: 'Examples', link: '/examples/' },
            { text: 'API', link: '/api/' },
        ],

        // Sidebar configuration
        sidebar: {
            '/guide/': [
                {
                    text: 'Getting Started',
                    children: [
                        '/guide/introduction.md',
                        '/guide/why-async.md',
                        '/guide/key-concepts.md',
                    ],
                },
                {
                    text: 'Core Concepts',
                    children: [
                        '/guide/future-trait.md',
                        '/guide/async-await.md',
                        '/guide/executors.md',
                    ],
                },
                {
                    text: 'Advanced Topics',
                    children: [
                        '/guide/custom-futures.md',
                        '/guide/combinators.md',
                        '/guide/error-handling.md',
                    ],
                },
            ],
            '/examples/': [
                {
                    text: 'Basic Examples',
                    children: [
                        '/examples/basic-future.md',
                        '/examples/custom-delay.md',
                        '/examples/error-handling.md',
                    ],
                },
                {
                    text: 'Advanced Examples',
                    children: [
                        '/examples/combinators.md',
                        '/examples/autonomous-agent.md',
                        '/examples/real-world.md',
                    ],
                },
            ],
        },
    }),

    // Plugin configuration
    plugins: [
        backToTopPlugin(),
        mediumZoomPlugin({
            selector: '.content__default img',
        }),
        searchPlugin({
            locales: {
                '/': {
                    placeholder: 'Search documentation',
                },
            },
        }),
    ],

    // Markdown configuration
    markdown: {
        code: {
            lineNumbers: true,
        },
        extractHeaders: ['h2', 'h3', 'h4'],
    },

    // Build configuration
    dest: 'dist',
    temp: '.temp',
    cache: '.cache',
}) 