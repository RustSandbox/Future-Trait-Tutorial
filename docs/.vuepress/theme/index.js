import { defaultTheme } from '@vuepress/theme-default'
import InteractiveExample from './components/InteractiveExample.vue'

export default {
    name: 'vuepress-theme-rust-async',
    extends: defaultTheme,
    components: {
        InteractiveExample
    },

    // Add custom styles
    clientAppEnhanceFiles: [
        './enhanceApp.js'
    ],

    // Add custom layouts
    layouts: {
        Layout: defaultTheme.layouts.Layout,
        Home: defaultTheme.layouts.Home,
        Page: defaultTheme.layouts.Page,
        404: defaultTheme.layouts.NotFound
    }
} 