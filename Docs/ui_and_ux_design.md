# Apple Human Interface Guidelines (HIG) & Design System

This comprehensive guide details the design philosophy, visual foundations, and interface patterns that define Apple's ecosystem. It is structured to help designers and developers build applications that feel distinctively native to iOS (iPhone) and macOS (Mac).

---

## 1. Core Philosophy
Apple’s design ethos rests on three pillars that differentiate it from other design systems (like Material Design).

### 1.1 Clarity
*   **Text Legibility**: Text size, weight, and color must prioritize readability.
*   **Iconography**: Icons should be precise and lucid, communicating meaning instantly.
*   **Negative Space**: Use ample whitespace to denote hierarchy and importance, avoiding clutter.
*   **Functionality**: Decoration should never overshadow functionality.

### 1.2 Deference
*   **Content First**: The UI should recede, allowing content to take center stage.
*   **Fluid Motion**: Motion explains relationships and transitions but shouldn't be gratuitous.
*   **Lightness**: Use translucent backgrounds and blurs to hint at context without obscuring content.

### 1.3 Depth
*   **Visual Layers**: Distinct visual layers and realistic motion convey hierarchy and position.
*   **Context**: Translucency provides a sense of place (knowing what is behind the current view).
*   **Interaction**: Touch and gestures should feel physical and responsive.

---

## 2. Visual Foundations

### 2.1 Typography
Apple uses the **San Francisco (SF)** family. It is a neo-grotesque sans-serif typeface designed for legibility.

#### Font Families
*   **SF Pro**: The system font for iOS, macOS, and tvOS.
    *   *SF Pro Display*: 20pt+. Used for titles and headers.
    *   *SF Pro Text*: <20pt. Optimized for body text with wider spacing.
*   **SF Mono**: Monospaced font for code, data, and technical numerals.
*   **SF Compact**: Flat-sided font optimized for Apple Watch.
*   **New York**: A companion serif font for reading-heavy contexts (Books, News).

#### iOS Dynamic Type Scale (Default)
Support Dynamic Type to let users adjust text size.

| Style | Weight | Size (pts) | Leading (pts) | Usage |
| :--- | :--- | :--- | :--- | :--- |
| **Large Title** | Regular | 34 | 41 | Top of a navigation stack (transitioning to inline). |
| **Title 1** | Regular | 28 | 34 | Primary page headings. |
| **Title 2** | Regular | 22 | 28 | Section headings. |
| **Title 3** | Regular | 20 | 25 | Subsection headings. |
| **Headline** | Semibold | 17 | 22 | Paragraph headers. |
| **Body** | Regular | 17 | 22 | Primary content text. |
| **Callout** | Regular | 16 | 21 | Featured text or pull quotes. |
| **Subhead** | Regular | 15 | 20 | Secondary metadata (e.g., list item descriptions). |
| **Footnote** | Regular | 13 | 18 | Timestamps, disclaimers. |
| **Caption 1** | Regular | 12 | 16 | Image captions, subtle labels. |
| **Caption 2** | Regular | 11 | 13 | Smallest legible text. |

### 2.2 Color System
Apple uses **Semantic Colors** that adapt automatically to Light and Dark modes. **Never hardcode hex values** if possible; use system tokens.

#### Tint Colors
*   **System Blue**: Default interactive elements (Buttons, Links).
*   **System Red**: Destructive actions (Delete, Remove).
*   **System Green**: Success states, "On" switches.
*   **System Orange/Yellow**: Warnings, attention.

#### Backgrounds (The Layering Model)
*   **System Background**: Pure White (Light) / Pure Black (Dark). Main canvas.
*   **Secondary Background**: Light Gray (Light) / Dark Gray (Dark). Used for Grouped Table Views.
*   **Tertiary Background**: Even lighter/darker. Used for nested grouping.

#### Grays (Neutrals)
Apple provides 6 semantic grays (`SystemGray` to `SystemGray6`).
*   **Light Mode**: Gray 6 is the lightest (near white).
*   **Dark Mode**: Gray 6 is the darkest (near black).
*   *Note*: This inversion ensures contrast is maintained.

#### Dark Mode Elevation
In Material Design, elevation is shown via lighter surfaces and shadows. In Apple Design:
*   **No Shadows**: Dark mode relies less on shadows.
*   **Elevation**: Elevation is often depicted by **lighter** background grays on top of darker ones.
*   **Vibrancy**: Translucent materials become more prominent to separate layers.

### 2.3 Materials (Glassmorphism)
Blur effects ("Materials") create a sense of depth and context.

*   **Ultra Thin Material**: Highly translucent (e.g., Spotlight Search).
*   **Thin Material**: Standard HUDs.
*   **Regular Material**: Default sidebar or tab bar background.
*   **Thick Material**: Nav bars, higher contrast needs.
*   **Chrome Material**: System chrome, almost opaque.

