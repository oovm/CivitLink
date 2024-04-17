<template>
  <div class="markdown-viewer prose prose-slate max-w-none">
    <div ref="contentRef" v-html="renderedContent"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, nextTick } from 'vue'
import { marked } from 'marked'
import { codeToHtml, createHighlighter } from 'shiki'

interface Props {
  content: string
}

const props = defineProps<Props>()

const renderedContent = ref('')
const contentRef = ref<HTMLElement | null>(null)
let highlighter: any = null

async function initHighlighter() {
  try {
    highlighter = await createHighlighter({
      themes: ['one-dark-pro', 'vitesse-light'],
      langs: ['rust', 'javascript', 'typescript', 'html', 'css', 'toml', 'json', 'bash', 'yaml', 'shell', 'text']
    })
    await renderMarkdown()
  } catch (error) {
    console.error('Failed to initialize highlighter:', error)
    await renderMarkdown()
  }
}

async function highlightCodeElement(codeElement: HTMLElement, preElement: HTMLElement) {
  if (!highlighter) return

  const lang = codeElement.className.replace('language-', '') || 'text'
  const code = codeElement.textContent || ''

  try {
    const highlighted = await codeToHtml(code, {
      lang,
      theme: 'one-dark-pro'
    })
    preElement.outerHTML = highlighted
  } catch (error) {
    console.error('Failed to highlight code:', error)
  }
}

async function applyCodeHighlighting() {
  await nextTick()
  
  if (!contentRef.value || !highlighter) return

  const codeBlocks = contentRef.value.querySelectorAll('pre code')
  const promises: Promise<void>[] = []

  codeBlocks.forEach((codeElement) => {
    const preElement = codeElement.parentElement
    if (preElement && preElement.tagName === 'PRE') {
      promises.push(highlightCodeElement(codeElement as HTMLElement, preElement as HTMLElement))
    }
  })

  await Promise.all(promises)
}

async function renderMarkdown() {
  const content = props.content.replace(/^---\n[\s\S]*?\n---/, '')
  renderedContent.value = marked.parse(content) as string
  await applyCodeHighlighting()
}

watch(() => props.content, renderMarkdown)

onMounted(() => {
  initHighlighter()
})
</script>

<style scoped>
.markdown-viewer {
  @apply text-gray-700;
}

.markdown-viewer :deep(h1) {
  @apply text-3xl font-bold mb-6 mt-8 text-gray-900;
}

.markdown-viewer :deep(h2) {
  @apply text-2xl font-semibold mb-4 mt-6 text-gray-900 border-b-2 border-[#b7410e]/20 pb-2;
}

.markdown-viewer :deep(h3) {
  @apply text-xl font-semibold mb-3 mt-5 text-gray-900;
}

.markdown-viewer :deep(p) {
  @apply mb-4 leading-relaxed;
}

.markdown-viewer :deep(ul),
.markdown-viewer :deep(ol) {
  @apply mb-4 pl-6;
}

.markdown-viewer :deep(li) {
  @apply mb-2;
}

.markdown-viewer :deep(code) {
  @apply bg-gray-100 px-1.5 py-0.5 rounded text-sm font-mono text-[#b7410e];
}

.markdown-viewer :deep(pre) {
  @apply bg-gray-900 text-gray-100 p-4 rounded-xl mb-4 overflow-x-auto;
}

.markdown-viewer :deep(pre code) {
  @apply bg-transparent px-0 py-0 text-gray-100;
}

.markdown-viewer :deep(a) {
  @apply text-[#b7410e] hover:text-[#9c3a0d] underline;
}

.markdown-viewer :deep(blockquote) {
  @apply border-l-4 border-[#b7410e] pl-4 italic text-gray-600 mb-4 bg-[#b7410e]/5 py-2;
}

.markdown-viewer :deep(table) {
  @apply w-full border-collapse mb-4;
}

.markdown-viewer :deep(th),
.markdown-viewer :deep(td) {
  @apply border border-gray-300 px-4 py-2;
}

.markdown-viewer :deep(th) {
  @apply bg-gray-100 font-semibold text-gray-800;
}
</style>
