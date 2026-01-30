/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      // Apple HIG-inspired design tokens
      colors: {
        // Semantic colors that adapt to light/dark mode
        'system-blue': {
          DEFAULT: '#007AFF',
          dark: '#0A84FF',
        },
        'system-red': {
          DEFAULT: '#FF3B30',
          dark: '#FF453A',
        },
        'system-green': {
          DEFAULT: '#34C759',
          dark: '#30D158',
        },
        'system-orange': {
          DEFAULT: '#FF9500',
          dark: '#FF9F0A',
        },
        'system-yellow': {
          DEFAULT: '#FFCC00',
          dark: '#FFD60A',
        },
        // Background layers
        'bg-primary': {
          DEFAULT: '#FFFFFF',
          dark: '#000000',
        },
        'bg-secondary': {
          DEFAULT: '#F2F2F7',
          dark: '#1C1C1E',
        },
        'bg-tertiary': {
          DEFAULT: '#FFFFFF',
          dark: '#2C2C2E',
        },
        // Gray scale (Apple semantic grays)
        'gray-1': {
          DEFAULT: '#8E8E93',
          dark: '#8E8E93',
        },
        'gray-2': {
          DEFAULT: '#AEAEB2',
          dark: '#636366',
        },
        'gray-3': {
          DEFAULT: '#C7C7CC',
          dark: '#48484A',
        },
        'gray-4': {
          DEFAULT: '#D1D1D6',
          dark: '#3A3A3C',
        },
        'gray-5': {
          DEFAULT: '#E5E5EA',
          dark: '#2C2C2E',
        },
        'gray-6': {
          DEFAULT: '#F2F2F7',
          dark: '#1C1C1E',
        },
        // Text colors
        'label-primary': {
          DEFAULT: '#000000',
          dark: '#FFFFFF',
        },
        'label-secondary': {
          DEFAULT: 'rgba(60, 60, 67, 0.6)',
          dark: 'rgba(235, 235, 245, 0.6)',
        },
        'label-tertiary': {
          DEFAULT: 'rgba(60, 60, 67, 0.3)',
          dark: 'rgba(235, 235, 245, 0.3)',
        },
      },
      fontFamily: {
        // SF Pro system font stack
        sans: [
          '-apple-system',
          'BlinkMacSystemFont',
          'SF Pro Text',
          'SF Pro Display',
          'Helvetica Neue',
          'Helvetica',
          'Arial',
          'sans-serif',
        ],
        mono: [
          'SF Mono',
          'Menlo',
          'Monaco',
          'Consolas',
          'Liberation Mono',
          'Courier New',
          'monospace',
        ],
      },
      // Apple Dynamic Type scale
      fontSize: {
        'large-title': ['34px', { lineHeight: '41px', fontWeight: '400' }],
        'title-1': ['28px', { lineHeight: '34px', fontWeight: '400' }],
        'title-2': ['22px', { lineHeight: '28px', fontWeight: '400' }],
        'title-3': ['20px', { lineHeight: '25px', fontWeight: '400' }],
        headline: ['17px', { lineHeight: '22px', fontWeight: '600' }],
        body: ['17px', { lineHeight: '22px', fontWeight: '400' }],
        callout: ['16px', { lineHeight: '21px', fontWeight: '400' }],
        subhead: ['15px', { lineHeight: '20px', fontWeight: '400' }],
        footnote: ['13px', { lineHeight: '18px', fontWeight: '400' }],
        'caption-1': ['12px', { lineHeight: '16px', fontWeight: '400' }],
        'caption-2': ['11px', { lineHeight: '13px', fontWeight: '400' }],
      },
      // Apple-style border radius
      borderRadius: {
        'apple-sm': '8px',
        'apple': '12px',
        'apple-lg': '16px',
        'apple-xl': '20px',
      },
      // Apple-style shadows
      boxShadow: {
        'apple': '0 1px 3px rgba(0, 0, 0, 0.1), 0 1px 2px rgba(0, 0, 0, 0.06)',
        'apple-md': '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
        'apple-lg': '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
        'apple-modal': '0 25px 50px -12px rgba(0, 0, 0, 0.25)',
      },
      // Spring-based animations (Apple motion)
      transitionTimingFunction: {
        'apple-ease': 'cubic-bezier(0.25, 0.1, 0.25, 1)',
        'apple-spring': 'cubic-bezier(0.175, 0.885, 0.32, 1.275)',
      },
      animation: {
        'fade-in': 'fadeIn 0.2s ease-out',
        'slide-up': 'slideUp 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275)',
        'slide-down': 'slideDown 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275)',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideUp: {
          '0%': { transform: 'translateY(10px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
        slideDown: {
          '0%': { transform: 'translateY(-10px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
      },
    },
  },
  plugins: [],
};
