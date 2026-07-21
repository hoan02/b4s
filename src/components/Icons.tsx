/** Monochrome line icons — large, clean (Apple-style weight) */
import type { JSX } from "solid-js";

interface IconProps {
  size?: number;
  class?: string;
}

const s = (p: IconProps) => p.size ?? 28;

export const IconNormal = (p: IconProps) => (
  <svg class={p.class} width={s(p)} height={s(p)} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M12 3v18" />
    <path d="M8 8c2-2 6-2 8 0" />
    <path d="M7 12c2.5-2 7.5-2 10 0" />
    <path d="M6 16c3-2 9-2 12 0" />
  </svg>
);

export const IconAmbient = (p: IconProps) => (
  <svg class={p.class} width={s(p)} height={s(p)} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <circle cx="12" cy="12" r="3" />
    <path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" />
  </svg>
);

export const IconAnc = (p: IconProps) => (
  <svg class={p.class} width={s(p)} height={s(p)} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M12 3a9 9 0 0 0-9 9v2a3 3 0 0 0 3 3h1v-6a5 5 0 0 1 10 0v6h1a3 3 0 0 0 3-3v-2a9 9 0 0 0-9-9z" />
    <path d="M9 18v1a3 3 0 0 0 6 0v-1" />
  </svg>
);

export const IconGame = (p: IconProps) => (
  <svg class={p.class} width={s(p)} height={s(p)} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <rect x="2" y="7" width="20" height="12" rx="3" />
    <path d="M8 13h4M10 11v4" />
    <circle cx="16" cy="12" r="1" fill="currentColor" stroke="none" />
    <circle cx="18" cy="14" r="1" fill="currentColor" stroke="none" />
  </svg>
);

export const IconFind = (p: IconProps) => (
  <svg class={p.class} width={s(p)} height={s(p)} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <circle cx="11" cy="11" r="7" />
    <path d="M21 21l-4.3-4.3" />
    <path d="M11 8v6M8 11h6" />
  </svg>
);

export const IconEq = (p: IconProps) => (
  <svg class={p.class} width={s(p)} height={s(p)} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M4 20V10M12 20V4M20 20v-8" />
  </svg>
);

export const IconSpatial = (p: IconProps) => (
  <svg class={p.class} width={s(p)} height={s(p)} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <circle cx="12" cy="12" r="9" />
    <ellipse cx="12" cy="12" rx="4" ry="9" />
    <path d="M3 12h18" />
  </svg>
);

export const IconMore = (p: IconProps) => (
  <svg class={p.class} width={s(p)} height={s(p)} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <circle cx="5" cy="12" r="1.5" fill="currentColor" stroke="none" />
    <circle cx="12" cy="12" r="1.5" fill="currentColor" stroke="none" />
    <circle cx="19" cy="12" r="1.5" fill="currentColor" stroke="none" />
  </svg>
);

export const IconSettings = (p: IconProps) => (
  <svg class={p.class} width={s(p)} height={s(p)} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <circle cx="12" cy="12" r="3" />
    <path d="M12 1v2M12 21v2M4.2 4.2l1.4 1.4M18.4 18.4l1.4 1.4M1 12h2M21 12h2M4.2 19.8l1.4-1.4M18.4 5.6l1.4-1.4" />
    <path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V20a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H4a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H10a1.7 1.7 0 0 0 1-1.5V4a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V10c.3.6.9 1 1.5 1H20a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z" />
  </svg>
);

export const IconPower = (p: IconProps) => (
  <svg class={p.class} width={s(p)} height={s(p)} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M12 2v10" />
    <path d="M6.3 6.3a8 8 0 1 0 11.4 0" />
  </svg>
);

export const IconBattery = (p: IconProps) => (
  <svg class={p.class} width={s(p)} height={s(p)} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <rect x="2" y="7" width="18" height="10" rx="2" />
    <path d="M22 11v2" />
    <path d="M6 10v4M10 10v4M14 10v4" />
  </svg>
);

export const IconBack = (p: IconProps) => (
  <svg class={p.class} width={s(p)} height={s(p)} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M15 18l-6-6 6-6" />
  </svg>
);

export const IconGithub = (p: IconProps) => (
  <svg class={p.class} width={s(p)} height={s(p)} viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
    <path d="M12 2C6.48 2 2 6.58 2 12.24c0 4.53 2.87 8.37 6.84 9.73.5.1.68-.22.68-.49v-1.7c-2.78.62-3.37-1.23-3.37-1.23-.46-1.2-1.11-1.52-1.11-1.52-.91-.64.07-.63.07-.63 1.01.08 1.54 1.06 1.54 1.06.9 1.58 2.35 1.12 2.92.86.09-.67.35-1.12.64-1.38-2.22-.26-4.56-1.14-4.56-5.08 0-1.12.39-2.04 1.03-2.76-.1-.26-.45-1.31.1-2.73 0 0 .84-.27 2.76 1.05A9.3 9.3 0 0 1 12 6.8c.85 0 1.7.12 2.5.36 1.92-1.32 2.76-1.05 2.76-1.05.55 1.42.2 2.47.1 2.73.64.72 1.03 1.64 1.03 2.76 0 3.95-2.35 4.82-4.58 5.08.36.32.68.94.68 1.9v2.82c0 .27.18.59.69.49A10.24 10.24 0 0 0 22 12.24C22 6.58 17.52 2 12 2Z" />
  </svg>
);

export const IconTheme = (p: IconProps) => (
  <svg class={p.class} width={s(p)} height={s(p)} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <circle cx="12" cy="12" r="4" />
    <path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" />
  </svg>
);

export const IconOffice = (p: IconProps) => (
  <svg class={p.class} width={s(p)} height={s(p)} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M4 21V5a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v16M16 9h3a1 1 0 0 1 1 1v11M8 7h4M8 11h4M8 15h4M10 21v-3h2v3" />
  </svg>
);

export const IconOutdoor = (p: IconProps) => (
  <svg class={p.class} width={s(p)} height={s(p)} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <circle cx="17" cy="6" r="2.5" /><path d="M3 21l6-9 4 5 3-4 5 8M3 21h18" />
  </svg>
);

export const IconTransit = (p: IconProps) => (
  <svg class={p.class} width={s(p)} height={s(p)} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <rect x="4" y="3" width="16" height="14" rx="3" /><path d="M4 11h16M8 21l2-4M16 21l-2-4M8 7h.01M16 7h.01" />
  </svg>
);

export const IconFlight = (p: IconProps) => (
  <svg class={p.class} width={s(p)} height={s(p)} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M3 13l7-2 4-7 2 1-1 7 5 2c1 .4 1 1.8 0 2.2l-5 1.8 1 3-2 1-4-3-7 1v-2l6-2-6-2z" />
  </svg>
);

export type IconComp = (p: IconProps) => JSX.Element;
