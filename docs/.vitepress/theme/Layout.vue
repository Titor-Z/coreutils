<script setup>
import DefaultTheme from 'vitepress/theme'
import { useData } from 'vitepress'
import MondrianHome from './components/MondrianHome.vue'

const { frontmatter } = useData()
const DefaultLayout = DefaultTheme.Layout
</script>

<template>
  <MondrianHome v-if="frontmatter.layout === 'mondrian'" />
  <DefaultLayout v-else />
</template>
