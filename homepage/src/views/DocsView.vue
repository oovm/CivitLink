<template>
  <div class="w-full px-4 py-8">
    <div class="flex gap-8 max-w-7xl mx-auto">
      <DocsSidebar 
        :doc-tree="docTree"
        :current-doc-path="currentDocPath"
        @select="selectDoc"
      />

      <main class="flex-1 min-w-0">
        <div v-if="currentDoc">
          <h2 class="text-3xl font-bold mb-6 text-gray-800">{{ currentDoc.title }}</h2>
          <MarkdownViewer :content="currentDocContent" />
          
          <div v-if="hasPrev || hasNext" class="mt-12 pt-8 border-t border-gray-200">
            <div class="flex gap-4">
              <button
                v-if="hasPrev"
                @click="goToPrev"
                class="flex-1 flex items-center gap-3 p-4 rounded-xl border border-gray-200 hover:border-orange-300 hover:bg-orange-50 transition-all text-left group"
              >
                <svg class="w-5 h-5 text-gray-400 group-hover:text-orange-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
                </svg>
                <div>
                  <span class="text-sm text-gray-400">上一页</span>
                  <div class="font-semibold text-gray-800 group-hover:text-orange-600">{{ prevDoc?.title }}</div>
                </div>
              </button>
              <button
                v-if="hasNext"
                @click="goToNext"
                class="flex-1 flex items-center justify-end gap-3 p-4 rounded-xl border border-gray-200 hover:border-orange-300 hover:bg-orange-50 transition-all text-right group"
              >
                <div>
                  <span class="text-sm text-gray-400">下一页</span>
                  <div class="font-semibold text-gray-800 group-hover:text-orange-600">{{ nextDoc?.title }}</div>
                </div>
                <svg class="w-5 h-5 text-gray-400 group-hover:text-orange-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                </svg>
              </button>
            </div>
          </div>
        </div>
        <div v-else class="text-gray-500 text-center py-12">
          <p class="text-lg">请从左侧选择文档查看</p>
        </div>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { loadDocs, getDocContent, type DocNode } from '@/docs'
import MarkdownViewer from '@/components/MarkdownViewer.vue'
import DocsSidebar from '@/components/DocsSidebar.vue'

const docTree = ref<DocNode[]>([])
const currentDocPath = ref<string>('')
const currentDoc = ref<DocNode | null>(null)
const currentDocContent = ref<string>('')

async function init() {
  console.log('Initializing docs...')
  docTree.value = await loadDocs()
  console.log('Loaded docTree:', docTree.value)
  if (docTree.value.length > 0) {
    const firstDoc = findFirstDoc(docTree.value[0])
    if (firstDoc) {
      selectDoc(firstDoc)
    }
  }
}

function findFirstDoc(node: DocNode): DocNode | null {
  if (!node.isDirectory && !node.children?.length) {
    return node
  }
  if (node.children && node.children.length > 0) {
    return findFirstDoc(node.children[0])
  }
  return null
}

async function selectDoc(node: DocNode) {
  currentDocPath.value = node.path
  currentDoc.value = node
  currentDocContent.value = await getDocContent(node.path)
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

const allDocs = computed(() => flattenDocs(docTree.value))

const currentIndex = computed(() => {
  return allDocs.value.findIndex(doc => doc.path === currentDocPath.value)
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
    selectDoc(prevDoc.value)
  }
}

function goToNext() {
  if (nextDoc.value) {
    selectDoc(nextDoc.value)
  }
}

onMounted(() => {
  init()
})
</script>
