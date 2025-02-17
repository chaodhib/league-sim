import './assets/main.css'
import './style.css'
import * as wasm from "league-sim";

import { createApp } from 'vue'
import App from './App.vue'
import PrimeVue from 'primevue/config';
import Aura from '@primevue/themes/aura';
import Button from 'primevue/button'
import Dropdown from 'primevue/dropdown'
import TabView from 'primevue/tabview'
import TabPanel from 'primevue/tabpanel'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import Tag from 'primevue/tag'
import InputNumber from 'primevue/inputnumber'
import Checkbox from 'primevue/checkbox'
import Tooltip from 'primevue/tooltip';

import 'primeicons/primeicons.css'

const app = createApp(App);
app.use(PrimeVue, {
    // Default theme configuration
    theme: {
        preset: Aura,
        options: {
            prefix: 'p',
            darkModeSelector: 'system',
            cssLayer: false
        }
    }
});

// Register PrimeVue components
app.component('Button', Button)
app.component('Dropdown', Dropdown)
app.component('TabView', TabView)
app.component('TabPanel', TabPanel)
app.component('DataTable', DataTable)
app.component('Column', Column)
app.component('Tag', Tag)
app.component('InputNumber', InputNumber)
app.component('Checkbox', Checkbox)

app.directive('tooltip', Tooltip);
app.component('Tooltip', Tooltip);

app.mount('#app');
wasm.init();
// wasm.execute_simulation();
