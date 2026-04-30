---
name: SkillSym
description: A calm desktop management surface for publishing reusable AI coding skills across local tool folders.
colors:
  bg: "oklch(0.991 0.0015 248)"
  rail-bg: "oklch(0.982 0.004 248)"
  surface: "oklch(0.997 0.0015 248)"
  surface-muted: "oklch(0.966 0.006 250)"
  surface-active: "oklch(0.936 0.018 250)"
  line: "oklch(0.895 0.009 250)"
  line-strong: "oklch(0.61 0.027 248)"
  text: "oklch(0.49 0.01 248)"
  text-muted: "oklch(0.643 0.029 257)"
  accent: "oklch(0.753 0.162 66.6)"
  accent-hover: "oklch(0.69 0.15 66.6)"
  accent-point: "oklch(0.774 0.164 74.3)"
  accent-soft: "oklch(0.955 0.035 75)"
  accent-text: "oklch(0.991 0.0015 248)"
  danger-bg: "oklch(0.955 0.035 24)"
  danger-text: "oklch(0.58 0.16 24)"
  ok-bg: "oklch(0.945 0.04 137)"
  ok-text: "oklch(0.52 0.14 137)"
  dark-bg: "oklch(0.173 0.015 267)"
  dark-rail-bg: "oklch(0.158 0.014 267)"
  dark-surface: "oklch(0.191 0.018 264)"
  dark-surface-muted: "oklch(0.218 0.021 268)"
  dark-surface-active: "oklch(0.27 0.029 264)"
  dark-line: "oklch(0.264 0.018 268)"
  dark-text: "oklch(0.798 0.01 94)"
typography:
  headline:
    fontFamily: "Geist, Inter, -apple-system, BlinkMacSystemFont, system-ui, sans-serif"
    fontSize: "1.1rem"
    fontWeight: 600
    lineHeight: 1.16
    letterSpacing: "0"
  title:
    fontFamily: "Geist, Inter, -apple-system, BlinkMacSystemFont, system-ui, sans-serif"
    fontSize: "0.96rem"
    fontWeight: 500
    lineHeight: 1.2
    letterSpacing: "0"
  body:
    fontFamily: "Geist, Inter, -apple-system, BlinkMacSystemFont, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.42
    letterSpacing: "0"
  label:
    fontFamily: "Geist, Inter, -apple-system, BlinkMacSystemFont, system-ui, sans-serif"
    fontSize: "0.76rem"
    fontWeight: 500
    lineHeight: 1.3
    letterSpacing: "0"
rounded:
  segment: "4px"
  control: "5px"
  popover: "6px"
  pill: "999px"
spacing:
  xs: "3px"
  sm: "5px"
  md: "8px"
  lg: "12px"
  xl: "18px"
components:
  button-quiet:
    backgroundColor: "transparent"
    textColor: "{colors.text}"
    rounded: "{rounded.control}"
    padding: "3px 8px"
    height: "24px"
  button-primary:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.accent-text}"
    rounded: "{rounded.control}"
    padding: "3px 8px"
    height: "24px"
  segmented-control:
    backgroundColor: "{colors.surface-muted}"
    textColor: "{colors.text-muted}"
    rounded: "{rounded.control}"
    padding: "2px"
  nav-scope-active:
    backgroundColor: "{colors.surface-active}"
    textColor: "{colors.text}"
    rounded: "{rounded.control}"
    padding: "5px 6px 5px 28px"
  notification:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text}"
    rounded: "{rounded.popover}"
    padding: "5px 8px"
---

# Design System: SkillSym

## 1. Overview

**Creative North Star: "The Quiet Command Desk"**

SkillSym should feel like a precise desktop workbench: quiet enough to disappear, structured enough to make file state obvious, and restrained enough that users trust each action before they publish or remove anything. The system is a dense product UI, not a landing page, so clarity comes from columns, fine separators, subtle active states, and predictable controls.

The visual language is light by default because users are managing local folders in a daytime desktop workflow, often comparing paths and skill names in a normal IDE-adjacent environment. Dark mode exists for low-light use, but it keeps the same subdued hierarchy instead of becoming a separate dramatic theme.

