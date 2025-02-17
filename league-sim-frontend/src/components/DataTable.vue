T
<script setup>
import { ref } from 'vue';

const data = ref([
    { id: 1, name: 'Test 1', status: 'Active', timestamp: '2024-02-20 10:00:00' },
    { id: 2, name: 'Test 2', status: 'Inactive', timestamp: '2024-02-20 11:00:00' },
    { id: 3, name: 'Test 3', status: 'Active', timestamp: '2024-02-20 12:00:00' },
]);

const getSeverity = (status) => {
    switch (status.toLowerCase()) {
        case 'active':
            return 'success';
        case 'inactive':
            return 'danger';
        default:
            return 'info';
    }
};

// const results = ref([]);

// // Initialize with default values
// const initializeResults = () => {
//     results.value = [
//         {
//             ability: 'Q - Taste Their Fear',
//             baseDamage: 0,
//             bonusDamage: 0,
//             totalDamage: 0,
//             mitigatedDamage: 0,
//             type: 'Physical'
//         },
//         {
//             ability: 'W - Void Spike',
//             baseDamage: 0,
//             bonusDamage: 0,
//             totalDamage: 0,
//             mitigatedDamage: 0,
//             type: 'Physical'
//         },
//         {
//             ability: 'E - Leap',
//             baseDamage: 0,
//             bonusDamage: 0,
//             totalDamage: 0,
//             mitigatedDamage: 0,
//             type: 'Physical'
//         },
//         {
//             ability: 'Passive - Unseen Threat',
//             baseDamage: 0,
//             bonusDamage: 0,
//             totalDamage: 0,
//             mitigatedDamage: 0,
//             type: 'Magic'
//         }];
// };

// // Update results with new damage values
// const updateResults = (damageValues) => {
//     results.value = results.value.map((result, index) => ({
//         ...result,
//         ...damageValues[index]
//     }));
// };

// // Initialize on component creation
// initializeResults();

// defineExpose({
//     updateResults
// });

// const getDamageType = (type) => {
//     switch (type.toLowerCase()) {
//         case 'physical':
//             return 'danger';
//         case 'magic':
//             return 'info';
//         default:
//             return 'warning';
//     }
// };
</script>

<template>
    <div class="card">
        <h2 class="title">Results</h2>
        <DataTable :value="data" tableStyle="min-width: 50rem" sortMode="multiple" class="light-theme">
            <Column field="id" header="ID" sortable></Column>
            <Column field="name" header="Name" sortable></Column>
            <Column field="status" header="Status" sortable>
                <template #body="{ data }">
                    <Tag :value="data.status" :severity="getSeverity(data.status)" />
                </template>
            </Column>
            <Column field="timestamp" header="Timestamp" sortable></Column>
        </DataTable>

        <!-- <DataTable :value="results" tableStyle="min-width: 50rem">
            <Column field="ability" header="Ability"></Column>
            <Column field="type" header="Damage Type">
                <template #body="{ data }">
                    <Tag :value="data.type" :severity="getDamageType(data.type)" />
                </template>
            </Column>
            <Column field="baseDamage" header="Base Damage">
                <template #body="{ data }">
                    {{ Math.round(data.baseDamage) }}
                </template>
            </Column>
            <Column field="bonusDamage" header="Bonus Damage">
                <template #body="{ data }">
                    {{ Math.round(data.bonusDamage) }}
                </template>
            </Column>
            <Column field="totalDamage" header="Total Damage">
                <template #body="{ data }">
                    {{ Math.round(data.totalDamage) }}
                </template>
            </Column>
            <Column field="mitigatedDamage" header="After Mitigation">
                <template #body="{ data }">
                    {{ Math.round(data.mitigatedDamage) }}
                </template>
            </Column>
        </DataTable> -->
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
