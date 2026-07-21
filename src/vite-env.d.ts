/// <reference types="vite/client" />

declare module "*.scss" {
  const content: { [className: string]: string };
  export default content;
}

interface ImportMetaEnv {
  readonly TAURI_DEV_HOST?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

declare module "*.jpg" {
  const src: string;
  export default src;
}
declare module "*.png" {
  const src: string;
  export default src;
}
declare module "*.webp" {
  const src: string;
  export default src;
}
