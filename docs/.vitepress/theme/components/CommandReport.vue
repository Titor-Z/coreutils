<script setup>
import { computed, ref } from 'vue'
import raw from '../../../data/commands.json'

const CATS = [
  { key: 'microsoft', label: 'microsoft 基础', color: '#fac901', ink: '#0e0e0e' },
  { key: 'uutils', label: 'uutils 生态', color: '#225095', ink: '#ffffff' },
  { key: 'custom', label: '本项目自研', color: '#dd0100', ink: '#ffffff' },
]

const COLOR = Object.fromEntries(CATS.map((c) => [c.key, c.color]))
const LABEL = Object.fromEntries(CATS.map((c) => [c.key, c.label]))

const impl = computed(() => raw.commands.filter((c) => c.loc > 0))
const aliases = computed(() => raw.commands.filter((c) => c.loc === 0))

const stats = computed(() =>
  CATS.map((c) => {
    const items = impl.value.filter((x) => x.category === c.key)
    const loc = items.reduce((s, x) => s + x.loc, 0)
    return {
      ...c,
      implCount: items.length,
      aliasCount: aliases.value.filter((x) => x.category === c.key).length,
      totalCount: raw.commands.filter((x) => x.category === c.key).length,
      loc,
      pctLoc: (loc / raw.totalLoc) * 100,
      pctCount: (items.length / impl.value.length) * 100,
      files: items.reduce((s, x) => s + x.files, 0),
      max: items.length ? Math.max(...items.map((x) => x.loc)) : 0,
    }
  }),
)

const maxLoc = computed(() => Math.max(...impl.value.map((c) => c.loc)))

const filter = ref('all')
const query = ref('')

const rows = computed(() => {
  const q = query.value.trim().toLowerCase()
  return raw.commands
    .filter((c) => filter.value === 'all' || c.category === filter.value)
    .filter((c) => !q || c.name.toLowerCase().includes(q) || (c.source ?? '').toLowerCase().includes(q))
    .filter((c) => c.loc > 0)
    .sort((a, b) => b.loc - a.loc || a.name.localeCompare(b.name))
})

const aliasRows = computed(() => {
  const q = query.value.trim().toLowerCase()
  return aliases.value
    .filter((c) => filter.value === 'all' || c.category === filter.value)
    .filter((c) => !q || c.name.toLowerCase().includes(q))
})

const filteredLoc = computed(() => rows.value.reduce((s, c) => s + c.loc, 0))
const filteredFiles = computed(() => rows.value.reduce((s, c) => s + c.files, 0))
</script>

