/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{rs,html}", "./styles/**/*.css"],
  theme: {
    extend: {
      colors: {
        ink: "#0f172a",
        mist: "#e2e8f0",
        accent: "#0ea5e9",
      },
    },
  },
  plugins: [],
};
