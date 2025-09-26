import type { Config } from 'tailwindcss';

export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        background: '#05060a',
        panel: '#0f111a',
        accent: '#49ffcb',
        accentSoft: '#0d7a5f',
        border: '#1d2233'
      }
    }
  },
  plugins: []
} satisfies Config;
