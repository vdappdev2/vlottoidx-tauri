/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // VerusIDX Mountain Lake Color Palette (from real mountain lake images)
        verusidx: {
          // Deep Blues/Greys (Mountains & Deep Water)
          'lake-deep': '#1a3a52',
          'mountain-blue': '#2d4a63',
          'stone-dark': '#364048',
          'stone-medium': '#5a6b7a',
          
          // Lake/Water Colors
          'turquoise-deep': '#2d8080',
          'turquoise-bright': '#4da6a6',
          'turquoise-light': '#6db8b8',
          'lake-blue': '#5a8db0',
          
          // Natural Supporting Colors
          'forest-deep': '#2d4d3a',
          'mountain-grey': '#8a9ba8',
          'mountain-mist': '#b8c8d4',
          'snow-ice': '#e8f0f4',
          'sky-soft': '#f4f8fa',
          
          // Legacy mappings for compatibility
          'navy-deepest': '#1a3a52',
          'navy-dark': '#2d4a63',
          'navy-medium': '#364048',
          'navy-purple': '#364048',
          'blue-purple': '#2d8080',
          'blue-bright': '#4da6a6',
          'blue-light': '#6db8b8',
          'blue-sky': '#8a9ba8',
          'purple-bright': '#4da6a6',
          'purple-light': '#6db8b8',
          'purple-pink': '#b8c8d4',
          'teal': '#4da6a6',
          'teal-medium': '#2d8080',
          'teal-light': '#6db8b8',
          'teal-mist': '#b8c8d4',
          'slate': '#364048',
          'stone': '#5a6b7a',
          'fog': '#e8f0f4',
          'snow': '#f4f8fa',
          'white-soft': '#f4f8fa',
          'grey-light': '#b8c8d4',
          'grey-pale': '#e8f0f4',
        },
        // Keep original verus colors for compatibility
        verus: {
          50: '#f0f9ff',
          100: '#e0f2fe',
          200: '#bae6fd',
          300: '#7dd3fc',
          400: '#38bdf8',
          500: '#0ea5e9',
          600: '#0284c7',
          700: '#0369a1',
          800: '#075985',
          900: '#0c4a6e',
        },
      },
      fontFamily: {
        sans: ['-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
        mono: ['SFMono-Regular', 'Consolas', 'Liberation Mono', 'Menlo', 'monospace'],
      },
      spacing: {
        '18': '4.5rem',
        '88': '22rem',
      },
      backgroundImage: {
        // Mountain Lake gradients (for special cases)
        'gradient-lake': 'linear-gradient(135deg, #1a3a52 0%, #2d4a63 100%)',
        'gradient-turquoise': 'linear-gradient(135deg, #2d8080 0%, #4da6a6 100%)',
        'gradient-mountain': 'linear-gradient(135deg, #2d4a63 0%, #364048 100%)',
        'gradient-mist': 'linear-gradient(135deg, #8a9ba8 0%, #b8c8d4 100%)',
        
        // Legacy gradient mappings
        'gradient-navy': 'linear-gradient(135deg, #1a3a52 0%, #2d4a63 100%)',
        'gradient-blue': 'linear-gradient(135deg, #2d8080 0%, #4da6a6 100%)',
        'gradient-subtle': 'linear-gradient(135deg, #364048 0%, #5a6b7a 100%)',
        'gradient-depth': 'linear-gradient(135deg, #2d4a63 0%, #364048 100%)',
        'gradient-dawn': 'linear-gradient(135deg, #364048 0%, #2d8080 100%)',
        'gradient-card': 'linear-gradient(135deg, #2d4a63 0%, #364048 100%)',
        'gradient-hover': 'linear-gradient(135deg, #5a6b7a 0%, #2d8080 100%)',
        'gradient-navy-purple': 'linear-gradient(135deg, #1a3a52 0%, #2d8080 100%)',
        'gradient-purple-blue': 'linear-gradient(135deg, #2d8080 0%, #4da6a6 100%)',
        'gradient-purple-bright': 'linear-gradient(135deg, #4da6a6 0%, #6db8b8 100%)',
      },
    },
  },
  plugins: [],
};