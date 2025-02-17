<template>
    <img :src="imageUrl" :width="size" :height="size" :alt="champion" class="champion-icon" />
</template>

<script setup>
const props = defineProps({
    champion: {
        type: String,
        required: true
    },
    size: {
        type: String,
        default: '64'
    }
});

const imageUrl = `http://ddragon.leagueoflegends.com/cdn/14.24.1/img/champion/${props.champion}.png`;
</script>

<style scoped>
.champion-icon {
    border-radius: 50%;
    object-fit: cover;
}
</style>