The system explicitly rejects dirty greens, decorative gradients, visible texture, heavy shadows, crowded cards, and attention-seeking motion. Any new screen must preserve the current operating-room calm: text and file locations first, controls second, decoration last.

**Key Characteristics:**
- Dense but readable desktop layout with a fixed rail and task-first content pane.
- Restrained OKLCH palette: tinted neutrals plus a rare brass accent.
- Low elevation by default, with shadows reserved for transient overlays and notices.
- System-sans typography tuned for Chinese and English labels at compact sizes.
- Motion communicates state changes only, using 140-240ms ease-out curves.

## 2. Colors

The palette is cool porcelain and graphite with a small brass signal color for selection, primary action, and publish state.

### Primary
- **Brass Pin** (`accent`, `accent-point`): Use for the primary action, active publish dots, focus accents, and selected scope markers. It must remain rare so state remains legible.
- **Soft Brass Wash** (`accent-soft`): Use only when a larger area needs a warm state tint. Do not use it as page decoration.

### Secondary
- **Success Leaf** (`ok-bg`, `ok-text`): Use only for successful notices and confirmed operational state.
- **Delete Red** (`danger-bg`, `danger-text`): Use only for destructive actions, error notices, and delete affordances.

### Neutral
- **Porcelain Worktop** (`bg`): The page background and main workspace floor.
- **Rail Mist** (`rail-bg`): The left rail and settings side navigation surface.
- **Sheet Surface** (`surface`): Headers, list shells, active toggles, and overlay panels.
- **Quiet Fill** (`surface-muted`): Section headers, quiet buttons, icons, cards, and metric cells.
- **Selected Wash** (`surface-active`): Active scope cards, active rows, and selected tabs.
- **Hairline Graphite** (`line`, `line-strong`): Separators, subtle strokes, tree connectors, and scroll affordances.
- **Cool Graphite Text** (`text`, `text-muted`): Primary labels, paths, secondary metadata, and empty states.
- **Night Desk Set** (`dark-*` tokens): Dark mode mirrors the light hierarchy with cooler low-light surfaces and the same brass accent family.

### Named Rules
**The Rare Brass Rule.** Brass is a state signal, not a brand wash. Keep it under 10% of any screen and never use it as background decoration.

**The Porcelain Neutral Rule.** Backgrounds are always lightly tinted neutrals. Pure black and pure white are forbidden.

## 3. Typography

**Display Font:** None. Product screens do not use display type.
**Body Font:** Geist, Inter, -apple-system, BlinkMacSystemFont, system-ui, sans-serif.
**Label/Mono Font:** None. Paths use the same sans stack for continuity.

**Character:** The type system is compact, native, and operational. Weight changes do the work: 400 for content, 500 for labels and active controls, 600 for screen titles.

### Hierarchy
- **Headline** (600, `1.1rem`, `1.16`): Use for top-level screen titles such as Settings.
- **Title** (500, `0.96rem`, `1.2`): Use for panel titles and major group headers.
- **Body** (400, `14px`, `1.42`): Use for skill names, paths, table content, and ordinary controls. Cap prose at 65-75ch, but allow file paths and data rows to wrap as needed.
- **Label** (500, `0.76rem`, `1.3`, letter spacing `0`): Use for metadata, section kickers, table headers, pills, and compact button text.

### Named Rules
**The No Display Type Rule.** Labels, buttons, file paths, and data never use display fonts or decorative letter spacing.

**The Weight-Only Emphasis Rule.** Emphasis is created with weight, color, and placement. Do not add oversized text inside compact panels.

## 4. Elevation

SkillSym is flat at rest. Depth is conveyed through tonal surfaces, hairline separators, sticky headers, and scroll fades. Real shadows appear only for floating notices and workspace menus, where the element must separate from the rail or content pane.

### Shadow Vocabulary
- **Overlay Lift** (`0 8px 18px color-mix(in oklch, var(--shadow) 74%, transparent)`): Use for workspace menus and compact popovers.
- **Notice Lift** (`0 6px 16px color-mix(in oklch, var(--shadow) 72%, transparent)`): Use for transient status and error notices.
- **Inset Separator** (`inset 0 -1px color-mix(in oklch, var(--line) 68-70%, transparent)`): Use for sticky headers and surface boundaries without creating card elevation.