<template>
  <div class="r">
    <!-- headline figures -->
    <div class="r-figs">
      <div class="r-fig">
        <span class="r-fig-k">命令总数</span>
        <span class="r-fig-v">{{ raw.total }}</span>
      </div>
      <div class="r-fig">
        <span class="r-fig-k">独立实现</span>
        <span class="r-fig-v">{{ impl.length }}</span>
      </div>
      <div class="r-fig">
        <span class="r-fig-k">别名</span>
        <span class="r-fig-v">{{ aliases.length }}</span>
      </div>
      <div class="r-fig">
        <span class="r-fig-k">源码行数</span>
        <span class="r-fig-v">{{ raw.totalLoc.toLocaleString('en-US') }}</span>
      </div>
      <div class="r-fig">
        <span class="r-fig-k">可执行文件</span>
        <span class="r-fig-v">{{ (raw.binaryBytes / 1048576).toFixed(2) }}<small>MiB</small></span>
      </div>
    </div>

    <!-- category share, one stacked bar -->
    <section class="r-sec">
      <h2 class="r-h">分类占比</h2>
      <p class="r-sub">面积 = 该分类的 Rust 源码行数占全量比例。</p>

      <div class="r-stack">
        <div
          v-for="s in stats"
          :key="s.key"
          class="r-stack-cell"
          :style="{ flexGrow: s.pctLoc, background: s.color, color: s.ink }"
        >
          <span v-if="s.pctLoc > 8">{{ s.pctLoc.toFixed(1) }}%</span>
        </div>
      </div>

      <!-- per-category detail bars -->
      <div class="r-legend">
        <div v-for="s in stats" :key="s.key" class="r-legend-row">
          <span class="r-chip" :style="{ background: s.color }" />
          <span class="r-legend-name">{{ s.label }}</span>
          <span class="r-legend-bar">
            <span class="r-legend-fill" :style="{ width: s.pctLoc + '%', background: s.color }" />
          </span>
          <span class="r-legend-num">{{ s.pctLoc.toFixed(1) }}%</span>
          <span class="r-legend-meta">
            {{ s.implCount }} 实现<template v-if="s.aliasCount"> + {{ s.aliasCount }} 别名</template>
            · {{ s.loc.toLocaleString('en-US') }} 行 · {{ s.files }} 文件
          </span>
        </div>
      </div>
    </section>

    <!-- filter -->
    <section class="r-sec">
      <div class="r-controls">
        <div class="r-tabs">
          <button
            v-for="opt in [{ key: 'all', label: '全部' }, ...CATS]"
            :key="opt.key"
            class="r-tab"
            :class="{ on: filter === opt.key }"
            @click="filter = opt.key"
          >
            {{ opt.label }}
          </button>
        </div>
        <input v-model="query" class="r-search" type="search" placeholder="筛选命令名或源码路径…" />
      </div>

      <p class="r-count">
        显示 <b>{{ rows.length }}</b> 个实现，合计
        <b>{{ filteredLoc.toLocaleString('en-US') }}</b> 行 · {{ filteredFiles }} 个文件
        <span v-if="aliasRows.length"> · 另有 {{ aliasRows.length }} 个别名</span>
      </p>

      <table class="r-table">
        <thead>
          <tr>
            <th class="c-idx">#</th>
            <th>命令</th>
            <th>分类</th>
            <th class="c-num">行数</th>
            <th class="c-bar">占最大命令</th>
            <th class="c-num">文件</th>
            <th class="c-src">源码路径</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(c, i) in rows" :key="c.name">
            <td class="c-idx">{{ i + 1 }}</td>
            <td class="c-name">{{ c.name }}</td>
            <td>
              <span class="r-tag" :style="{ background: COLOR[c.category], color: c.category === 'microsoft' ? '#0e0e0e' : '#fff' }">
                {{ LABEL[c.category] }}
              </span>
            </td>
            <td class="c-num">{{ c.loc.toLocaleString('en-US') }}</td>
            <td class="c-bar">
              <span class="r-mini"><span :style="{ width: (c.loc / maxLoc) * 100 + '%', background: COLOR[c.category] }" /></span>
            </td>
            <td class="c-num">{{ c.files }}</td>
            <td class="c-src">{{ c.source }}</td>
          </tr>
        </tbody>
      </table>

      <p v-if="!rows.length" class="r-empty">没有匹配的命令。</p>

      <div v-if="aliasRows.length" class="r-alias">
        <h3>共用实现</h3>
        <p class="r-sub">下列命令与另一命令共享同一份实现，不单独计入行数。</p>
        <ul>
          <li v-for="a in aliasRows" :key="a.name">
            <code>{{ a.name }}</code> ← 同 <code>{{ a.aliasOf }}</code>
          </li>
        </ul>
      </div>
    </section>

    <p class="r-foot">
      数据由 <code>docs/scripts/gen-data.mjs</code> 从二进制与源码树自动生成，生成日期 {{ raw.generatedAt }}。
      行数为对应实现 crate 的 Rust 源码行数（含空行与注释），不等于二进制体积贡献。
    </p>
  </div>
</template>

<style scoped>
.r {
  --ink: #0e0e0e;
  --paper: #f2f0e6;
  --grey: #6f6b60;
  --mono: 'JetBrains Mono', 'Cascadia Code', 'SF Mono', Consolas, monospace;
  color: var(--ink);
}

.r-figs {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  border-top: 2px solid var(--ink);
  border-bottom: 2px solid var(--ink);
  margin-bottom: 40px;
}

.r-fig {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 14px 14px 14px 0;
}

.r-fig-k {
  font-family: var(--mono);
  font-size: 11px;
  letter-spacing: 0.06em;
  color: var(--grey);
}

.r-fig-v {
  font-family: var(--mono);
  font-size: clamp(20px, 2.2vw, 28px);
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  letter-spacing: -0.03em;
}

.r-fig-v small {
  font-size: 0.45em;
  margin-left: 3px;
  color: var(--grey);
}

.r-sec { margin-bottom: 48px; }

.r-h {
  font-size: 20px;
  font-weight: 700;
  letter-spacing: -0.02em;
  margin: 0 0 6px;
}

.r-sub {
  font-size: 13px;
  color: var(--grey);
  margin: 0 0 18px;
  line-height: 1.7;
}

/* stacked share bar */
.r-stack {
  display: flex;
  height: 54px;
  border: 2px solid var(--ink);
  margin-bottom: 22px;
}

