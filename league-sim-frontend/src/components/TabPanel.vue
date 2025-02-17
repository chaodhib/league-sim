<script setup>
import { ref, watch } from 'vue';
import runesImage from '../assets/images/runes.png';
import skillOrderImage from '../assets/images/skill_order.png';

const props = defineProps({
    mode: {
        type: String,
        required: true
    }
});

// Clear items and abilities based on mode
watch(() => props.mode, (newMode) => {
    if (newMode === 'items') {
        selectedItems.value = [];
    }
    if (newMode === 'combo') {
        abilitySequence.value = [];
    }
});

// Champion stats
const level = ref(1);
const healthPercentage = ref(100);

// Ability sequence
const abilitySequence = ref([]);
const availableAbilities = [
    { id: 'Q', name: 'Q - Taste Their Fear' },
    { id: 'W', name: 'W - Void Spike' },
    { id: 'E', name: 'E - Leap' },
    { id: 'R', name: 'R - Void Assault' },
    { id: 'AA', name: 'Auto Attack' }
];

const addAbility = (ability) => {
    if (props.mode !== 'combo') {
        abilitySequence.value.push(ability);
    }
};

const removeAbility = (index) => {
    if (props.mode !== 'combo') {
        abilitySequence.value.splice(index, 1);
    }
};

const clearAbilities = () => {
    if (props.mode !== 'combo') {
        abilitySequence.value = [];
    }
};

// Champion evolution settings
const isolatedTarget = ref(false);
const qEvolved = ref(false);
const rEvolved = ref(false);

// Runes
const darkHarvestStacks = ref(0);

// Items
const selectedItems = ref([]);
const availableItems = ref([
    { id: 3158, name: 'Ionian Boots of Lucidity' },
    { id: 3006, name: 'Berserker\'s Greaves' },
    { id: 3142, name: 'Youmuu\'s Ghostblade' },
    { id: 6701, name: 'Opportunity' },
    { id: 3814, name: 'Edge of Night' },
    { id: 6694, name: 'Serylda\'s Grudge' },
    { id: 6698, name: 'Profane Hydra' },
    { id: 6692, name: 'Eclipse' },
    { id: 3156, name: 'Maw of Malmortius' },
    { id: 3179, name: 'Umbral Glaive' },
    { id: 6697, name: 'Hubris' },
    { id: 6333, name: 'Death\'s Dance' },
    { id: 3036, name: 'Lord Dominik\'s Regards' },
    { id: 3033, name: 'Mortal Reminder' },
    { id: 6609, name: 'Chempunk Chainsword' },
    { id: 3071, name: 'Black Cleaver' },
    { id: 6676, name: 'The Collector' },
    { id: 3072, name: 'Bloodthirster' },
    { id: 6699, name: 'Voltaic Cyclosword' },
    { id: 6695, name: 'Serpent\'s Fang' },
    { id: 3026, name: 'Guardian Angel' },
    { id: 3161, name: 'Spear of Shojin' },
    { id: 6696, name: 'Axiom Arc' },
    { id: 6610, name: 'Sundered Sky' },
    { id: 3074, name: 'Ravenous Hydra' },
    { id: 3143, name: 'Randuin\'s Omen' },
    { id: 3110, name: 'Frozen Heart' },
    { id: 6631, name: 'Stridebreaker' },
    { id: 3153, name: 'Blade of the Ruined King' },
]);

// Target stats
const armor = ref(0);
const maxHealth = ref(100);
const currentHealth = ref(100);
const magicResistance = ref(0);

const getState = () => {
    return {
        mode: props.mode,
        champion: {
            level: level.value,
            healthPercentage: healthPercentage.value,
        },
        champion_evolution: {
            isolatedTarget: isolatedTarget.value,
            qEvolved: qEvolved.value,
            rEvolved: rEvolved.value,
        },
        runes: {
            darkHarvestStacks: darkHarvestStacks.value,
            selected: ['Dark Harvest']
        },
        abilities: {
            sequence: abilitySequence.value.map(ability => ability.id)
        },
        items: {
            selected: selectedItems.value
        },
        target: {
            armor: armor.value,
            maxHealth: maxHealth.value,
            currentHealth: currentHealth.value,
            magicResistance: magicResistance.value,
        }
    };
};

const addItem = (item) => {
    if (selectedItems.value.length < 6 && props.mode !== 'items') {
        selectedItems.value.push(item);
    }
};

const removeItem = (index) => {
    if (props.mode !== 'items') {
        selectedItems.value.splice(index, 1);
    }
};

defineExpose({
    getState
});
</script>

