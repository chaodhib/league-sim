<script setup>
// import InputText from 'primevue/inputtext';
// import Button from 'primevue/button';
// import { ref } from 'vue';

// const text = ref('');


// import Tabs from 'primevue/tabs';
// import TabList from 'primevue/tablist';
// import Tab from 'primevue/tab';
// import TabPanels from 'primevue/tabpanels';
// import TabPanel from 'primevue/tabpanel';


// import DataTable from 'primevue/datatable';
// import Column from 'primevue/column';
// import ColumnGroup from 'primevue/columngroup';   // optional
// import Row from 'primevue/row';                   // optional

import TabPanel from './components/TabPanel.vue';
import DataTable from './components/DataTable.vue';

</script>

<template>
  <div class="app-layout">
    <div class="side-panel">
      <Button label="Start" severity="primary" class="p-button-lg w-full" raised />
    </div>
    <div class="main-content">
      <TabPanel />
      <DataTable />
    </div>
  </div>
</template>

<style>
.app-layout {
  display: flex;
  width: 100%;
  min-height: 100vh;
}

.side-panel {
  width: 200px;
  background-color: white;
  border-right: 1px solid #e2e8f0;
  padding: 1rem;
  box-shadow: 2px 0 4px rgba(0, 0, 0, 0.05);
}

.main-content {
  flex: 1;
  padding: 2rem;
  background-color: #f8fafc;
}

#app {
  max-width: 100%;
  margin: 0;
  padding: 0;
  text-align: left;
}

body {
  background-color: #f8fafc;
  margin: 0;
  padding: 0;
}
</style>