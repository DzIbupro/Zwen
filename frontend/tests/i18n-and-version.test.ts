import { describe, it, expect, beforeEach } from 'vitest';
import { LOCALE_STORAGE_KEY, LOCALES, getLocale, setLocale, t } from '../lib/i18n.svelte';
import { APP_VERSION, GITHUB_REPO_URL } from '../lib/bridge';
import { defaultQuickActions, localizePresetQuickActions } from '../lib/quick-actions';

describe('i18n and versioning', () => {
  beforeEach(() => {
    localStorage.clear();
    setLocale('zh-CN');
  });

  it('defaults to zh-CN and switches reactively across all supported locales', () => {
    expect(getLocale()).toBe('zh-CN');
    expect(t('nav.settings')).toBe('设置');
    expect(LOCALES.map(l => l.id)).toEqual(['zh-CN', 'en', 'ja', 'ko', 'es']);

    setLocale('en');
    expect(getLocale()).toBe('en');
    expect(document.documentElement.lang).toBe('en');
    expect(localStorage.getItem(LOCALE_STORAGE_KEY)).toBe('en');
    expect(t('nav.settings')).toBe('Settings');
    expect(t('general.upToDate', { version: '0.0.1' })).toBe('You are up to date (v0.0.1)');

    setLocale('ja');
    expect(t('nav.settings')).toBe('設定');

    setLocale('ko');
    expect(t('nav.settings')).toBe('설정');

    setLocale('es');
    expect(t('nav.settings')).toBe('Ajustes');

    setLocale('zh-CN');
    expect(t('nav.settings')).toBe('设置');
  });

  it('uses 0.0.x initial debug versioning and points to the GitHub repo', () => {
    expect(APP_VERSION).toMatch(/^0\.0\.\d+$/);
    expect(APP_VERSION).toBe('0.0.2');
    expect(GITHUB_REPO_URL).toBe('https://github.com/DzIbupro/Zwen');
  });

  it('localizes preset quick action prompts when language changes while preserving custom user prompts', () => {
    const initial = defaultQuickActions('zh-CN');
    expect(initial[0].name).toBe('翻译');

    const toEn = localizePresetQuickActions(initial, 'en');
    expect(toEn.changed).toBe(true);
    expect(toEn.actions[0].name).toBe('Translate');
    expect(toEn.actions[0].prompt).toContain('translate the following selected text into English');
    expect(toEn.actions[1].name).toBe('Explain');

    // User customizes explain prompt, then switches to Japanese
    const customized = toEn.actions.map(a =>
      a.id === 'explain' ? { ...a, prompt: 'My custom prompt: {{selection}}' } : a
    );
    const toJa = localizePresetQuickActions(customized, 'ja');
    expect(toJa.changed).toBe(true);
    expect(toJa.actions[0].name).toBe('翻訳');
    expect(toJa.actions[0].prompt).toContain('日本語に翻訳');
    // Custom prompt is preserved
    expect(toJa.actions[1].prompt).toBe('My custom prompt: {{selection}}');
  });
});
