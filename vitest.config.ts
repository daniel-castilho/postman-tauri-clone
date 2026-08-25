import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    globals: false,
    setupFiles: ['./src/setupTests.ts'],
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'lcov'],
      reportsDirectory: './coverage/frontend',
      // Gate the surfaces that have real suites today. Expand `include` as
      // more component tests land; generated IPC bindings stay excluded.
      // Next increment: remaining components under src/components/**.
      include: [
        'src/lib/**/*.ts',
        'src/store/**/*.ts',
        'src/hooks/**/*.ts',
        'src/components/LoadTestingPanel/LoadTestCharts.tsx',
        'src/components/LoadTestingPanel/LoadTestReport.tsx',
        'src/components/WorkspaceSelector.tsx',
      ],
      exclude: ['src/types/generated/**', '**/*.d.ts', 'src/__tests__/**'],
      thresholds: {
        lines: 80,
        functions: 80,
        branches: 75,
        statements: 80,
      },
    },
  },
});
