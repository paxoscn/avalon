import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import Cookies from 'js-cookie';
import en from './locales/en.json';
import zh from './locales/zh.json';

const LANGUAGE_COOKIE_KEY = 'app_language';

i18n
  .use({
    type: 'languageDetector',
    async: false,
    init: () => {},
    detect: () => {
      // 1. Check cookie first
      const cookieLang = Cookies.get(LANGUAGE_COOKIE_KEY);
      if (cookieLang) {
        return cookieLang;
      }
      
      // 2. Check browser language
      const browserLang = navigator.language || (navigator as any).userLanguage;
      if (browserLang) {
        // Convert browser language to our supported languages
        if (browserLang.startsWith('zh')) {
          return 'zh';
        }
        return 'en';
      }
      
      // 3. Default to English
      return 'en';
    },
    cacheUserLanguage: (lng: string) => {
      Cookies.set(LANGUAGE_COOKIE_KEY, lng, { expires: 365 });
    },
  })
  .use(initReactI18next)
  .init({
    resources: {
      en: { translation: en },
      zh: { translation: zh },
    },
    fallbackLng: 'en',
    supportedLngs: ['en', 'zh'],
    interpolation: {
      escapeValue: false,
    },
  });

export default i18n;
export { LANGUAGE_COOKIE_KEY };
