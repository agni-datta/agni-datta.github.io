/**
 * Academic Website - Main JavaScript Module
 * 
 * This module handles all interactive functionality for the academic website including:
 * - Theme switching (light/dark mode with system preference detection)
 * - Mobile navigation drawer
 * - Smooth scrolling navigation
 * - Mobile-specific behaviors and optimizations
 * 
 * @author Agni Datta
 * @version 1.0.0
 */

// ============================================================================
// GLOBAL VARIABLES
// ============================================================================

/** @type {HTMLElement | null} Theme toggle button element */
const themeToggle = document.getElementById('theme-toggle');

/** @type {HTMLElement} Body element for theme application */
const body = document.body;

/** @type {HTMLElement | null} Profile drawer element for mobile navigation */
const profileDrawer = document.getElementById('profile-drawer');

/** @type {HTMLElement | null} Mobile navigation drawer element */
const mobileNav = document.getElementById('mobile-nav');

/** @type {HTMLElement | null} Header content element for collapse animation */
const headerContent = document.querySelector('.header-content');

/** @type {HTMLElement | null} Scroll to top element */
const scrollToTop = document.getElementById('scroll-to-top');

/** @type {HTMLElement | null} Header element for scroll behavior */
const header = document.querySelector('header');

// ============================================================================
// THEME MANAGEMENT
// ============================================================================

/**
 * Initializes the theme system with system preference detection
 * 
 * This function:
 * 1. Checks for a saved theme preference in localStorage
 * 2. Falls back to system preference if no saved theme exists
 * 3. Applies the theme to both body and document elements
 * 4. Updates the theme toggle icon display
 * 5. Sets up the theme toggle click handler
 * 
 * @returns {void}
 */
function initializeTheme() {
    if (!themeToggle) {
        console.warn('Theme toggle element not found');
        return;
    }

    // Check for saved theme preference or use system preference
    let savedTheme = localStorage.getItem('theme') || getSystemThemePreference();

    // Apply theme to both body and document element
    applyTheme(savedTheme);

    // Set up theme toggle click handler
    setupThemeToggle();
}

/**
 * Gets the system's preferred color scheme
 * 
 * @returns {'light' | 'dark'} The system's preferred theme
 */
