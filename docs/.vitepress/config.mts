import { defineConfig } from 'vitepress'

export default defineConfig({
  lang: 'zh-CN',
  title: 'coreutils · Windows',
  description:
    '把 microsoft/coreutils、uutils/findutils、grep、sed、diffutils、tar 整合成一套原生 Windows 命令集',
  base: process.env.DOCS_BASE ?? '/coreutils/',
  cleanUrls: true,
  head: [
    ['meta', { name: 'theme-color', content: '#DD0100' }],
    ['link', { rel: 'icon', href: '/coreutils/favicon.svg' }],
  ],
  themeConfig: {
    nav: [
      { text: '概览', link: '/#composition' },
      { text: '报表', link: '/commands' },
      { text: '安装', link: '/#install' },
      { text: '来源', link: '/#sources' },
      { text: 'GitHub', link: 'https://github.com/Titor-Z/coreutils' },
    ],
    socialLinks: [{ icon: 'github', link: 'https://github.com/Titor-Z/coreutils' }],
    outline: { label: '本页目录' },
    docFooter: { prev: '上一页', next: '下一页' },
  },
})
