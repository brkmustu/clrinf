import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  plugins: [svelte(), tailwindcss()],
  resolve: {
    conditions: ['browser'],
  },
  server: {
    port: 3001,
    host: '0.0.0.0',
  },
});
