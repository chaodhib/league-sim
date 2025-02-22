<script setup>
import { ref } from 'vue';

const results = ref([]);

const updateResults = (newResults) => {
    results.value = newResults;
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
            <Column field="selected_commands" header="Commands">
                <template #body="{ data }">
                    {{ data.selected_commands.join(' → ') }}
                </template>
            </Column>
            <Column field="kill" header="Results in a kill?">
                <template #body="{ data }">
                    <Checkbox v-model="data.kill" binary disabled variant="filled" />
                </template>
            </Column>
        </DataTable>

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