function getSystemThemePreference() {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

/**
 * Applies a theme to the website
 * 
 * @param {'light' | 'dark'} theme - The theme to apply
 * @returns {void}
 */
function applyTheme(theme) {
    // Apply theme attributes
    body.setAttribute('data-theme', theme);
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('theme', theme);

    // Update icon display
    updateThemeToggleIcons(theme);
}

/**
 * Updates the theme toggle button icons based on current theme
 * 
 * @param {'light' | 'dark'} theme - The current theme
 * @returns {void}
 */
function updateThemeToggleIcons(theme) {
    if (!themeToggle) return;

    const lightIcon = themeToggle.querySelector('.light-icon');
    const darkIcon = themeToggle.querySelector('.dark-icon');

    if (!lightIcon || !darkIcon) {
        console.warn('Theme toggle icons not found');
        return;
    }

    if (theme === 'dark') {
        lightIcon.style.display = 'none';
        darkIcon.style.display = 'block';
    } else {
        lightIcon.style.display = 'block';
        darkIcon.style.display = 'none';
    }
}

/**
 * Sets up the theme toggle button click handler
 * 
 * @returns {void}
 */
function setupThemeToggle() {
    if (!themeToggle) return;

    themeToggle.addEventListener('click', () => {
        const currentTheme = body.getAttribute('data-theme');
        const newTheme = currentTheme === 'light' ? 'dark' : 'light';
        applyTheme(newTheme);
    });
}

// ============================================================================
// MOBILE NAVIGATION DRAWER
// ============================================================================

/**
 * Drawer state management
 * @type {Object}
 */
const drawerState = {
    isOpen: false,
    profileDrawer: profileDrawer,
    mobileNav: mobileNav,
    headerContent: headerContent
};

/**
 * Opens the mobile navigation drawer
 * 
 * This function:
 * 1. Adds active classes to drawer elements
 * 2. Prevents body scrolling
 * 3. Updates drawer state
 * 
 * @returns {void}
 */
function openDrawer() {
    if (!mobileNav || !headerContent || !profileDrawer) {
        console.warn('Drawer elements not found');
        return;
    }

    mobileNav.classList.add('active');
    headerContent.classList.add('collapsed');
    profileDrawer.classList.add('active');
    drawerState.isOpen = true;
    document.body.style.overflow = 'hidden';
}

/**
 * Closes the mobile navigation drawer
 * 
 * This function:
 * 1. Removes active classes from drawer elements
 * 2. Restores body scrolling
 * 3. Updates drawer state
 * 
 * @returns {void}
 */
function closeDrawer() {
    if (!mobileNav || !headerContent || !profileDrawer) {
        console.warn('Drawer elements not found');
        return;
    }

    mobileNav.classList.remove('active');
    headerContent.classList.remove('collapsed');
    profileDrawer.classList.remove('active');
    drawerState.isOpen = false;
    document.body.style.overflow = '';
}

/**
 * Sets up the mobile navigation drawer functionality
 * 
 * This function:
 * 1. Sets up profile drawer click handler
 * 2. Sets up navigation link click handlers
 * 3. Sets up outside click handler to close drawer
 * 
 * @returns {void}
 */
function setupMobileNavigation() {
    if (!profileDrawer) {
        console.warn('Profile drawer element not found');
        return;
    }

    // Profile drawer click handler
    profileDrawer.addEventListener('click', () => {
        if (drawerState.isOpen) {
            closeDrawer();
        } else {
            openDrawer();
        }
    });

    // Navigation link click handlers
    setupNavigationLinks();

    // Outside click handler
    setupOutsideClickHandler();
}

/**
 * Sets up navigation link click handlers
 * 
 * @returns {void}
 */
function setupNavigationLinks() {
    if (!mobileNav) return;

    const navLinks = mobileNav.querySelectorAll('a');

    navLinks.forEach(link => {
        // Close drawer when clicking navigation links
        link.addEventListener('click', () => {
            closeDrawer();
        });

        // Smooth scrolling for navigation links
        link.addEventListener('click', (e) => {
            e.preventDefault();
            const targetId = link.getAttribute('href')?.substring(1) || '';
            const targetSection = document.getElementById(targetId);

            if (targetSection && header) {
                const headerHeight = header.offsetHeight;
                const targetPosition = targetSection.offsetTop - headerHeight - 20;

                window.scrollTo({
                    top: targetPosition,
                    behavior: 'smooth'
                });
            }
        });
    });
}

/**
 * Sets up outside click handler to close drawer
 * 
 * @returns {void}
 */
function setupOutsideClickHandler() {
    document.addEventListener('click', (e) => {
        const target = e.target;

        if (drawerState.isOpen &&
            mobileNav &&
            !mobileNav.contains(target) &&
            profileDrawer &&
            !profileDrawer.contains(target)) {
            closeDrawer();
        }
    });
}

// ============================================================================
// SCROLL FUNCTIONALITY
// ============================================================================

/**
 * Scroll state management
 * @type {Object}
 */
const scrollState = {
    lastScrollTop: 0,
    header: header
};

/**
 * Sets up scroll to top functionality
 * 
 * @returns {void}
 */
function setupScrollToTop() {
    if (!scrollToTop) {
        console.warn('Scroll to top element not found');
        return;
    }

    scrollToTop.addEventListener('click', () => {
        window.scrollTo({
            top: 0,
            behavior: 'smooth'
        });
    });
}

/**
 * Sets up mobile scroll behavior for header hiding/showing
 * 
 * @returns {void}
 */
function setupMobileScrollBehavior() {
    if (!header) {
        console.warn('Header element not found');
        return;
    }

    window.addEventListener('scroll', () => {
        const scrollTop = window.pageYOffset || document.documentElement.scrollTop;

        // Hide/show header on scroll for mobile
        if (window.innerWidth <= 768) {
            if (scrollTop > scrollState.lastScrollTop && scrollTop > 100) {
                // Scrolling down - hide header
                header.style.transform = 'translateY(-100%)';
            } else {
                // Scrolling up - show header
                header.style.transform = 'translateY(0)';
            }
        }

        scrollState.lastScrollTop = scrollTop;
    });
}

// ============================================================================
// MOBILE OPTIMIZATIONS
// ============================================================================

/**
 * Sets up mobile-specific touch feedback
 * 
 * @returns {void}
 */
function setupMobileTouchFeedback() {
    const touchElements = document.querySelectorAll('.mdc-button, .pub-links a, .mobile-nav a');

    touchElements.forEach(element => {
        element.addEventListener('touchstart', () => {
            element.style.transform = 'scale(0.95)';
        });

        element.addEventListener('touchend', () => {
            setTimeout(() => {
                element.style.transform = '';
            }, 150);
        });
    });
}

/**
 * Updates mobile CSS classes based on screen size
 * 
 * @returns {void}
 */
function updateMobileClasses() {
    if (window.innerWidth <= 768) {
        document.body.classList.add('mobile');
    } else {
        document.body.classList.remove('mobile');
    }
}

/**
 * Sets up mobile class updates on window resize
 * 
 * @returns {void}
 */
function setupMobileClassUpdates() {
    // Initialize mobile classes
    updateMobileClasses();

    // Update mobile classes on resize
    window.addEventListener('resize', updateMobileClasses);
}

// ============================================================================
// SERVICE WORKER
// ============================================================================

/**
 * Registers service worker for mobile performance optimization
 * 
 * @returns {void}
 */
function registerServiceWorker() {
    if ('serviceWorker' in navigator) {
        window.addEventListener('load', () => {
            navigator.serviceWorker.register('/sw.js').catch(() => {
                // Service worker registration failed - this is expected in development
                console.log('Service worker registration failed (expected in development)');
            });
        });
    }
}

// ============================================================================
// INITIALIZATION
// ============================================================================

/**
 * Main initialization function
 * 
 * This function sets up all the interactive features of the website:
 * - Theme system
 * - Mobile navigation
 * - Scroll behaviors
 * - Mobile optimizations
 * - Service worker
 * 
 * @returns {void}
 */
function initializeWebsite() {
    // Initialize theme system
    initializeTheme();

    // Set up mobile navigation
    setupMobileNavigation();

    // Set up scroll functionality
    setupScrollToTop();
    setupMobileScrollBehavior();

    // Set up mobile optimizations
    setupMobileTouchFeedback();
    setupMobileClassUpdates();

    // Register service worker
    registerServiceWorker();

    // Add loading animation for mobile
    window.addEventListener('load', () => {
        document.body.classList.add('loaded');
    });

    console.log('Academic website initialized successfully');
}

// ============================================================================
// MODULE EXPORTS (for potential future modularization)
// ============================================================================

// Export functions for potential future use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        initializeWebsite,
        initializeTheme,
        applyTheme,
        openDrawer,
        closeDrawer,
        setupMobileNavigation,
        setupScrollToTop,
        setupMobileScrollBehavior,
        updateMobileClasses
    };
}

// Initialize the website when the DOM is ready
document.addEventListener('DOMContentLoaded', initializeWebsite); 