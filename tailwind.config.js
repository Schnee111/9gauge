/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{svelte,js,ts,jsx,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        void: {
          dark: 'rgba(13, 15, 20, 0.82)',
          light: 'rgba(255, 255, 255, 0.78)',
        },
        glass: {
          card: 'rgba(255, 255, 255, 0.04)',
          hover: 'rgba(255, 255, 255, 0.08)',
          border: 'rgba(255, 255, 255, 0.12)',
        },
        ink: {
          healthy: '#2e9e6b',
          warning: '#d99a2b',
          critical: '#d4553f',
          neutral: '#717682',
        },
        token: {
          prompt: '#60a5fa',
          output: '#a78bfa',
          cache: '#34d399',
        },
      },
      fontFamily: {
        sans: ['"Plus Jakarta Sans"', 'system-ui', 'sans-serif'],
        mono: ['"JetBrains Mono"', 'monospace'],
      },
      boxShadow: {
        bezel: 'inset 0 1px 0 rgba(255, 255, 255, 0.18)',
        'bezel-outer': '0 0 0 1px rgba(255, 255, 255, 0.12)',
        double: '0 0 0 1px rgba(255, 255, 255, 0.12), inset 0 1px 0 rgba(255, 255, 255, 0.18)',
      },
      keyframes: {
        breathe: {
          '0%, 100%': { opacity: '0.9', transform: 'scale(1)' },
          '50%': { opacity: '0.35', transform: 'scale(0.85)' },
        },
      },
      animation: {
        breathe: 'breathe 3s ease-in-out infinite',
      },
    },
  },
  plugins: [],
};