### Named Rules
**The Flat-Until-Floating Rule.** Static panels, rows, cards, tabs, and buttons do not cast shadows. Only overlays and notices may lift.

## 5. Components

### Buttons
- **Shape:** Quiet rounded rectangles (5px radius), compact height (`20-30px`) and low vertical padding.
- **Primary:** Brass fill (`accent`) with porcelain text (`accent-text`), used for final or explicit actions such as checking updates.
- **Hover / Focus:** Hover uses `control-bg-hover`; focus uses a 2px OKLCH outline mixed from accent and surface. Active press moves down by 1px.
- **Secondary / Ghost:** Default buttons are transparent at rest, with a muted tonal fill on hover. Icon buttons stay square and fixed-size.

### Chips
- **Style:** Pills use muted surface fill, muted text, 999px radius, and compact `2px 5px` padding.
- **State:** Selected or protected state should change fill and text only. Do not add side stripes or heavy borders.

### Cards / Containers
- **Corner Style:** Most containers are unframed. Where a contained unit is necessary, use 5-6px radius.
- **Background:** Static panels use `bg`, `rail-bg`, `surface`, and `surface-muted` rather than floating card stacks.
- **Shadow Strategy:** No shadows at rest. Use the Elevation vocabulary only for menus and notices.
- **Border:** Prefer 1px hairlines or inset separators. Dashed borders are reserved for empty states.
- **Internal Padding:** Desktop panels use 12-18px outer padding; dense rows use 4-8px internal rhythm.

### Inputs / Fields
- **Style:** The current UI uses button-like selectors and folder pickers rather than text fields. Future fields should use `surface` background, 1px `line` border, 5px radius, and compact height.
- **Focus:** Use the same 2px accent-mixed focus outline as buttons.
- **Error / Disabled:** Disabled controls use opacity `0.55`; errors use `danger-bg` and `danger-text`.

### Navigation
- **Style, typography, default/hover/active states, mobile treatment.** The app uses a fixed left rail on desktop with 308px width, 296px under 720px. Scope cards are transparent at rest, fill with a muted surface on hover, and use `surface-active` plus a small brass dot when selected. Settings uses a side tab list on desktop and horizontal tabs on mobile.

### Skill Matrix
The skill matrix is the signature working surface. Rows are dense, table-like, and separated by hairlines. Target actions are 26px icon buttons with a small brass active dot at the bottom-right, making publish state visible without turning the whole row into an alert.

### Notices
Notices float at the top-right with 6px radius, a 1px border, and a small status dot. They auto-dismiss and animate with opacity plus vertical motion only. Success and danger notices use semantic tints, not saturated banners.

## 6. Do's and Don'ts

### Do:
- **Do** keep primary content first: skill names, scope paths, source location, publish mode, and target state must be faster to scan than surrounding controls.
- **Do** use OKLCH tokens and `color-mix()` for hover, focus, and status blends.
- **Do** keep controls compact: 5px radius, 20-30px height, and fixed icon button dimensions.
- **Do** use fine separators, tonal panels, sticky headers, and scroll fades to define structure.
- **Do** keep motion between 140ms and 240ms, using ease-out quart, quint, or expo curves.
- **Do** preserve both light and dark theme token parity when adding new surfaces.

### Don't:
- **Don't** use dirty greens, decorative gradients, visible texture, heavy shadows, crowded cards, or attention-seeking motion.
- **Don't** use pure black, pure white, glassmorphism, gradient text, or decorative color washes.
- **Don't** use `border-left` or `border-right` greater than 1px as a colored stripe on cards, rows, callouts, or alerts.
- **Don't** add nested cards. Use full-width bands, separators, or table-like rows instead.
- **Don't** make inactive states saturated. Accent color belongs to action, focus, selection, and status.
- **Don't** animate layout properties or create page-load choreography. Product UI loads into the task.