.r-stack-cell {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding-right: 10px;
  min-width: 0;
  border-right: 2px solid var(--ink);
  font-family: var(--mono);
  font-size: 13px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  transition: flex-grow 400ms cubic-bezier(0.22, 1, 0.36, 1);
}

.r-stack-cell:last-child { border-right: 0; }

.r-legend { display: grid; gap: 10px; }

.r-legend-row {
  display: grid;
  grid-template-columns: 12px minmax(96px, 130px) minmax(80px, 1fr) 60px auto;
  align-items: center;
  gap: 10px;
  font-family: var(--mono);
  font-size: 12px;
}

.r-chip {
  width: 10px;
  height: 10px;
  outline: 1.5px solid var(--ink);
}

.r-legend-name { color: var(--ink); }

.r-legend-bar {
  height: 10px;
  background: var(--paper);
  border: 1.5px solid var(--ink);
  display: block;
  overflow: hidden;
}

.r-legend-fill {
  display: block;
  height: 100%;
  transition: width 500ms cubic-bezier(0.22, 1, 0.36, 1);
}

.r-legend-num {
  text-align: right;
  font-variant-numeric: tabular-nums;
  font-weight: 700;
}

.r-legend-meta {
  color: var(--grey);
  font-size: 11px;
  white-space: nowrap;
}

/* controls */
.r-controls {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
}

.r-tabs { display: flex; flex-wrap: wrap; gap: 6px; }

.r-tab {
  font-family: var(--mono);
  font-size: 12px;
  padding: 5px 12px;
  border: 2px solid var(--ink);
  background: transparent;
  color: var(--ink);
  cursor: pointer;
  transition: background 130ms ease, color 130ms ease;
}

.r-tab:hover { background: var(--paper); }
.r-tab.on { background: var(--ink); color: #fff; }

.r-search {
  font-family: var(--mono);
  font-size: 12px;
  padding: 6px 10px;
  border: 2px solid var(--ink);
  background: #fff;
  color: var(--ink);
  min-width: 220px;
  outline: none;
}

.r-search:focus { box-shadow: 3px 3px 0 var(--ink); }

.r-count {
  font-family: var(--mono);
  font-size: 12px;
  color: var(--grey);
  margin: 0 0 12px;
}

.r-count b { color: var(--ink); }

/* table */
.r-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12.5px;
  font-family: var(--mono);
  font-variant-numeric: tabular-nums;
}

.r-table th {
  text-align: left;
  font-size: 10.5px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--grey);
  padding: 0 10px 8px 0;
  border-bottom: 2px solid var(--ink);
  font-weight: 600;
  white-space: nowrap;
}

.r-table td {
  padding: 7px 10px 7px 0;
  border-bottom: 1px solid #e3dfd2;
  vertical-align: middle;
}

.r-table tbody tr:hover td { background: #faf8f0; }

.c-idx { width: 34px; color: var(--grey); }
.c-num { text-align: right; white-space: nowrap; }
.c-bar { width: 130px; }
.c-src {
  color: var(--grey);
  font-size: 11px;
  max-width: 260px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.c-name { font-weight: 600; }

.r-mini {
  display: block;
  height: 8px;
  background: var(--paper);
  border: 1px solid #cfcabb;
}

.r-mini > span {
  display: block;
  height: 100%;
}

.r-tag {
  font-size: 10px;
  padding: 1px 6px;
  border: 1.5px solid var(--ink);
  white-space: nowrap;
}

.r-empty {
  font-family: var(--mono);
  font-size: 12px;
  color: var(--grey);
  padding: 16px 0;
}

.r-alias { margin-top: 34px; }

.r-alias h3 {
  font-size: 15px;
  margin: 0 0 4px;
  font-weight: 700;
}

.r-alias ul {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-wrap: wrap;
  gap: 8px 22px;
  font-family: var(--mono);
  font-size: 12px;
}

.r-alias code {
  background: var(--paper);
  padding: 1px 5px;
}

.r-foot {
  font-size: 12px;
  line-height: 1.75;
  color: var(--grey);
  border-top: 2px solid var(--ink);
  padding-top: 14px;
}

.r-foot code {
  font-family: var(--mono);
  background: var(--paper);
  padding: 1px 4px;
}

@media (max-width: 720px) {
  .c-bar, .c-src { display: none; }
  .r-legend-row {
    grid-template-columns: 12px 1fr 52px;
  }
  .r-legend-bar, .r-legend-meta { display: none; }
}
</style>
