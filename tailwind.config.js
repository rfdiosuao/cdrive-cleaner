/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{vue,js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        dark: {
          50: "#f8fafc",
          100: "#f1f5f9",
          200: "#e2e8f0",
          300: "#cbd5e1",
          400: "#94a3b8",
          500: "#64748b",
          600: "#475569",
          700: "#334155",
          800: "#1e293b",
          900: "#0f172a",
          950: "#020617",
        },
        accent: {
          blue: "#3b82f6",
          "blue-hover": "#2563eb",
          "blue-light": "#60a5fa",
          green: "#22c55e",
          "green-light": "#4ade80",
          red: "#ef4444",
          "red-light": "#f87171",
          orange: "#f97316",
          "orange-light": "#fb923c",
          yellow: "#eab308",
          purple: "#a855f7",
        },
      },
    },
  },
  plugins: [],
};
