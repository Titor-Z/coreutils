import DefaultTheme from 'vitepress/theme'
import Layout from './Layout.vue'
import CommandReport from './components/CommandReport.vue'
import './styles/mondrian.css'

export default {
  extends: DefaultTheme,
  Layout,
  enhanceApp({ app }) {
    app.component('CommandReport', CommandReport)
  },
}
