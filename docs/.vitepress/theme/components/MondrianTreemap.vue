<script setup>
import { computed, ref } from 'vue'
import raw from '../../../data/commands.json'

const W = 1000
const H = 560

const COLORS = {
  uutils: { fill: '#225095', ink: '#ffffff' },
  custom: { fill: '#dd0100', ink: '#ffffff' },
}

// Hover inverts to the off-white plane colour: strong contrast on both blue and
// red, and it carries no provenance meaning of its own.
const HOVER = { fill: '#f2f0e6', ink: '#0e0e0e' }

const LABELS = {
  uutils: 'uutils 生态',
  custom: '本项目自研',
}

const sum = (a) => a.reduce((x, y) => x + y, 0)

function worstRatio(row, side) {
  const s = sum(row)
  if (!s || !side) return Infinity
  const mx = Math.max(...row)
  const mn = Math.min(...row)
  return Math.max((side * side * mx) / (s * s), (s * s) / (side * side * mn))
}

// Squarified treemap (Bruls, Huizing & van Wijk, 2000).
function squarify(values, rect) {
  const out = []
  let rest = values.slice()
  let { x, y, w, h } = rect
  while (rest.length) {
    const side = Math.min(w, h)
    const row = [rest[0]]
    let k = 1
    while (k < rest.length && worstRatio([...row, rest[k]], side) <= worstRatio(row, side)) {
      row.push(rest[k])
      k++
    }
    const rowSum = sum(row)
    if (w >= h) {
      const colW = rowSum / h
      let cy = y
      for (const a of row) {
        const ch = a / colW
        out.push({ x, y: cy, w: colW, h: ch })
        cy += ch
      }
      x += colW
      w -= colW
    } else {
      const rowH = rowSum / w
      let cx = x
      for (const a of row) {
        const cw = a / rowH
        out.push({ x: cx, y, w: cw, h: rowH })
        cx += cw
      }
      y += rowH
      h -= rowH
    }
    rest = rest.slice(row.length)
  }
  return out
}

// Two-level treemap: category planes first, then commands inside each plane.
// Planes are stacked as horizontal bands (largest on top) so the small
// categories still get enough width for labels.
const layout = computed(() => {
  const added = raw.commands.filter((c) => c.category !== 'microsoft' && c.loc > 0)
  const addedLoc = sum(added.map((c) => c.loc))

  const groups = ['uutils', 'custom']
    .map((key) => {
      const cmds = added.filter((c) => c.category === key).sort((a, b) => b.loc - a.loc)
      return { key, loc: sum(cmds.map((c) => c.loc)), cmds }
    })
    .sort((a, b) => b.loc - a.loc)

  const out = []
  const parents = []
  let bandY = 0

  groups.forEach((g) => {
    const band = { x: 0, y: bandY, w: W, h: (g.loc / addedLoc) * H }
    bandY += band.h

    const inset = 5
    const box = {
      x: band.x + inset,
      y: band.y + inset,
      w: band.w - inset * 2,
      h: band.h - inset * 2,
    }
    const free = box.w * box.h
    const cells = squarify(
      g.cmds.map((c) => (c.loc / g.loc) * free),
      box,
    )

    g.cmds.forEach((c, ci) => {
      out.push({
        label: c.name,
        loc: c.loc,
        files: c.files,
        category: g.key,
        pct: (c.loc / addedLoc) * 100,
        ...cells[ci],
        idx: out.length,
      })
    })

    parents.push({ ...band, key: g.key, loc: g.loc, pct: (g.loc / addedLoc) * 100, count: g.cmds.length })
  })

  return {
    tiles: out,
    parents,
    addedLoc,
    addedCount: added.length,
    aliases: raw.commands.filter((c) => c.category !== 'microsoft' && c.loc === 0),
  }
})

// Store the label, not the tile object: ref() deep-wraps objects in a reactive
// proxy, so `active === t` could never be true for the v-for item.
const activeKey = ref(null)

const fontSize = (t) => Math.round(Math.max(11, Math.min(34, t.w / 7.5, t.h / 3)))

// Narrow, tall cells get their label set vertically so `tar` is not a blank tile.
const isRotated = (t) => t.h >= t.w * 1.6 && t.h > 90 && t.w > 40
const rotatedFont = (t) => Math.round(Math.max(11, Math.min(20, t.w / 2.6, t.h / 7)))

const caption = computed(() => {
  const t = activeKey.value ? layout.value.tiles.find((x) => x.label === activeKey.value) : null
  if (!t) {
    return {
      title: '面积 = 源码行数',
      body: `在 microsoft 原版 76 个命令之上新增 ${layout.value.addedCount} 个实现，合计 ${layout.value.addedLoc.toLocaleString('en-US')} 行 Rust。悬停任意色块查看明细。`,
    }
  }
  return {
    title: `${t.label} · ${LABELS[t.category]}`,
    body: `${t.loc.toLocaleString('en-US')} 行 · ${t.files} 个文件 · 占新增代码 ${t.pct.toFixed(1)}%`,
  }
})
</script>

