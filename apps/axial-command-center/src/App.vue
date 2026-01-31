<template>
  <div class="axial-container">
    <header>
      <h1>AXIAL v1 (MAX)</h1>
      <div class="status-bar">
        <span>System: Online</span>
        <span>Ledger: Verified</span>
      </div>
    </header>

    <main>
      <section class="tasks">
        <h2>Command Center</h2>
        <div class="input-group">
          <input v-model="taskInput" @keyup.enter="submitTask" placeholder="Enter instruction for the local-first brain..." />
          <button @click="submitTask">Deploy</button>
        </div>
        <div v-if="response" class="response-box">
          {{ response }}
        </div>
        
        <div class="terminal-container">
          <h2>Neural Terminal Replay</h2>
          <div id="terminal"></div>
          <button @click="replayLastSession" class="secondary">Replay session</button>
        </div>
      </section>

      <section class="ledger-view">
        <h2>Ledger Timeline</h2>
        <ul>
          <li v-for="entry in ledgerEntries" :key="entry.index">
            <span class="timestamp">{{ new Date(entry.timestamp).toLocaleTimeString() }}</span>
            <span class="payload">{{ entry.payload }}</span>
          </li>
        </ul>
      </section>
    </main>

    <aside class="workforce">
      <h2>Workforce Profiles</h2>
      <div class="profile-card" v-for="profile in profiles" :key="profile.name">
        <h3>{{ profile.name }}</h3>
        <p>{{ profile.preferred_tools.join(', ') }}</p>
      </div>
    </aside>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const taskInput = ref('');
const response = ref('');
const ledgerEntries = ref([]);
const profiles = ref([
  { name: 'Research Architect', preferred_tools: ['Aider', 'Claude'] },
  { name: 'Infrastructure Bot', preferred_tools: ['Terraform', 'CLI'] }
]);

import { Terminal } from 'xterm';
import 'xterm/css/xterm.css';

let term;

const replayLastSession = async () => {
  if (!term) {
    term = new Terminal({ theme: { background: '#0f172a' } });
    term.open(document.getElementById('terminal'));
  }
  
  try {
    const events = await invoke('get_pty_events', { sessionId: "last" }); 
    term.clear();
    for (const event of events) {
      term.write(new Uint8Array(event.data));
    }
  } catch (err) {
    console.error('Replay failed:', err);
  }
};

const submitTask = async () => {
  if (!taskInput.value) return;
  try {
    response.value = 'Routing...';
    const res = await invoke('run_axial_task', { task: taskInput.value });
    response.value = res;
    taskInput.value = '';
    await fetchLedger();
  } catch (err) {
    response.value = `Error: ${err}`;
  }
};

const fetchLedger = async () => {
  try {
    ledgerEntries.value = await invoke('get_ledger_entries');
  } catch (err) {
    console.error('Failed to fetch ledger:', err);
  }
};

onMounted(() => {
  fetchLedger();
  setInterval(fetchLedger, 5000);
});
</script>

<style scoped>
.axial-container {
  display: grid;
  grid-template-areas: 
    "header header"
    "main side";
  grid-template-columns: 1fr 300px;
  height: 100vh;
  background: #0f172a;
  color: #f8fafc;
  font-family: 'Inter', sans-serif;
}

header {
  grid-area: header;
  padding: 1rem 2rem;
  background: #1e293b;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #334155;
}

main {
  grid-area: main;
  padding: 2rem;
  overflow-y: auto;
}

aside {
  grid-area: side;
  padding: 2rem;
  background: #1e293b;
  border-left: 1px solid #334155;
}

.input-group {
  display: flex;
  gap: 1rem;
  margin-bottom: 2rem;
}

input {
  flex: 1;
  padding: 0.75rem;
  border-radius: 0.5rem;
  border: 1px solid #334155;
  background: #0f172a;
  color: white;
}

button {
  padding: 0.75rem 1.5rem;
  background: #3b82f6;
  border: none;
  border-radius: 0.5rem;
  color: white;
  cursor: pointer;
}

.response-box {
  background: #334155;
  padding: 1rem;
  border-radius: 0.5rem;
  border-left: 4px solid #3b82f6;
}

.ledger-view ul {
  list-style: none;
  padding: 0;
}

.ledger-view li {
  padding: 0.5rem 0;
  border-bottom: 1px solid #334155;
  font-size: 0.9rem;
}

.timestamp {
  color: #94a3b8;
  margin-right: 1rem;
}
</style>