### 2.4 Iconography (SF Symbols)
A library of 5,000+ vector icons seamlessly integrated with text.

*   **Weights**: Match text weights (Ultralight to Black).
*   **Scales**: Small, Medium, Large (match line-height).
*   **Rendering Modes**:
    1.  **Monochrome**: Single color (tint).
    2.  **Hierarchical**: Single color with varying opacity (e.g., cloud is 100%, rain is 50%).
    3.  **Palette**: Two or more distinct colors.
    4.  **Multicolor**: Full intrinsic colors (e.g., a green leaf icon).

---

## 3. iOS Design Patterns (Mobile)

### 3.1 Layout & Grid
*   **8pt Grid**: Align elements to an 8pt grid.
*   **Margins**: Standard layout margins are **16pt** (iPhone SE/older) or **20pt** (modern iPhones).
*   **Touch Targets**: Minimum **44x44pt** tappable area.

### 3.2 Navigation Structure
*   **Tab Bar**: Flat, bottom navigation for parallel hierarchies (3-5 tabs).
*   **Navigation Bar**: Top bar showing title, Back button (left), and Action/Edit button (right).
    *   *Large Title*: Expands when at the top, collapses to *Inline Title* on scroll.
*   **Search**: Typically a search bar inside the Navigation Bar, often hidden under a swipe-down gesture.

### 3.3 UI Elements
*   **Lists (Table Views)**:
    *   *Plain*: Full width, dividers (separators) inset by 16pt/60pt.
    *   *Grouped*: Rounded rectangles sitting on a secondary background color.
*   **Cards**: Rounded rectangles (Corner radius: usually **12pt - 20pt**) used for distinct content blocks.
*   **Modals (Sheets)**:
    *   Views that slide up from the bottom, covering most of the screen.
    *   "Grabber" handle at the top indicating it can be swiped down to close.
    *   Used for tasks, settings, or details that don't require full navigation context switch.
*   **Action Sheets**: Slide up from bottom with 2+ choices (e.g., "Delete", "Cancel"). Destructive options are Red.

### 3.4 Inputs
*   **Text Fields**: Rounded corners, clear button on right.
*   **Pickers**: Scrolling wheel for selecting dates/values (distinctive iOS interaction).
*   **Segmented Controls**: Horizontal pill-shaped toggle for mutually exclusive options.
*   **Switches**: Green for "On", Gray for "Off". Immediate action.

---

## 4. macOS Design Patterns (Desktop)

### 4.1 Window Anatomy
*   **Sidebar**: Full-height, translucent (material) background on the left. Used for navigation.
*   **Toolbar**: Unifies the window title and controls. Icons are often glyph-style (borderless) until hovered.
*   **Content Area**: White/Black opaque background for the main task.

### 4.2 Controls & Interactions
*   **Point & Click**: Targets are smaller than iOS. No minimum 44pt rule, but hit-testing should be generous.
*   **Push Buttons**: Rounded rectangles with a gradient or solid fill. "Default" action is blue (return key triggers it).
*   **Context Menus**: Right-click menus. Essential for power user workflows.
*   **Checkboxes & Radio Buttons**: Standard desktop controls, unlike iOS toggles.
*   **Tooltips**: Essential for icon-only toolbar buttons.

### 4.3 App Icons
*   **Shape**: Uniform rounded rectangle (Squircle).
*   **Style**: Realistic, "physical" rendering. Often depicts tools (hammer, pen) or materials (paper, metal).
*   **Shadow**: A consistent drop shadow to imply the icon is sitting on a desk.

---

## 5. Motion & Animation

Apple's motion feels "physical" due to spring physics.

*   **Springs**: Do not use linear easing. Use spring parameters (Mass, Stiffness, Damping).
    *   *Interactive*: Motion tracks the finger 1:1 (e.g., swiping a card).
    *   *Impulse*: When the finger releases, the object continues with the momentum before settling.
*   **Transitions**:
    *   *Navigation Push*: New screen slides in from right, old slides out left (parallax).
    *   *Sheet Presentation*: Rear layer scales down slightly, new sheet slides up over it.
    *   *App Launch*: Icon expands into the app window (Zoom transition).

---

## 6. Accessibility (A11y)

Accessibility is a first-class citizen in Apple Design.

*   **VoiceOver**: All custom controls must have `accessibilityLabel` and `accessibilityHint`.
*   **Dynamic Type**: Apps must verify layouts don't break when users set font size to XXXL.
*   **Reduce Motion**: Respect the user's system setting to disable parallax/zooms (replace with simple fades).
*   **Contrast**: Ensure 4.5:1 contrast ratio for text, even with transparency.

---

## 7. Resources & Tools

*   **SF Symbols App**: Mac app to browse and export icon vectors.
*   **Apple Design Resources**: Official Figma/Sketch kits available at developer.apple.com.
*   **Human Interface Guidelines (HIG)**: The official web documentation.