const { fontFamily } = require('tailwindcss/defaultTheme');

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./templates/*.html'],
  theme: {
    extend: {
      fontFamily: {
        sans: ['JetBrains Mono', ...fontFamily.mono],
      },
    },
  },
};
