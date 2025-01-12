<script setup>
  import InputText from 'primevue/inputtext';
  import Button from 'primevue/button';
  import { ref } from 'vue';

  const text = ref('');
</script>

<template>
  
  <div>
    <InputText v-model="text" />
    <Button label="Submit" />

    {{ text }}
  </div>
</template>
