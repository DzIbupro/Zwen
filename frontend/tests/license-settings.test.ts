import {flushSync, mount, unmount} from 'svelte';
import {expect, it, vi} from 'vitest';
import LicenseSettings from '../LicenseSettings.svelte';
import {getLocale, setLocale} from '../lib/i18n.svelte';

vi.mock('svelte', () => vi.importActual<typeof import('svelte')>('../../node_modules/svelte/src/index-client.js'));

it('finds the bundled font license and shows its offline text', async () => {
  const locale = getLocale();
  setLocale('en');
  const target = document.createElement('div');
  document.body.append(target);
  const app = mount(LicenseSettings, {target});

  try {
    flushSync();
    const disclosure = target.querySelector('details')!;
    expect(disclosure.open).toBe(false);
    expect(disclosure.querySelector('summary')?.textContent).toContain('Third-party licenses');
    disclosure.open = true;
    disclosure.dispatchEvent(new Event('toggle'));
    await vi.waitFor(() => expect(target.querySelector('input[type=search]')).not.toBeNull());

    const search = target.querySelector<HTMLInputElement>('input[type=search]')!;
    search.value = 'OFL-1.1';
    search.dispatchEvent(new Event('input', {bubbles: true}));
    flushSync();
    const results = target.querySelectorAll<HTMLDetailsElement>('li > details');
    expect(results).toHaveLength(1);
    expect(results[0].textContent).toContain('KaTeX fonts');
    results[0].open = true;
    results[0].dispatchEvent(new Event('toggle'));
    await vi.waitFor(() => {
      const texts = [...results[0].querySelectorAll('pre')].map(notice => notice.textContent);
      expect(texts.some(text => text?.includes('SIL OPEN FONT LICENSE'))).toBe(true);
      expect(texts.some(text => text?.includes('Design Science'))).toBe(true);
    });

    search.value = 'unmatched-license-name';
    search.dispatchEvent(new Event('input', {bubbles: true}));
    flushSync();
    expect(target.textContent).toContain('No matching components');
  } finally {
    await unmount(app);
    target.remove();
    setLocale(locale);
  }
});