<template>
  <figure class="m-map">
    <svg
      :viewBox="`0 0 ${W} ${H}`"
      class="m-map-svg"
      role="img"
      aria-label="新增命令的源码行数占比矩形树图，蓝色为 uutils 生态，红色为本项目自研"
      @mouseleave="activeKey = null"
    >
      <g
        v-for="t in layout.tiles"
        :key="t.label"
        class="m-tile"
        :class="{ 'is-active': activeKey === t.label }"
        :style="{ animationDelay: `${t.idx * 52}ms` }"
        @mouseenter="activeKey = t.label"
      >
        <rect
          :x="t.x + 3"
          :y="t.y + 3"
          :width="Math.max(0, t.w - 6)"
          :height="Math.max(0, t.h - 6)"
          :fill="activeKey === t.label ? HOVER.fill : COLORS[t.category].fill"
        />
        <g v-if="!isRotated(t) && t.w > 104 && t.h > 46">
          <text
            :x="t.x + 16"
            :y="t.y + 14 + fontSize(t)"
            :fill="activeKey === t.label ? HOVER.ink : COLORS[t.category].ink"
            :font-size="fontSize(t)"
            class="m-tile-label"
          >
            {{ t.label }}
          </text>
          <text
            v-if="t.h > 84"
            :x="t.x + 16"
            :y="t.y + 20 + fontSize(t) * 1.9"
            :fill="activeKey === t.label ? HOVER.ink : COLORS[t.category].ink"
            :font-size="Math.max(10, fontSize(t) * 0.4)"
            class="m-tile-meta"
          >
            {{ t.pct.toFixed(1) }}%
          </text>
        </g>
        <text
          v-else-if="isRotated(t)"
          :x="t.x + t.w - 18"
          :y="t.y + t.h - 18"
          :fill="activeKey === t.label ? HOVER.ink : COLORS[t.category].ink"
          :font-size="rotatedFont(t)"
          :transform="`rotate(-90 ${t.x + t.w - 18} ${t.y + t.h - 18})`"
          class="m-tile-label"
        >
          {{ t.label }}
        </text>
      </g>

      <rect
        v-for="p in layout.parents"
        :key="`p-${p.key}`"
        :x="p.x + 2"
        :y="p.y + 2"
        :width="Math.max(0, p.w - 4)"
        :height="Math.max(0, p.h - 4)"
        class="m-parent"
      />
    </svg>

    <figcaption class="m-map-caption">
      <div class="m-map-caption-title">{{ caption.title }}</div>
      <div class="m-map-caption-body">{{ caption.body }}</div>
      <ul class="m-map-legend">
        <li v-for="p in layout.parents" :key="p.key">
          <span class="m-swatch" :style="{ background: COLORS[p.key].fill }" />
          {{ LABELS[p.key] }} · {{ p.count }} 个
        </li>
        <li v-if="layout.aliases.length" class="m-alias">
          另 {{ layout.aliases.map((a) => a.name).join('、') }} 为别名，不单独计行数
        </li>
      </ul>
    </figcaption>
  </figure>
</template>

<style scoped>
.m-map {
  margin: 0;
  width: 100%;
}

.m-map-svg {
  display: block;
  width: 100%;
  height: auto;
  background: var(--m-ink);
}

.m-tile {
  transform-box: fill-box;
  transform-origin: center;
  animation: tile-in 620ms cubic-bezier(0.22, 1, 0.36, 1) both;
  cursor: pointer;
}

/* entrance lives on the <g>, hover effects on the <rect> — an animation with
   fill-mode both would otherwise pin transform back to its end state forever */
.m-tile rect {
  transform-box: fill-box;
  transform-origin: center;
  transition:
    fill 150ms ease,
    transform 150ms ease;
}

.m-tile.is-active rect {
  transform: scale(0.955);
}

.m-parent {
  fill: none;
  stroke: var(--m-ink);
  stroke-width: 9;
  pointer-events: none;
  animation: fade-in 700ms ease 480ms both;
}

@keyframes tile-in {
  from {
    opacity: 0;
    transform: translateY(20px) scale(0.93);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

@keyframes fade-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

.m-tile-label {
  font-family: var(--m-mono);
  font-weight: 600;
  letter-spacing: -0.02em;
  pointer-events: none;
}

.m-tile-meta {
  font-family: var(--m-mono);
  opacity: 0.7;
  pointer-events: none;
}

.m-map-caption {
  border-top: 2px solid var(--m-ink);
  padding-top: 12px;
  min-height: 96px;
}

.m-map-caption-title {
  font-family: var(--m-mono);
  font-size: 13px;
  font-weight: 600;
}

.m-map-caption-body {
  font-size: 13px;
  color: var(--m-grey);
  line-height: 1.6;
  margin-top: 2px;
}

.m-map-legend {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 18px;
  list-style: none;
  margin-top: 10px;
  font-family: var(--m-mono);
  font-size: 11px;
  color: var(--m-grey);
}

.m-map-legend li {
  display: flex;
  align-items: center;
  gap: 6px;
}

.m-alias {
  opacity: 0.8;
}

.m-swatch {
  width: 10px;
  height: 10px;
  display: inline-block;
  outline: 1.5px solid var(--m-ink);
}
</style>
