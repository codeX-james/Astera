'use client';

export function getStoredLocale(): string | null {
  if (typeof window === 'undefined') return null;
  return localStorage.getItem('astera-locale');
}

export function setStoredLocale(locale: string) {
  if (typeof window === 'undefined') return;
  localStorage.setItem('astera-locale', locale);
  // Set cookie for next-intl middleware
  document.cookie = `NEXT_LOCALE=${locale}; path=/; max-age=31536000`;
}
