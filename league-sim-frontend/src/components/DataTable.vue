<script setup>
import { ref } from 'vue';
import Button from 'primevue/button';
import Dialog from 'primevue/dialog';

const results = ref([]);
const eventHistoryColumn = ref(false);
const visible = ref(false);
const eventHistoryVisible = ref(false);
const selectedDamageHistory = ref([]);
const selectedEventHistory = ref([]);

const updateResults = (newResults) => {
    results.value = newResults;
};

const toggleEventHistoryColumn = (showEventHistoryColumn) => {
    eventHistoryColumn.value = showEventHistoryColumn;
};

const showDamageHistory = (data) => {
    selectedDamageHistory.value = data.damage_history || [];
    visible.value = true;
};

const showEventHistory = (data) => {
    selectedEventHistory.value = data.event_history || [];
    eventHistoryVisible.value = true;
};

const exportToCSV = () => {
    // Format headers
    const headers = ['Damage', 'Time (s)', 'DPS', 'Items', 'Cost (gold)', 'Ability sequence', 'Results in a kill?'];

    // Format data rows
    const csvData = results.value.map(row => [
        Math.round(row.damage),
        (row.time_ms / 1000.0).toFixed(2),
        Math.round(row.dps),
        `"${row.item_names.join(', ')}"`,
        row.cost,
        `"${row.selected_commands.join(' -> ')}"`,
        row.kill
    ]);

    // Create CSV content
    const csvContent = [headers, ...csvData].map(row => row.join(',')).join('\n');

    // Create downloadable file
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const link = document.createElement('a');
    const url = URL.createObjectURL(blob);
    link.setAttribute('href', url);
    link.setAttribute('download', 'simulation_results.csv');
    link.style.visibility = 'hidden';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
};

defineExpose({
    updateResults,
    toggleEventHistoryColumn
});
</script>

<template>
    <div class="card">
        <div class="header-container">
            <h2 class="title">Top Results ({{ results.length }} results)</h2>
            <Button label="Export to CSV" icon="pi pi-download" @click="exportToCSV" severity="info" />
        </div>
        <DataTable :value="results" tableStyle="min-width: 50rem" sortMode="multiple" class="light-theme">
            <Column field="damage" header="Damage">
                <template #body="{ data }">
                    {{ Math.round(data.damage) }}
                </template>
            </Column>
            <Column field="time_ms" header="Time (s)">
                <template #body="{ data }">
                    {{ data.time_ms / 1000.0 }}
                </template>
            </Column>
            <Column field="dps" header="DPS">
                <template #body="{ data }">
                    {{ Math.round(data.dps) }}
                </template>
            </Column>
            <Column field="item_names" header="Items">
                <template #body="{ data }">
                    {{ data.item_names.join(', ') }}
                </template>
            </Column>
            <Column field="cost" header="Cost (gold)">
                <template #body="{ data }">
                    {{ data.cost }}
                </template>
            </Column>
            <Column field="selected_commands" header="Ability sequence">
                <template #body="{ data }">
                    {{ data.selected_commands.join(' → ') }}
                </template>
            </Column>
            <Column field="kill" header="Results in a kill?">
                <template #body="{ data }">
                    <Checkbox v-model="data.kill" binary disabled variant="filled" />
                </template>
            </Column>
            <Column header="Damage Breakdown">
                <template #body="{ data }">
                    <Button icon="pi pi-chart-line" severity="secondary" text rounded
                        @click="showDamageHistory(data)" />
                </template>
            </Column>
            <Column header="Event History" v-if="eventHistoryColumn">
                <template #body="{ data }">
                    <Button icon="pi pi-history" severity="secondary" text rounded @click="showEventHistory(data)" />
                </template>
            </Column>
        </DataTable>

        <Dialog v-model:visible="visible" modal header="Damage Breakdown" :style="{ width: '80vw' }" dismissableMask>
            <DataTable v-if="selectedDamageHistory.length > 0" :value="selectedDamageHistory"
                tableStyle="min-width: 50rem" class="light-theme">
                <Column field="time_ms" header="Time (s)">
                    <template #body="{ data }">
                        {{ data.time_ms / 1000.0 }}
                    </template>
                </Column>
                <Column field="damage" header="Damage">
                    <template #body="{ data }">
                        <span :style="{
                            color: data.damage_type === 'Magical' ? '#00B0F0' :
                                data.damage_type === 'Physical' ? '#FF8C34' :
                                    data.damage_type === true ? '#F9966B' : 'inherit'
                        }">
                            {{ Math.floor(data.amount) }}
                        </span>
                    </template>
                </Column>
                <Column field="source" header="Source Type" />
                <Column header="Source Name">
                    <template #body="{ data }">
                        {{ data.source === 'Ability' ? data.source_ability :
                            data.source === 'ItemPassive' ? data.source_item :
                                data.source === 'Rune' ? data.source_rune : '' }}
                    </template>
                </Column>
            </DataTable>
        </Dialog>

        <Dialog v-model:visible="eventHistoryVisible" modal header="Event History" :style="{ width: '80vw' }"
            dismissableMask>
            <DataTable v-if="selectedEventHistory.length > 0" :value="selectedEventHistory"
                tableStyle="min-width: 50rem" class="light-theme">
                <Column field="time_ms" header="Time (s)">
                    <template #body="{ data }">
                        {{ data.time_ms / 1000.0 }}
                    </template>
                </Column>
                <Column field="category" header="Event Type" />
                <Column header="Attack Type">
                    <template #body="{ data }">
                        {{ data.attack_type || '-' }}
                    </template>
                </Column>
                <Column header="Passive Effect">
                    <template #body="{ data }">
                        {{ data.passive_effect || '-' }}
                    </template>
                </Column>
                <Column header="Aura">
                    <template #body="{ data }">
                        {{ data.aura || '-' }}
                    </template>
                </Column>
            </DataTable>
        </Dialog>
    </div>
</template>

<style scoped>
.card {
    padding: 1.5rem;
    border-radius: 0.5rem;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.header-container {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
}

.title {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--text-color);
}
</style>
