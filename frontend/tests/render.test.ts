import { it, expect } from 'vitest';
import { renderMarkdown } from '../lib/render';
function parsed(source:string){const d=document.createElement('div');d.innerHTML=renderMarkdown(source);return d;}
it('renders a real table and both kinds of formula',()=>{
 const d=parsed('| A | B |\n|---|---|\n| 1 | 2 |\n\n行内 $E=mc^2$\n\n$$\n\\sum_{i=1}^n i\n$$');
 expect(d.querySelector('td')?.textContent).toBe('1');expect(d.querySelectorAll('.katex').length).toBe(2);expect(d.querySelector('.katex-display')).not.toBeNull();
});
it('preserves dollar signs inside code',()=>{
 const d=parsed('`$HOME`\n\n```sh\necho "$HOME"\n```');expect(d.querySelector('pre code')?.textContent).toContain('$HOME');expect(d.querySelector('.katex')).toBeNull();
});
it('removes scripts, event handlers and unsafe links while retaining text',()=>{
 const d=parsed('<script>alert(1)</script><img src=x onerror="alert(1)">[危险](javascript:alert(1))\n\n安全文字');
 expect(d.querySelector('script')).toBeNull();expect(d.querySelector('[onerror]')).toBeNull();expect(d.querySelector('[href^="javascript:"]')).toBeNull();expect(d.querySelector('img')).toBeNull();expect(d.textContent).toContain('安全文字');
});
it('falls back locally for malformed or oversized math',()=>{
 const d=parsed('坏公式 $\\frac{$\n\n$'+ 'a'.repeat(5000)+'$\n\n后面的正文');expect(d.textContent).toContain('后面的正文');expect(d.querySelector('.math-error')).not.toBeNull();
});

it('does not allow raw HTML to overlay trusted application controls',()=>{
 const d=parsed('<div style="position:fixed;inset:0;z-index:9999">覆盖</div>');
 expect(d.querySelector('[style]')).toBeNull();expect(d.textContent).toContain('<div');
});
