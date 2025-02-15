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
</script>

<template>
    <div class="card">
        <h2 class="title">Results</h2>
        <DataTable :value="data" tableStyle="min-width: 50rem" sortMode="multiple">
            <Column field="id" header="ID" sortable></Column>
            <Column field="name" header="Name" sortable></Column>
            <Column field="status" header="Status" sortable>
                <template #body="{ data }">
                    <Tag :value="data.status" :severity="getSeverity(data.status)" />
                </template>
            </Column>
            <Column field="timestamp" header="Timestamp" sortable></Column>
        </DataTable>
    </div>
</template>

<style scoped>
.card {
    background-color: white;
    padding: 1.5rem;
    border-radius: 6px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.title {
    margin: 0 0 1rem 0;
    font-size: 1.5rem;
    font-weight: 600;
    color: #1e293b;
}
</style>