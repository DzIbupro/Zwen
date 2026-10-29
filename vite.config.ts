import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
export default defineConfig({
  plugins: [svelte()],
  test: { environment: 'jsdom', include: ['frontend/tests/**/*.test.ts'] },
  build: { target: 'es2022', sourcemap: false },
  clearScreen: false,
});
