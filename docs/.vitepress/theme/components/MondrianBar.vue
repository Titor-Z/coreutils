<script setup>
import { computed } from 'vue'
import { useData, withBase } from 'vitepress'

const REPO = 'https://github.com/Titor-Z/coreutils'

const { page } = useData()

// 首页的导航项是章节锚点，从其它页面过来必须带上 `/`。
const NAV = [
  { label: '概览', path: '/', hash: '#top', match: (p) => p === '' || p === 'index.md' },
  { label: '报表', path: '/commands', hash: '', match: (p) => p.startsWith('commands') },
  { label: '构成', path: '/', hash: '#composition' },
  { label: '安装', path: '/', hash: '#install' },
  { label: '来源', path: '/', hash: '#sources' },
  { label: '已知缺项', path: '/', hash: '#gaps' },
]

const current = computed(() => page.value.relativePath)

const items = computed(() =>
  NAV.map((n) => ({
    label: n.label,
    href: withBase(n.path) + n.hash,
    active: n.match ? n.match(current.value) : false,
  })),
)
</script>

<template>
  <header class="m-bar">
    <a class="m-brand" :href="withBase('/')">
      <span class="m-brand-mark" aria-hidden="true">
        <i class="r" /><i class="b" /><i class="y" />
      </span>
      <span class="m-brand-name">coreutils<em>.exe</em></span>
    </a>
    <nav class="m-nav">
      <a
        v-for="n in items"
        :key="n.label"
        :href="n.href"
        :class="{ on: n.active }"
        :aria-current="n.active ? 'page' : undefined"
      >
        {{ n.label }}
      </a>
      <a :href="REPO" target="_blank" rel="noreferrer">GitHub ↗</a>
    </nav>
  </header>
</template>

<style scoped>
.m-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  padding: 0 var(--gutter);
  height: 60px;
  border-bottom: 2px solid var(--m-ink);
  position: sticky;
  top: 0;
  background: var(--m-white);
  z-index: 30;
}

.m-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  font-family: var(--m-mono);
  font-weight: 700;
  letter-spacing: -0.02em;
}

.m-brand-name em {
  font-style: normal;
  color: var(--m-grey);
  font-weight: 500;
}

.m-brand-mark {
  display: grid;
  grid-template-columns: 6px 6px 6px;
  gap: 2px;
  height: 14px;
}

.m-brand-mark i {
  display: block;
  height: 100%;
}

.m-brand-mark .r { background: var(--m-red); }
.m-brand-mark .b { background: var(--m-blue); }
.m-brand-mark .y { background: var(--m-yellow); }

.m-nav {
  display: flex;
  gap: clamp(12px, 2vw, 30px);
  font-size: 13px;
  font-family: var(--m-mono);
  color: var(--m-ink);
}

.m-nav a:hover { color: var(--m-red); }

.m-nav a.on {
  color: var(--m-red);
  box-shadow: inset 0 -2px 0 var(--m-red);
}

@media (max-width: 900px) {
  .m-nav a:not(:last-child) {
    display: none;
  }
}
</style>
