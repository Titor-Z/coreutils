<script setup>
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { withBase } from 'vitepress'
import raw from '../../../data/commands.json'
import MondrianBar from './MondrianBar.vue'
import MondrianTreemap from './MondrianTreemap.vue'

const REPO = 'https://github.com/Titor-Z/coreutils'

const share = computed(() => {
  const byCat = (cat) => raw.commands.filter((c) => c.category === cat).reduce((s, c) => s + c.loc, 0)
  const cats = [
    { key: 'microsoft', label: 'microsoft 基础' },
    { key: 'uutils', label: 'uutils 生态' },
    { key: 'custom', label: '本项目自研' },
  ]
  return cats.map((c) => ({ ...c, loc: byCat(c.key), pct: (byCat(c.key) / raw.totalLoc) * 100 }))
})

const miB = (raw.binaryBytes / 1048576).toFixed(2)

const groupCount = computed(() => (key) => raw.counts[key] ?? 0)
const groupNames = computed(() => (key) =>
  raw.commands
    .filter((c) => c.category === key)
    .map((c) => c.name)
    .join('、'),
)

const CMD = 'coreutils.exe free -h'
const OUT = [
  '              total        used        free      shared  buff/cache   available',
  'Mem:           3.9GiB      3.0GiB    931.9MiB          0B          0B    931.9MiB',
  'Swap:          6.2GiB      1.5GiB      4.7GiB          0B          0B      4.7GiB',
]

const typed = ref('')
const shown = ref(0)
let timers = []

onMounted(() => {
  const reduce = window.matchMedia?.('(prefers-reduced-motion: reduce)').matches
  if (reduce) {
    typed.value = CMD
    shown.value = OUT.length
    return
  }
  let i = 0
  const tick = () => {
    typed.value = CMD.slice(0, ++i)
    if (i < CMD.length) timers.push(setTimeout(tick, 52 + Math.random() * 46))
    else OUT.forEach((_, k) => timers.push(setTimeout(() => (shown.value = k + 1), 280 + k * 170)))
  }
  timers.push(setTimeout(tick, 600))
})

onUnmounted(() => timers.forEach(clearTimeout))

const gaps = [
  { item: 'awk 的 split / gsub / sprintf / for-in', state: 'panic', note: '上游 todo!() 未实现' },
  { item: 'diff3 / sdiff', state: 'missing', note: '上游零实现' },
  { item: 'egrep / fgrep', state: 'missing', note: '零实现，可用 grep -E/-F 替代' },
  { item: 'procps 的 ps / pidof', state: 'panic', note: '依赖 /proc，Windows 不可用' },
]
</script>

