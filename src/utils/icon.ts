import { resolveResource } from '@tauri-apps/api/path';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import { invoke } from '@tauri-apps/api/tauri';
import { FALLBACK_ICON_SYMBOL, icons } from '../cache';

export const getIcon = async (filePath: string) => {
  try {
    // Always try to get fresh icon path for the current filePath
    const iconPath = await invoke<string>('extract_icon_path_from_desktop', {
      filePath,
    });

    // If we have a valid icon path, convert it and return
    if (iconPath && iconPath.length > 0) {
      const icon = convertFileSrc(iconPath);
      return {
        icon,
        fallbackIcon: await getFallbackIcon(),
      };
    }

    // If no icon path, return fallback for both
    const fallbackIcon = await getFallbackIcon();
    return {
      icon: fallbackIcon,
      fallbackIcon,
    };
  } catch (error) {
    console.error('Error fetching icon:', error);
    const fallbackIcon = await getFallbackIcon();
    return {
      icon: fallbackIcon,
      fallbackIcon,
    };
  }
};

// Helper function to get fallback icon
async function getFallbackIcon(): Promise<string> {
  try {
    // Check cache first
    let fallbackIcon = icons.get(FALLBACK_ICON_SYMBOL);
    if (fallbackIcon) {
      return fallbackIcon;
    }

    // If not in cache, resolve and cache it
    fallbackIcon = convertFileSrc(await resolveResource('assets/default.svg'));
    icons.set(FALLBACK_ICON_SYMBOL, fallbackIcon);
    return fallbackIcon;
  } catch (error) {
    console.error('Error getting fallback icon:', error);
    return ''; // Return empty string if even fallback fails
  }
}
