<script setup>
import { withBase } from 'vitepress'
import MondrianBar from './MondrianBar.vue'
import raw from '../../../data/commands.json'

const REPO = 'https://github.com/Titor-Z/coreutils'
</script>

<template>
  <div class="m-page m-shell">
    <MondrianBar />

    <main class="m-main">
      <div class="m-doc">
        <slot />
      </div>
    </main>

    <footer class="m-foot">
      <div>
        <span class="m-brand-mark small" aria-hidden="true">
          <i class="r" /><i class="b" /><i class="y" />
        </span>
        coreutils for Windows · MIT
      </div>
      <div class="m-foot-links">
        <a :href="REPO" target="_blank" rel="noreferrer">GitHub</a>
        <a :href="`${REPO}/releases`" target="_blank" rel="noreferrer">Releases</a>
        <a :href="withBase('/commands')">命令报表</a>
      </div>
      <div class="m-foot-meta m-mono">数据生成于 {{ raw.generatedAt }}</div>
    </footer>
  </div>
</template>

<style scoped>
.m-shell {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

.m-main {
  flex: 1;
  padding: clamp(28px, 4vw, 56px) var(--gutter) clamp(40px, 5vw, 72px);
}

.m-doc {
  max-width: 1140px;
  margin: 0 auto;
}

/* markdown content inherits the site's typography instead of VitePress defaults */
.m-doc :deep(h1) {
  font-size: clamp(28px, 3.6vw, 40px);
  line-height: 1.1;
  letter-spacing: -0.03em;
  font-weight: 700;
  margin: 0 0 10px;
}

.m-doc :deep(h2) {
  font-size: 20px;
  letter-spacing: -0.02em;
  font-weight: 700;
  margin: 40px 0 8px;
}

.m-doc :deep(h3) {
  font-size: 15px;
  font-weight: 700;
  margin: 28px 0 6px;
}

.m-doc :deep(p) {
  font-size: 14px;
  line-height: 1.75;
  color: #2b2b2b;
  margin: 0 0 12px;
  max-width: 76ch;
}

.m-doc :deep(a) {
  color: var(--m-ink);
  border-bottom: 2px solid var(--m-red);
}

.m-doc :deep(code) {
  font-family: var(--m-mono);
  font-size: 0.92em;
  background: var(--m-paper);
  padding: 1px 4px;
}

.m-foot {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 26px var(--gutter) 34px;
  border-top: 2px solid var(--m-ink);
  font-family: var(--m-mono);
  font-size: 12px;
  color: var(--m-grey);
}

.m-foot > div:first-child {
  display: flex;
  align-items: center;
  gap: 9px;
  color: var(--m-ink);
}

.m-foot-links {
  display: flex;
  gap: 18px;
}

.m-foot-links a:hover { color: var(--m-red); }

.m-brand-mark {
  display: grid;
  grid-template-columns: 5px 5px 5px;
  gap: 2px;
  height: 11px;
}

.m-brand-mark i { display: block; height: 100%; }
.m-brand-mark .r { background: var(--m-red); }
.m-brand-mark .b { background: var(--m-blue); }
.m-brand-mark .y { background: var(--m-yellow); }
</style>