<template>
  <div class="m-page">
    <MondrianBar />

    <section id="top" class="m-hero">
      <div class="m-hero-copy">
        <p class="m-kicker">{{ raw.total }} commands · one binary · no WSL</p>
        <h1 class="m-title">
          把 Unix 的<br />
          <span class="m-title-accent">命令集</span><br />
          装进 Windows
        </h1>
        <p class="m-lede">
          microsoft/coreutils 打底，uutils 的 findutils、grep、sed、diffutils、tar 补齐，
          再加几个只有 Windows 才需要的小工具 —— 合成一个原生 EXE，不依赖 MSYS2、Cygwin 或 WSL。
        </p>

        <div class="m-term" aria-hidden="true">
          <div class="m-term-head">
            <span class="m-dot r" /><span class="m-dot y" /><span class="m-dot g" />
            <span class="m-term-path">PowerShell</span>
          </div>
          <pre class="m-term-body"><span class="m-prompt">PS&gt;</span> {{ typed }}<span class="m-caret" /><template v-for="(line, i) in OUT" :key="i"><br /><span v-if="i < shown" class="m-out">{{ line }}</span></template></pre>
        </div>

        <div class="m-cta">
          <a class="m-btn m-btn-primary" :href="`${REPO}/releases`" target="_blank" rel="noreferrer">
            下载安装器
          </a>
          <a class="m-btn m-btn-ghost" :href="REPO" target="_blank" rel="noreferrer">查看源码 ↗</a>
        </div>
      </div>

      <div class="m-hero-art">
        <MondrianTreemap />
      </div>
    </section>

    <section id="composition" class="m-band">
      <div class="m-band-head">
        <p class="m-kicker">composition</p>
        <h2 class="m-h2">这个二进制由什么构成</h2>
      </div>

      <div class="m-split" role="img" aria-label="源码行数按来源占比">
        <div
          v-for="s in share"
          :key="s.key"
          class="m-split-cell"
          :class="s.key"
          :style="{ flexGrow: s.pct }"
        >
          <span class="m-split-pct">{{ s.pct.toFixed(1) }}%</span>
        </div>
      </div>

      <dl class="m-figures">
        <div class="m-figure">
          <dt>命令</dt>
          <dd>{{ raw.total }}</dd>
        </div>
        <div class="m-figure">
          <dt>Rust 源码</dt>
          <dd>{{ raw.totalLoc.toLocaleString('en-US') }}<small>行</small></dd>
        </div>
        <div class="m-figure">
          <dt>单一可执行文件</dt>
          <dd>{{ miB }}<small>MiB</small></dd>
        </div>
        <div v-for="s in share" :key="s.key" class="m-figure m-figure-cat">
          <dt><span class="m-swatch" :class="s.key" />{{ s.label }}</dt>
          <dd>{{ s.loc.toLocaleString('en-US') }}<small>行</small></dd>
        </div>
      </dl>

      <p class="m-note">
        面积与占比均取自各命令对应实现 crate 的 Rust 源码行数（<code>docs/scripts/gen-data.mjs</code>
        自动生成，随源码变动）。行数不等于二进制体积贡献 —— 后者还受泛型实例化、LTO 与链接期裁剪影响。
      </p>
    </section>

    <section id="install" class="m-band m-band-alt">
      <div class="m-band-head">
        <p class="m-kicker">install</p>
        <h2 class="m-h2">三步装完</h2>
      </div>
      <ol class="m-steps">
        <li>
          <span class="m-step-no">01</span>
          <div>
            <h3>下载</h3>
            <p>从 Releases 取 <code>coreutils-Titor-Z.exe</code>，Inno Setup 打包，无需预装运行时。</p>
          </div>
        </li>
        <li>
          <span class="m-step-no">02</span>
          <div>
            <h3>安装</h3>
            <p>运行安装器，自动写入 PATH，并为核心命令建立硬链接 —— 直接敲 <code>sed</code> 也能用。</p>
          </div>
        </li>
        <li>
          <span class="m-step-no">03</span>
          <div>
            <h3>验证</h3>
            <p>新开一个终端，<code>coreutils.exe --list</code> 会列出全部 {{ raw.total }} 个命令。</p>
          </div>
        </li>
      </ol>
    </section>

    <section id="sources" class="m-band">
      <div class="m-band-head">
        <p class="m-kicker">sources</p>
        <h2 class="m-h2">三个来源，一个入口</h2>
      </div>
      <div class="m-cards">
        <article class="m-card">
          <div class="m-card-bar y" />
          <h3>microsoft/coreutils</h3>
          <p class="m-card-num">{{ groupCount('microsoft') }} <small>命令</small></p>
          <p>上游把 Rust 版 coreutils 移植到 Windows 的成果，是整个二进制的地基。</p>
          <a :href="`${REPO}/tree/main/deps/coreutils`" target="_blank" rel="noreferrer">源码 →</a>
        </article>
        <article class="m-card">
          <div class="m-card-bar b" />
          <h3>uutils 生态</h3>
          <p class="m-card-num">{{ groupCount('uutils') }} <small>命令</small></p>
          <p>findutils（find、xargs）、grep、sed、diffutils（diff、cmp）、tar，按统一的多调用协议接入。</p>
          <a href="#gaps">接入细节 →</a>
        </article>
        <article class="m-card">
          <div class="m-card-bar r" />
          <h3>本项目自研</h3>
          <p class="m-card-num">{{ groupCount('custom') }} <small>命令</small></p>
          <p>{{ groupNames('custom') }} —— 只做 Windows 上缺的那部分。</p>
          <a :href="`${REPO}/tree/main/src`" target="_blank" rel="noreferrer">源码 →</a>
        </article>
      </div>
    </section>

    <section id="gaps" class="m-band m-band-alt">
      <div class="m-band-head">
        <p class="m-kicker">known gaps</p>
        <h2 class="m-h2">我们没装作它有</h2>
        <p class="m-sub">
          下列能力在整合上游时被逐个实测过，结论是不接入、不包装 —— 与其提供一个会崩的命令，
          不如让它明确报错。
        </p>
      </div>
      <table class="m-table">
        <thead>
          <tr><th>能力</th><th>实测结论</th><th>原因</th></tr>
        </thead>
        <tbody>
          <tr v-for="g in gaps" :key="g.item">
            <td class="m-table-item">{{ g.item }}</td>
            <td><span class="m-tag" :class="g.state">{{ g.state === 'panic' ? '运行期崩溃' : '未实现' }}</span></td>
            <td class="m-table-note">{{ g.note }}</td>
          </tr>
        </tbody>
      </table>
    </section>

    <footer class="m-foot">
      <div>
        <span class="m-brand-mark small" aria-hidden="true"><i class="r" /><i class="b" /><i class="y" /></span>
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
.m-brand-mark.small {
  height: 11px;
  grid-template-columns: 5px 5px 5px;
}

