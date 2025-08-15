---
title: README
linter-yaml-title-alias: README
date created: Saturday, May 4th 2024, 20:27:22
date modified: Friday, August 15th 2025, 19:23:51
aliases: README
---

## Agni Datta - Academic Website

A modern, responsive academic website built with HTML5, CSS3, and JavaScript. Features a clean design with light and dark theme support, mobile-first responsive design, and smooth animations.

### Features

- **Dual Theme System**: Light theme with royalblue4 palette and dark theme with bluish teal accents
- **System Preference Detection**: Automatically detects and applies user’s system theme preference
- **Mobile-First Design**: Fully responsive with optimized mobile navigation drawer
- **Smooth Animations**: CSS transitions and JavaScript-powered smooth scrolling
- **JavaScript Implementation**: Comprehensive JavaScript with detailed documentation
- **Accessibility**: WCAG compliant with proper ARIA labels and keyboard navigation
- **Performance Optimized**: Service worker support and optimized loading

### Theme System

#### Light Theme (royalblue4)

- Primary: `#4169E1` (Royal Blue)
- Secondary: `#1E3A8A` (Dark Blue)
- Accent: `#3B82F6` (Blue)
- Background: `#FFFFFF` (White)
- Text: `#1F2937` (Dark Gray)

#### Dark Theme (bluish teal)

- Primary: `#0F766E` (Teal)
- Secondary: `#134E4A` (Dark Teal)
- Accent: `#14B8A6` (Light Teal)
- Background: `#0F172A` (Dark Blue)
- Text: `#F1F5F9` (Light Gray)

### Project Structure

```
agni-datta.github.io/
├── index.html              # Main HTML file
├── css/
│   └── styles.css          # Main stylesheet with theme variables
├── js/
│   └── main.js            # Main JavaScript with comprehensive documentation
└── README.md              # This file
```

### Development Setup

#### Prerequisites

- Modern web browser with ES2020 support

#### Quick Start

1. Clone the repository
2. Serve the files using any HTTP server:

   ```bash
   # Using Python 3
   python3 -m http.server 8000
   
   # Using Node.js (if available)
   npx serve .
   
   # Using PHP
   php -S localhost:8000
   ```

3. Open `http://localhost:8000` in your browser

### Code Documentation

#### JavaScript Architecture

The main JavaScript file (`js/main.js`) is organized into logical sections:

##### 1. Theme Management

- `initializeTheme()`: Sets up theme system with localStorage persistence
- `getSystemThemePreference()`: Detects system dark/light mode preference
- `applyTheme(theme)`: Applies theme to DOM elements
- `updateThemeToggleIcons(theme)`: Updates toggle button display
- `setupThemeToggle()`: Sets up theme toggle click handler

##### 2. Mobile Navigation

- `setupMobileNavigation()`: Initializes mobile drawer functionality
- `openDrawer()`: Opens mobile navigation drawer
- `closeDrawer()`: Closes mobile navigation drawer
- `setupNavigationLinks()`: Sets up smooth scrolling navigation
- `setupOutsideClickHandler()`: Closes drawer when clicking outside

##### 3. Scroll Functionality

- `setupScrollToTop()`: Scroll to top button functionality
- `setupMobileScrollBehavior()`: Header hide/show on mobile scroll

##### 4. Mobile Optimizations

- `setupMobileTouchFeedback()`: Touch feedback for mobile interactions
- `updateMobileClasses()`: Dynamic CSS class management
- `setupMobileClassUpdates()`: Responsive class updates

##### 5. Service Worker

- `registerServiceWorker()`: Registers service worker for caching

#### CSS Architecture

The CSS is organized with CSS custom properties for theming:

```css
:root {
  /* Shared variables */
  --transition-speed: 0.3s;
  --border-radius: 8px;
}

[data-theme="light"] {
  /* Light theme variables */
  --primary-color: #4169E1;
  --secondary-color: #1E3A8A;
  /* ... */
}

[data-theme="dark"] {
  /* Dark theme variables */
  --primary-color: #0F766E;
  --secondary-color: #134E4A;
  /* ... */
}
```

### Customization

#### Adding New Themes

1. Add new theme variables in `css/styles.css`
2. Update the `applyTheme()` function in `js/main.js`
3. Add theme toggle icons to the HTML

#### Modifying Colors

Edit the CSS custom properties in the respective theme sections:

- Light theme: `[data-theme="light"]` block
- Dark theme: `[data-theme="dark"]` block

#### Adding New Sections

1. Add HTML content to `index.html`
2. Add navigation links
3. Update mobile navigation if needed

### Mobile Features

- **Responsive Design**: Adapts to all screen sizes
- **Touch Optimized**: Touch feedback and gesture support
- **Mobile Navigation**: Slide-out drawer navigation
- **Header Behavior**: Auto-hide/show on scroll
- **Performance**: Optimized for mobile devices

### Browser Support

- Chrome 80+
- Firefox 75+
- Safari 13+
- Edge 80+

### License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

### Contact

- **Website**: [agni-datta.github.io](https://agni-datta.github.io)
- **Email**: [Your Email]
- **GitHub**: [@agni-datta](https://github.com/agni-datta)

---

**Note**: This website is designed for academic purposes and showcases research, publications, and professional experience in a clean, accessible format.
