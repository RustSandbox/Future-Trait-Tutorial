import { defineClientAppEnhance } from '@vuepress/client'

export default defineClientAppEnhance(({ app, router, siteData }) => {
    // Add global components
    app.component('InteractiveExample', () => import('./components/InteractiveExample.vue'))

    // Add global mixins
    app.mixin({
        mounted() {
            // Add copy button to code blocks
            this.$nextTick(() => {
                document.querySelectorAll('pre code').forEach((block) => {
                    const button = document.createElement('button')
                    button.className = 'copy-button'
                    button.textContent = 'Copy'
                    button.addEventListener('click', () => {
                        navigator.clipboard.writeText(block.textContent)
                        button.textContent = 'Copied!'
                        setTimeout(() => {
                            button.textContent = 'Copy'
                        }, 2000)
                    })
                    block.parentNode.insertBefore(button, block)
                })
            })
        }
    })

    // Add keyboard shortcuts
    router.beforeEach((to, from, next) => {
        document.addEventListener('keydown', (e) => {
            if (e.ctrlKey || e.metaKey) {
                switch (e.key) {
                    case 'f':
                        e.preventDefault()
                        document.querySelector('.search-input')?.focus()
                        break
                    case 'h':
                        e.preventDefault()
                        window.history.back()
                        break
                }
            }
        })
        next()
    })
}) 