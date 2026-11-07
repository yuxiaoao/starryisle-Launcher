<template>
    <div
        aria-label="StarryIsle"
        class="flex h-full items-center gap-2 font-extrabold text-[var(--color-text-primary)]"
    >
        <img aria-hidden="true" class="aspect-square h-full object-contain" :src="axolotlVisual" />
        <span v-if="!iconOnly" class="hidden text-sm tracking-wide xl:inline">StarryIsle</span>
    </div>
</template>

<script setup lang="ts">
import axolotlVisual from '@/assets/branding/starryisle.png'

defineProps<{
    iconOnly?: boolean
}>()
</script>
