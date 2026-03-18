<template>
  <div class="docs-sidebar">
    <div class="sticky top-24">
      <h2 class="text-xl font-bold text-gray-800 mb-4">文档导航</h2>
      <nav v-if="docTree.length > 0">
        <div v-for="node in docTree" :key="node.id" class="mb-2">
          <DocTreeNode :node="node" :current-path="currentDocPath" @select="onSelect" />
        </div>
      </nav>
      <div v-else class="text-gray-500 text-sm">
        加载文档中...
      </div>

      <div v-if="hasPrev || hasNext" class="mt-8 pt-6 border-t border-gray-200">
        <div class="flex flex-col gap-3">
          <button
            v-if="hasPrev"
            @click="goToPrev"
            class="flex items-center gap-2 text-left text-sm text-gray-600 hover:text-orange-600 transition-colors"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
            </svg>
            <div class="flex flex-col">
              <span class="text-xs text-gray-400">上一页</span>
              <span class="font-medium">{{ prevDoc?.title }}</span>
            </div>
          </button>
          <button
            v-if="hasNext"
            @click="goToNext"
            class="flex items-center gap-2 text-right text-sm text-gray-600 hover:text-orange-600 transition-colors"
          >
            <div class="flex flex-col flex-1">
              <span class="text-xs text-gray-400">下一页</span>
              <span class="font-medium">{{ nextDoc?.title }}</span>
            </div>
            <svg class="w-4 h-4 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import DocTreeNode from './DocTreeNode.vue'
import type { DocNode } from '@/docs'

interface Props {
  docTree: DocNode[]
  currentDocPath: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'select', node: DocNode): void
}>()

function onSelect(node: DocNode) {
  emit('select', node)
}

function flattenDocs(nodes: DocNode[]): DocNode[] {
  const result: DocNode[] = []
  nodes.forEach(node => {
    if (!node.isDirectory) {
      result.push(node)
    }
    if (node.children) {
      result.push(...flattenDocs(node.children))
    }
  })
  return result
}

const allDocs = computed(() => flattenDocs(props.docTree))

const currentIndex = computed(() => {
  return allDocs.value.findIndex(doc => doc.path === props.currentDocPath)
})

const prevDoc = computed(() => {
  return currentIndex.value > 0 ? allDocs.value[currentIndex.value - 1] : null
})

const nextDoc = computed(() => {
  return currentIndex.value < allDocs.value.length - 1 ? allDocs.value[currentIndex.value + 1] : null
})

const hasPrev = computed(() => prevDoc.value !== null)
const hasNext = computed(() => nextDoc.value !== null)

function goToPrev() {
  if (prevDoc.value) {
    emit('select', prevDoc.value)
  }
}

function goToNext() {
  if (nextDoc.value) {
    emit('select', nextDoc.value)
  }
}
</script>

<style scoped>
.docs-sidebar {
  width: 320px;
  flex-shrink: 0;
}
</style>
