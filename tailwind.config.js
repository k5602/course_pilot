const path = require('path');

module.exports = {
  content: [
    "./src/**/*.rs",
    "./assets/index.html",
  ],
  theme: {
    extend: {},
  },
  plugins: [
    require('daisyui'),
  ],
  daisyui: {
    themes: ["lofi", "night"],
  },
}