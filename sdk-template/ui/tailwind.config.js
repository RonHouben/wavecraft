/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}', './node_modules/@wavecraft/components/**/*.js'],
  theme: {
    extend: {
      colors: {
        plugin: {
          dark: '#1a1a1a',
          surface: '#2a2a2a',
          border: '#444444',
          canvas: '#13161A',
          'surface-1': '#1B2129',
          'surface-2': '#242C36',
          'border-strong': '#48607A',
          'text-primary': '#E8EEF5',
          'text-secondary': '#B9C7D8',
          'text-muted': '#7D8DA1',
          focus: '#8BC3FF',
        },
        accent: {
          DEFAULT: '#4a9eff',
          light: '#6bb0ff',
        },
        state: {
          success: '#36C27B',
          warning: '#F0B429',
          danger: '#FF5D6C',
          info: '#64B5FF',
        },
        meter: {
          safe: '#4caf50',
          'safe-light': '#8bc34a',
          warning: '#ffeb3b',
          clip: '#ff1744',
          'clip-dark': '#d50000',
        },
      },
      fontSize: {
        'type-2xs': ['10px', { lineHeight: '12px', fontWeight: '500' }],
        'type-xs': ['11px', { lineHeight: '14px', fontWeight: '500' }],
        'type-sm': ['12px', { lineHeight: '16px', fontWeight: '500' }],
        'type-md': ['14px', { lineHeight: '18px', fontWeight: '500' }],
        'type-lg': ['16px', { lineHeight: '22px', fontWeight: '600' }],
        'type-xl-num': ['20px', { lineHeight: '24px', fontWeight: '700' }],
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
      boxShadow: {
        control: '0 1px 2px rgba(0, 0, 0, 0.24)',
        panel: '0 4px 12px rgba(0, 0, 0, 0.28)',
        'focus-ring': '0 0 0 2px rgba(139, 195, 255, 0.65)',
      },
    },
  },
  plugins: [],
};
