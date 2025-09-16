import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatRustSnakeCaseToCamelCase(str: string) {
  return str.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
}

export function formatRustSnakeCaseToPascalCase(str: string) {
  return str.replace(/_/g, " ").replace(/\b\w/g, (char) => char.toUpperCase());
}

export function transformServerSettingsConfig(config: any): any {
  const transformed: any = {};

  for (const [key, value] of Object.entries(config)) {
    const camelKey = formatRustSnakeCaseToCamelCase(key);

    if (value && typeof value === "object" && !Array.isArray(value)) {
      // Recursively transform nested objects
      transformed[camelKey] = transformServerSettingsConfig(value);
    } else {
      transformed[camelKey] = value;
    }
  }

  return transformed;
}
