/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ["./src/**/*.rs", "./assets/index.html"],
    plugins: [],

    // DaisyUI configuration
    daisyui: {
        themes: ["corporate", "business"],
        darkTheme: false,
        base: true,
        styled: true,
        utils: true,
        logs: true,
    },

    // Core Tailwind CSS configuration
    corePlugins: {
        preflight: true,
    },

    theme: {
        extend: {},
    },

    variants: {
        extend: {},
    },
};