.m-brand-mark .r { background: var(--m-red); }
.m-brand-mark .b { background: var(--m-blue); }
.m-brand-mark .y { background: var(--m-yellow); }

/* ---------- hero ---------- */

.m-hero {
  display: grid;
  grid-template-columns: minmax(0, 0.92fr) minmax(0, 1.08fr);
  gap: clamp(28px, 4vw, 64px);
  padding: clamp(36px, 6vw, 84px) var(--gutter) clamp(30px, 4vw, 56px);
  align-items: start;
}

.m-title {
  font-size: clamp(40px, 6.4vw, 86px);
  line-height: 0.98;
  letter-spacing: -0.035em;
  font-weight: 700;
  margin: 14px 0 0;
}

.m-title-accent {
  color: var(--m-red);
}

.m-lede {
  margin-top: 22px;
  max-width: 46ch;
  font-size: clamp(14px, 1.15vw, 16px);
  line-height: 1.72;
  color: #2b2b2b;
}

.m-term {
  margin-top: 28px;
  border: 2px solid var(--m-ink);
  background: var(--m-ink);
  color: #f4f2ea;
}

.m-term-head {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 10px;
  border-bottom: 1px solid #2a2a2a;
}

.m-dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  display: block;
}
.m-dot.r { background: var(--m-red); }
.m-dot.y { background: var(--m-yellow); }
.m-dot.g { background: var(--m-blue); }

.m-term-path {
  margin-left: 8px;
  font-family: var(--m-mono);
  font-size: 10.5px;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: #6f6f6f;
}

.m-term-body {
  margin: 0;
  padding: 12px 14px 14px;
  font-family: var(--m-mono);
  font-size: clamp(9.5px, 0.92vw, 11.5px);
  line-height: 1.62;
  white-space: pre;
  overflow-x: auto;
}

.m-prompt { color: var(--m-yellow); }

.m-out { color: #a9b6c6; }

.m-caret {
  display: inline-block;
  width: 7px;
  height: 1em;
  background: var(--m-yellow);
  vertical-align: -0.14em;
  margin-left: 2px;
  animation: caret 1.05s steps(1) infinite;
}

@keyframes caret {
  0%, 49% { opacity: 1; }
  50%, 100% { opacity: 0; }
}

.m-cta {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-top: 26px;
}

.m-btn {
  display: inline-flex;
  align-items: center;
  height: 44px;
  padding: 0 22px;
  border: 2px solid var(--m-ink);
  font-family: var(--m-mono);
  font-size: 13px;
  font-weight: 600;
  transition: transform 140ms ease, background 140ms ease, color 140ms ease;
}

.m-btn-primary {
  background: var(--m-red);
  color: var(--m-white);
}

.m-btn-primary:hover {
  transform: translate(-2px, -2px);
  box-shadow: 4px 4px 0 var(--m-ink);
}

.m-btn-ghost:hover {
  background: var(--m-ink);
  color: var(--m-white);
}

/* ---------- bands ---------- */

.m-band {
  padding: clamp(38px, 5.5vw, 76px) var(--gutter);
  border-top: 2px solid var(--m-ink);
}

.m-band-alt {
  background: var(--m-paper);
}

.m-band-head {
  max-width: 62ch;
  margin-bottom: clamp(22px, 3vw, 40px);
}

.m-h2 {
  font-size: clamp(26px, 3.2vw, 42px);
  line-height: 1.1;
  letter-spacing: -0.03em;
  font-weight: 700;
  margin-top: 8px;
}

.m-sub {
  margin-top: 14px;
  font-size: 14px;
  line-height: 1.7;
  color: #3a3a3a;
}

/* the proportional split bar: same rule, same colours, but 1-D */
.m-split {
  display: flex;
  height: 58px;
  border: 2px solid var(--m-ink);
}

.m-split-cell {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding-right: 10px;
  min-width: 0;
  border-right: 2px solid var(--m-ink);
  animation: grow-in 900ms cubic-bezier(0.22, 1, 0.36, 1) both;
}

.m-split-cell:last-child { border-right: 0; }

.m-split-cell.microsoft { background: var(--m-yellow); }
.m-split-cell.uutils { background: var(--m-blue); color: var(--m-white); }
.m-split-cell.custom { background: var(--m-red); color: var(--m-white); }

.m-split-pct {
  font-family: var(--m-mono);
  font-size: clamp(11px, 1.4vw, 16px);
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

@keyframes grow-in {
  from { opacity: 0; transform: scaleX(0.6); transform-origin: left; }
  to { opacity: 1; transform: none; }
}

.m-figures {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 0;
  margin: 34px 0 0;
  border-top: 2px solid var(--m-ink);
}

.m-figure {
  padding: 16px 0 14px;
  border-bottom: 2px solid var(--m-ink);
}

.m-figure dt {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 11.5px;
  letter-spacing: 0.06em;
  color: var(--m-grey);
  font-family: var(--m-mono);
}

.m-figure dd {
  margin: 6px 0 0;
  font-family: var(--m-mono);
  font-size: clamp(24px, 2.6vw, 34px);
  font-weight: 700;
  letter-spacing: -0.03em;
  font-variant-numeric: tabular-nums;
}

.m-figure dd small {
  font-size: 0.42em;
  font-weight: 500;
  margin-left: 5px;
  color: var(--m-grey);
}

.m-swatch {
  width: 9px;
  height: 9px;
  display: inline-block;
  outline: 1.5px solid var(--m-ink);
}

.m-swatch.microsoft { background: var(--m-yellow); }
.m-swatch.uutils { background: var(--m-blue); }
.m-swatch.custom { background: var(--m-red); }

.m-note {
  margin-top: 20px;
  max-width: 78ch;
  font-size: 12.5px;
  line-height: 1.75;
  color: var(--m-grey);
}

.m-note code {
  font-family: var(--m-mono);
  font-size: 0.94em;
  background: var(--m-paper);
  padding: 1px 4px;
}

/* ---------- steps ---------- */

.m-steps {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(230px, 1fr));
  gap: 0;
  border-top: 2px solid var(--m-ink);
}

.m-steps li {
  display: flex;
  gap: 14px;
  padding: 22px 22px 22px 0;
  border-bottom: 2px solid var(--m-ink);
}

.m-step-no {
  font-family: var(--m-mono);
  font-size: 12px;
  font-weight: 700;
  padding-top: 3px;
  color: var(--m-red);
}

.m-steps h3 {
  font-size: 15px;
  margin-bottom: 7px;
}

.m-steps p {
  font-size: 13px;
  line-height: 1.68;
  color: #3a3a3a;
}

.m-steps code,
.m-card code {
  font-family: var(--m-mono);
  font-size: 0.92em;
  background: #e6e3d8;
  padding: 1px 4px;
}

/* ---------- cards ---------- */

.m-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: clamp(14px, 1.8vw, 22px);
}

