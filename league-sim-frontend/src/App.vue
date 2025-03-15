<script setup>
import { ref, watch, nextTick } from 'vue';
import { getCurrentInstance } from 'vue';
import * as wasm from "league-sim";
import Button from 'primevue/button';
import Dropdown from 'primevue/dropdown';
import Message from 'primevue/message';
import TabPanel from './components/TabPanel.vue';
import DataTable from './components/DataTable.vue';
import ChampionIcon from './components/icons/ChampionIcon.vue';
import GitHubIcon from './components/icons/GitHubIcon.vue';
const tabPanelRef = ref(null);
const dataTableRef = ref(null);
const errorMessage = ref('');
const isSimulationRunning = ref(false);

// Simulation mode
const simulationModes = [
  {
    name: 'Item Optimizer',
    code: 'items',
    tooltip: 'Find the best item combinations for maximum damage'
  },
  {
    name: 'Combo Optimizer',
    code: 'combo',
    tooltip: 'Find the most effective ability sequence'
  },
  {
    name: 'Single Simulation',
    code: 'single',
    tooltip: 'Run a single simulation with the current settings'
  }
];
const selectedMode = ref(simulationModes[0]);

async function runSimulation() {
  const state = tabPanelRef.value.getState();

  // Map frontend state to backend config format
  const backendState = {
    mode: state.mode,
    abilitySequence: state.abilities.sequence,
    champion: state.champion,
    config: {
      CHAMPION_KHAZIX_ISOLATED_TARGET: state.champion.isolatedTarget ? "TRUE" : "FALSE",
      CHAMPION_KHAZIX_Q_EVOLVED: state.champion.qEvolved ? "TRUE" : "FALSE",
      CHAMPION_KHAZIX_R_EVOLVED: state.champion.rEvolved ? "TRUE" : "FALSE",
      RUNE_DARK_HARVEST_STACKS: state.runes.darkHarvestStacks.toString(),
      ITEM_HUBRIS_EMINENCE_ACTIVE: state.items.hubrisEminenceActive ? "TRUE" : "FALSE",
      ITEM_HUBRIS_EMINENCE_STACKS: state.items.hubrisEminenceStacks.toString(),
      ITEM_OPPORTUNITY_PREPARATION_READY: state.items.opportunityPreparationReady ? "TRUE" : "FALSE"
    },
    game: state.game,
    runes: state.runes,
    items: state.items,
    selectedItemIds: state.items.selected.map(item => item.id),
    target: state.target,
    general: state.general
  };

  // Execute simulation with the selected mode
  const result = await wasm.execute_simulation(backendState);

  console.log('Simulation result:', result);

  // Update results table with TopResult array
  dataTableRef.value.updateResults(result);
  dataTableRef.value.toggleEventHistoryColumn(state.general.showDetailledEventHistory)
};

const instance = getCurrentInstance();

async function startSimulation() {
  errorMessage.value = ''; // Clear any previous error
  isSimulationRunning.value = true;
  dataTableRef.value.updateResults([]);
  console.log('Simulation started');

  // Use nextTick to ensure Vue updates the UI before starting the async operation
  const $forceNextTick = instance.appContext.config.globalProperties.$forceNextTick;
  await $forceNextTick();

  try {
    await runSimulation();
    console.log('Simulation ended');
  } catch (error) {
    console.error('Simulation error:', error);
    errorMessage.value = error.message;
  } finally {
    isSimulationRunning.value = false;
  }
};

// Clear error when mode changes
watch(() => selectedMode.value, () => {
  errorMessage.value = '';
});
</script>

<template>
  <div class="app-layout">
    <div class="side-panel">
      <div class="side-panel-content">
        <div class="header">
          <div class="logo-container">
            <span class="logo-text">LeagueSim</span>
            <ChampionIcon champion="Khazix" size="48" />
          </div>
        </div>
        <div class="field mb-4">
          <label for="mode" class="block mb-2"
            v-tooltip.right="'Item Optimizer: The user needs to input the ability sequence. The sim tests every item combination and finds the combination with the highest damage output.\n\nCombo Optimizer: The user needs to input the item build. The sim tests every ability combination and finds the quickest ability sequence to kill the target.\n\nSingle Simulation: Test a specific item build and ability sequence.'">Simulation
            Mode:</label>
          <Dropdown id="mode" v-model="selectedMode" :options="simulationModes" optionLabel="name" class="w-full">
            <template #option="slotProps">
              <div class="p-2" v-tooltip.right.focus="slotProps.option.tooltip">
                {{ slotProps.option.name }}
              </div>
            </template>
          </Dropdown>
        </div>
        <Button severity="primary" class="p-button-lg w-full" raised @click="startSimulation">
          <div v-if="isSimulationRunning">
            <i class="pi pi-spinner pi-spin"></i> <span>Simulation in progress</span>
          </div>
          <div v-else>
            Run simulation
          </div>
        </Button>
        <Message v-if="errorMessage" severity="error">{{ errorMessage }}</Message>
        <div class="github-link">
          <a href="https://github.com/chaodhib/league-sim" target="_blank" rel="noopener noreferrer">
            <GitHubIcon />
          </a>
        </div>
      </div>
    </div>
    <div class="main-content">
      <TabPanel ref="tabPanelRef" :mode="selectedMode.code" />
      <DataTable ref="dataTableRef" />
    </div>
  </div>
</template>

<style>
.app-layout {
  display: flex;
  width: 100%;
  min-height: 100vh;
  background-color: var(--surface-ground);
}

.side-panel {
  width: 250px;
  background-color: var(--surface-card);
  border-right: 1px solid var(--surface-border);
  padding: 1rem;
  box-shadow: var(--card-shadow);
}

.side-panel-content {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  height: 100%;
  min-height: calc(100vh - 2rem);
  /* Account for padding */
}

.side-panel-content .field label {
  display: block;
  margin-bottom: 0.5rem;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--text-color-secondary);
}

.main-content {
  flex: 1;
  padding: 2rem;
  background-color: var(--surface-ground);
}

#app {
  max-width: 100%;
  margin: 0;
  padding: 0;
  text-align: left;
}

body {
  margin: 0;
  padding: 0;
}
</style>

<style scoped>
.header {
  display: flex;
  justify-content: center;
  margin-bottom: 1rem;
}

.logo-container {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.logo-text {
  font-size: 1.5rem;
  font-weight: bold;
  color: var(--text-color);
}

.header :deep(.champion-icon) {
  transition: transform 0.2s ease;
}

.header:hover :deep(.champion-icon) {
  transform: scale(1.1);
}

.github-link {
  margin-top: auto;
  display: flex;
  justify-content: center;
  padding-top: 1rem;
}

.github-link a {
  color: var(--text-color-secondary);
  transition: color 0.2s ease;
}

.github-link a:hover {
  color: var(--text-color);
}

.github-link svg {
  width: 48px;
  height: 48px;
}
</style>
