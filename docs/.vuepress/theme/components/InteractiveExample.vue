<template>
  <div class="interactive-example">
    <div class="example-header">
      <h3>{{ title }}</h3>
      <button class="run-button" @click="runExample" :disabled="isRunning">
        {{ isRunning ? 'Running...' : 'Run Example' }}
      </button>
    </div>
    
    <div class="code-container">
      <slot name="code"></slot>
    </div>
    
    <div v-if="output" class="output-container">
      <h4>Output:</h4>
      <pre><code>{{ output }}</code></pre>
    </div>
    
    <div v-if="error" class="error-container">
      <h4>Error:</h4>
      <pre><code>{{ error }}</code></pre>
    </div>
  </div>
</template>

<script>
export default {
  name: 'InteractiveExample',
  props: {
    title: {
      type: String,
      required: true
    },
    exampleId: {
      type: String,
      required: true
    }
  },
  data() {
    return {
      isRunning: false,
      output: null,
      error: null
    }
  },
  methods: {
    async runExample() {
      this.isRunning = true
      this.output = null
      this.error = null
      
      try {
        // In a real implementation, this would call your Rust code
        // For now, we'll simulate a response
        await new Promise(resolve => setTimeout(resolve, 1000))
        this.output = 'Example output would appear here'
      } catch (err) {
        this.error = err.message
      } finally {
        this.isRunning = false
      }
    }
  }
}
</script>

<style scoped>
.interactive-example {
  border: 1px solid var(--c-border);
  border-radius: 6px;
  margin: 1.5rem 0;
  overflow: hidden;
}

.example-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  background-color: var(--c-bg-soft);
  border-bottom: 1px solid var(--c-border);
}

.run-button {
  padding: 0.5rem 1rem;
  background-color: var(--c-brand);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.run-button:hover {
  background-color: var(--c-brand-dark);
}

.run-button:disabled {
  background-color: var(--c-text-light);
  cursor: not-allowed;
}

.code-container {
  padding: 1rem;
}

.output-container,
.error-container {
  padding: 1rem;
  margin-top: 1rem;
  border-top: 1px solid var(--c-border);
}

.error-container {
  background-color: var(--c-danger-bg);
  color: var(--c-danger);
}

pre {
  margin: 0;
  padding: 1rem;
  background-color: var(--c-bg-soft);
  border-radius: 4px;
  overflow-x: auto;
}

code {
  font-family: 'Fira Code', monospace;
  font-size: 0.9em;
}
</style> 