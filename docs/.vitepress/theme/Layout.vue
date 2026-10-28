<script setup>
import { useData } from 'vitepress'
import MondrianHome from './components/MondrianHome.vue'
import MondrianShell from './components/MondrianShell.vue'

const { frontmatter } = useData()
</script>

<template>
  <MondrianHome v-if="frontmatter.layout === 'mondrian'" />
  <MondrianShell v-else>
    <Content />
  </MondrianShell>
</template>
