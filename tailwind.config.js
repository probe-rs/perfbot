/** @type {import('tailwindcss').Config} */
module.exports = {
  mode: "all",
  content: ["./src/**/*.{rs,html,css}", "./dist/**/*.html"],
  theme: {
    extend: {
      colors: {
        "probe-rs-green": "#148571",
        "probe-rs-black": "#0f1116",
      },
    },
  },
  plugins: [],
};
