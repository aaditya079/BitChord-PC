/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        brand: {
          50: '#fff1f2',
          100: '#ffe4e6',
          400: '#fb7185',
          500: '#f43f5e',
          600: '#e11d48',
        },
        surface: {
          base: 'rgba(15, 15, 18, 0.75)',
          overlay: 'rgba(25, 25, 30, 0.65)',
          card: 'rgba(32, 32, 38, 0.55)',
        }
      },
      backdropBlur: {
        xs: '2px',
        '2xl': '40px',
        '3xl': '64px',
      },
      boxShadow: {
        'glass-glow': '0 8px 32px 0 rgba(225, 29, 72, 0.15)',
        'glass-subtle': '0 4px 20px 0 rgba(0, 0, 0, 0.4)',
      }
    },
  },
  plugins: [],
}
