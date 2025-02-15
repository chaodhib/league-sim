import './assets/main.css'
import './style.css'
import * as wasm from "league-sim";

import { createApp } from 'vue'
import App from './App.vue'
import PrimeVue from 'primevue/config';
import Aura from '@primevue/themes/aura';
import Material from '@primevue/themes/material';

import Button from 'primevue/button'
import TabView from 'primevue/tabview'
import TabPanel from 'primevue/tabpanel'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import Tag from 'primevue/tag'
import Checkbox from 'primevue/checkbox'
import InputNumber from 'primevue/inputnumber'

// import 'primevue/resources/themes/lara-light-blue/theme.css'
// import 'primeicons/primeicons.css'

const app = createApp(App);
app.use(PrimeVue
    , {
        theme: {
            preset: Material
        }
    }
);

// Register PrimeVue components
app.component('Button', Button)
app.component('TabView', TabView)
app.component('TabPanel', TabPanel)
app.component('DataTable', DataTable)
app.component('Column', Column)
app.component('Tag', Tag)
app.component('Checkbox', Checkbox)
app.component('InputNumber', InputNumber)

app.mount('#app');
wasm.init();
// wasm.execute_simulation();