<template>
    <TabView class="light-theme">
        <TabPanel header="Champion">
            <h2>Champion Settings</h2>
            <div class="input-group">
                <div class="field">
                    <label for="level">Champion Level</label>
                    <InputNumber id="level" v-model="level" :min="1" :max="18" showButtons buttonLayout="horizontal"
                        incrementButtonIcon="pi pi-plus" decrementButtonIcon="pi pi-minus" />
                </div>
                <div class="field">
                    <label for="healthPercentage">Current percentage health</label>
                    <InputNumber id="healthPercentage" v-model="healthPercentage" :min="0" :max="100" showButtons
                        buttonLayout="horizontal" incrementButtonIcon="pi pi-plus" decrementButtonIcon="pi pi-minus" />
                </div>
            </div>
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

        <TabPanel header="Runes">
            <h2>Rune Settings</h2>
            <div class="runes-container">
                <img :src="runesImage" alt="Runes" class="runes-image" />
                <div class="field">
                    <label for="darkHarvestStacks">Dark Harvest Stacks</label>
                    <InputNumber id="darkHarvestStacks" v-model="darkHarvestStacks" :min="0" showButtons
                        buttonLayout="horizontal" incrementButtonIcon="pi pi-plus" decrementButtonIcon="pi pi-minus" />
                </div>
            </div>
        </TabPanel>

        <TabPanel header="Skill Order">
            <h2>Skill Order</h2>
            <div class="skill-order-container">
                <img :src="skillOrderImage" alt="Skill Order" class="skill-order-image" />
            </div>
        </TabPanel>

        <TabPanel header="Items">
            <h2>Item Settings</h2>
            <div class="items-container">
                <div class="selected-items">
                    <h3>Selected Items {{ props.mode === 'items' ? '(disabled in Item Optimizer mode)' :
                        `(${selectedItems.length}/6)` }}</h3>
                    <div class="items-grid">
                        <div v-for="(item, index) in selectedItems" :key="index" class="item-slot">
                            <Button :label="item.name" severity="secondary" @click="removeItem(index)"
                                :disabled="props.mode === 'items'" />
                        </div>
                    </div>
                </div>
                <div class="available-items">
                    <h3>Available Items</h3>
                    <div class="items-grid">
                        <div v-for="item in availableItems" :key="item.id" class="item-slot">
                            <Button :label="item.name" :disabled="selectedItems.length >= 6 || props.mode === 'items'"
                                @click="addItem(item)" />
                        </div>
                    </div>
                </div>
            </div>
        </TabPanel>

        <TabPanel header="Abilities">
            <h2>Ability Sequence</h2>
            <div class="abilities-container">
                <div class="selected-abilities">
                    <div class="abilities-header">
                        <h3>Selected Sequence {{ props.mode === 'combo' ? '(disabled in Ability Optimizer mode)' : '' }}
                        </h3>
                        <Button label="Clear" severity="danger" @click="clearAbilities"
                            :disabled="abilitySequence.length === 0 || props.mode === 'combo'" />
                    </div>
                    <div class="abilities-sequence">
                        <div v-for="(ability, index) in abilitySequence" :key="index" class="ability-slot">
                            <Button :label="ability.name" severity="secondary" @click="removeAbility(index)"
                                :disabled="props.mode === 'combo'" />
                        </div>
                        <div v-if="abilitySequence.length === 0" class="empty-sequence">
                            No abilities selected
                        </div>
                    </div>
                </div>
                <div class="available-abilities">
                    <h3>Available Abilities</h3>
                    <div class="abilities-grid">
                        <div v-for="ability in availableAbilities" :key="ability.id" class="ability-slot">
                            <Button :label="ability.name" @click="addAbility(ability)"
                                :disabled="props.mode === 'combo'" />
                        </div>
                    </div>
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

.runes-container {
    display: flex;
    flex-direction: column;
    gap: 1rem;
}

.runes-image,
.skill-order-image {
    height: 450px;
    width: auto;
    border-radius: 0.5rem;
    margin-top: 1rem;
    object-fit: contain;
}

.skill-order-container {
    display: flex;
    flex-direction: column;
    gap: 1rem;
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
    color: var(--text-color-secondary);
}

.items-container {
    display: flex;
    flex-direction: column;
    gap: 2rem;
}

.selected-items,
.available-items {
    padding: 1rem;
    border-radius: 0.5rem;
}

.items-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 0.5rem;
    margin-top: 0.5rem;
}

.item-slot {
    display: flex;
    align-items: center;
}

.item-slot :deep(.p-button) {
    width: 100%;
    justify-content: flex-start;
    white-space: normal;
    height: auto;
    min-height: 2.5rem;
    padding: 0.5rem;
}

h3 {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-color);
    margin: 0;
}

.abilities-container {
    display: flex;
    flex-direction: column;
    gap: 2rem;
}

.abilities-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
}

.selected-abilities,
.available-abilities {
    padding: 1rem;
    border-radius: 0.5rem;
}

.abilities-sequence {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    min-height: 3rem;
}

.empty-sequence {
    width: 100%;
    text-align: center;
    color: var(--text-color-secondary);
    font-style: italic;
    padding: 1rem;
}

.abilities-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 0.5rem;
}

.ability-slot {
    display: flex;
    align-items: center;
}

.ability-slot :deep(.p-button) {
    width: 100%;
    justify-content: flex-start;
    white-space: normal;
    height: auto;
    min-height: 2.5rem;
    padding: 0.5rem;
}
</style>
