import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Khi dev với backend thật: đặt API_TARGET (ví dụ http://localhost:3000) 
const target = process.env.API_TARGET || 'http://localhost:3000';

export default defineConfig({
  plugins: [react()],
  server: { proxy: { '/api': target, '/uploads': target } },
  test: {
    environment: 'jsdom',
    setupFiles: './src/test/setup.js',
    include: ['src/**/*.test.{js,jsx}'],
    coverage: { provider: 'v8', include: ['src/**'], exclude: ['src/test/**', 'src/main.jsx'] }
  }
});