.m-card {
  border: 2px solid var(--m-ink);
  padding: 0 20px 20px;
  background: var(--m-white);
  transition: transform 160ms ease, box-shadow 160ms ease;
}

.m-card:hover {
  transform: translate(-3px, -3px);
  box-shadow: 6px 6px 0 var(--m-ink);
}

.m-card-bar {
  height: 12px;
  margin: 0 -20px 18px;
  border-bottom: 2px solid var(--m-ink);
}

.m-card-bar.y { background: var(--m-yellow); }
.m-card-bar.b { background: var(--m-blue); }
.m-card-bar.r { background: var(--m-red); }

.m-card h3 {
  font-size: 14px;
  font-family: var(--m-mono);
  letter-spacing: -0.01em;
}

.m-card-num {
  font-family: var(--m-mono);
  font-size: 34px;
  font-weight: 700;
  line-height: 1;
  margin: 12px 0 12px;
  letter-spacing: -0.04em;
}

.m-card-num small {
  font-size: 12px;
  font-weight: 500;
  color: var(--m-grey);
  margin-left: 4px;
}

.m-card p {
  font-size: 13px;
  line-height: 1.68;
  color: #3a3a3a;
}

.m-card a {
  display: inline-block;
  margin-top: 14px;
  font-family: var(--m-mono);
  font-size: 12px;
  border-bottom: 2px solid var(--m-red);
}

/* ---------- table ---------- */

.m-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.m-table th {
  text-align: left;
  font-family: var(--m-mono);
  font-size: 11px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--m-grey);
  padding: 0 14px 9px 0;
  border-bottom: 2px solid var(--m-ink);
}

.m-table td {
  padding: 12px 14px 12px 0;
  border-bottom: 1px solid #cfcabb;
  vertical-align: top;
}

.m-table-item {
  font-family: var(--m-mono);
  font-size: 12.5px;
}

.m-table-note {
  color: var(--m-grey);
}

.m-tag {
  display: inline-block;
  font-family: var(--m-mono);
  font-size: 10.5px;
  padding: 2px 7px;
  border: 1.5px solid var(--m-ink);
  white-space: nowrap;
}

.m-tag.panic {
  background: var(--m-red);
  color: var(--m-white);
}

.m-tag.missing {
  background: var(--m-white);
}

/* ---------- footer ---------- */

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

@media (max-width: 900px) {
  .m-hero {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
