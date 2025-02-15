<script setup>
import { ref } from 'vue';

const isolatedTarget = ref(false);
const qEvolved = ref(false);
const rEvolved = ref(false);

// Target stats
const armor = ref(0);
const maxHealth = ref(100);
const currentHealth = ref(100);
const magicResistance = ref(0);
</script>

<template>
    <TabView>
        <TabPanel header="General">
            <h2>General Settings</h2>
            <!-- Add general content here -->
        </TabPanel>

        <TabPanel header="Attacker">
            <h2>Attacker Settings</h2>
            <div class="checkbox-group">
                <div class="field-checkbox">
                    <Checkbox v-model="isolatedTarget" :binary="true" inputId="isolatedTarget" />
                    <label for="isolatedTarget">Isolated target</label>
                </div>
                <div class="field-checkbox">
                    <Checkbox v-model="qEvolved" :binary="true" inputId="qEvolved" />
                    <label for="qEvolved">Q Evolved</label>
                </div>
                <div class="field-checkbox">
                    <Checkbox v-model="rEvolved" :binary="true" inputId="rEvolved" />
                    <label for="rEvolved">R Evolved</label>
                </div>
            </div>
        </TabPanel>

        <TabPanel header="Target">
            <h2>Target Settings</h2>
            <div class="input-group">
                <div class="field">
                    <label for="armor">Armor</label>
                    <InputNumber id="armor" v-model="armor" :min="0" showButtons buttonLayout="horizontal"
                        incrementButtonIcon="pi pi-plus" decrementButtonIcon="pi pi-minus" />
                </div>
                <div class="field">
                    <label for="magicResistance">Magic Resistance</label>
                    <InputNumber id="magicResistance" v-model="magicResistance" :min="0" showButtons
                        buttonLayout="horizontal" incrementButtonIcon="pi pi-plus" decrementButtonIcon="pi pi-minus" />
                </div>
                <div class="field">
                    <label for="maxHealth">Max Health</label>
                    <InputNumber id="maxHealth" v-model="maxHealth" :min="0" showButtons buttonLayout="horizontal"
                        incrementButtonIcon="pi pi-plus" decrementButtonIcon="pi pi-minus" />
                </div>
                <div class="field">
                    <label for="currentHealth">Current Health</label>
                    <InputNumber id="currentHealth" v-model="currentHealth" :min="0" :max="maxHealth" showButtons
                        buttonLayout="horizontal" incrementButtonIcon="pi pi-plus" decrementButtonIcon="pi pi-minus" />
                </div>
            </div>
        </TabPanel>
    </TabView>
</template>

<style scoped>
.checkbox-group {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    margin-top: 1rem;
}

.field-checkbox {
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.input-group {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    margin-top: 1rem;
}

.field {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
}

.field label {
    font-size: 0.875rem;
    font-weight: 500;
    color: #64748b;
}
</style>