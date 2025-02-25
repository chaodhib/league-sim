<script setup>
import { ref } from 'vue';
import Button from 'primevue/button';
import Dialog from 'primevue/dialog';

const results = ref([]);
const visible = ref(false);
const eventHistoryVisible = ref(false);
const selectedDamageHistory = ref([]);
const selectedEventHistory = ref([]);

const updateResults = (newResults) => {
    results.value = newResults;
};

const showDamageHistory = (data) => {
    selectedDamageHistory.value = data.damage_history || [];
    visible.value = true;
};

const showEventHistory = (data) => {
    selectedEventHistory.value = data.event_history || [];
    eventHistoryVisible.value = true;
};

defineExpose({
    updateResults
});
</script>

<template>
    <div class="card">
        <h2 class="title">Top Results ({{ results.length }} results)</h2>
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
            <Column header="Event History">
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

.title {
    margin: 0 0 1rem 0;
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--text-color);
}
</style>
