import './assets/main.css'
import * as wasm from "league-sim";

import { createApp } from 'vue'
import App from './App.vue'


createApp(App).mount('#app')
wasm.greet();
