<script setup>
const config = useRuntimeConfig();
</script>
<template><p>Nuxt runtime: {{ config.public.label || 'ready' }}</p></template>
