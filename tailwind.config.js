/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.rs",
    "./assets/index.html",
  ],
  // In Tailwind v4, plugins are now imported in the CSS file
  plugins: [],
  
  // DaisyUI configuration
  daisyui: {
    // DaisyUI themes - using lofi and night
    themes: ["lofi", "night"],
    // We will control the theme through the data-theme attribute in Dioxus
    darkTheme: false,
    // Enable DaisyUI base styles
    base: true,
    // Enable styled components
    styled: true,
    // Enable responsive and utility variants
    utils: true,
    // Show logs in console
    logs: true,
  },
  
  // Core Tailwind CSS configuration
  corePlugins: {
    // Reset and base styles
    preflight: true,
  },
  
  // Theme configuration
  theme: {
    extend: {},
  },
  
  // Variant configuration
  variants: {
    extend: {},
  },
};};