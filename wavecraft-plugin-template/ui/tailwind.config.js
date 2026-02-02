/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        plugin: {
          dark: '#1a1a1a',
          surface: '#2a2a2a',
          border: '#444444',
        },
        accent: {
          DEFAULT: '#4a9eff',
          light: '#6bb0ff',
        },
        meter: {
          safe: '#4caf50',
          'safe-light': '#8bc34a',
          warning: '#ffeb3b',
          clip: '#ff1744',
          'clip-dark': '#d50000',
        },
      },
      fontFamily: {
        sans: [
          '-apple-system',
          'BlinkMacSystemFont',
          'Segoe UI',
          'Roboto',
          'Oxygen',
          'Ubuntu',
          'Cantarell',
          'Fira Sans',
          'Droid Sans',
          'Helvetica Neue',
          'sans-serif',
        ],
        mono: ['SF Mono', 'Monaco', 'Courier New', 'monospace'],
      },
      animation: {
        'clip-pulse': 'clip-pulse 0.5s ease-in-out infinite alternate',
      },
      keyframes: {
        'clip-pulse': {
          from: { opacity: '1' },
          to: { opacity: '0.7' },
        },
      },
    },
  },
  plugins: [],
};
