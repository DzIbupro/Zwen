import { Marked, type Tokens } from 'marked';
import DOMPurify from 'dompurify';
import katex from 'katex';
import { proseLinkClick } from './links';

function escape(text: string): string {
  return text.replace(/[&<>"']/g, c=>({ '&':'&amp;', '<':'&lt;', '>':'&gt;', '"':'&quot;', "'":'&#39;' })[c]!);
}
function math(text: string, displayMode: boolean): string {
  if (text.length > 4096) return `<code class="math-error">${escape(text)}</code>`;
  try {
    return katex.renderToString(text, { displayMode, throwOnError: true, trust: false, maxExpand: 1000, maxSize: 20 });
  } catch {
    return `<code class="math-error" title="公式暂无法排版，保留源文">${escape(text)}</code>`;
  }
}
const marked = new Marked({ gfm: true, breaks: false, renderer: { html: token=>escape(token.text) }, extensions: [
  { name: 'blockMath', level: 'block', start: src=>src.indexOf('$$'),
    tokenizer(src) { const m=/^\$\$[ \t]*\n?([\s\S]+?)\$\$(?:[ \t]*\n|$)/.exec(src); if(m) return {type:'blockMath',raw:m[0],text:m[1]}; },
    renderer(token: Tokens.Generic) { return math(String(token.text), true); } },
  { name: 'inlineMath', level: 'inline', start: src=>src.indexOf('$'),
    tokenizer(src) { const m=/^\$(?!\$)((?:\\.|[^$\\\n])+)\$/.exec(src); if(m) return {type:'inlineMath',raw:m[0],text:m[1]}; },
    renderer(token: Tokens.Generic) { return math(String(token.text), false); } },
] });

function enhanceCodeBlocks(html: string): string {
  return html.replace(/<pre><code(?:\s+class="language-([A-Za-z0-9_+-]+)")?>([\s\S]*?)<\/code><\/pre>/g, (_match, lang: string | undefined, body: string) => {
    const label = lang ? escape(lang) : 'code';
    const classAttr = lang ? ` class="language-${escape(lang)}"` : '';
    return `<div class="code-block-wrap"><div class="code-block-head"><span class="code-lang">${label}</span><button type="button" class="code-copy-btn" data-copy-code aria-label="Copy code">Copy</button></div><pre><code${classAttr}>${body}</code></pre></div>`;
  });
}

export function renderMarkdown(text: string): string {
  const limited = text.length > 65536;
  const source = limited ? text.slice(0, 65536) : text;
  const html = marked.parse(source, { async: false });
  // Remote images are intentionally inert until the attachment workflow defines local asset scope.
  const clean = DOMPurify.sanitize(html, {
    USE_PROFILES: { html: true, mathMl: true },
    FORBID_TAGS: ['script','style','iframe','object','embed','form','input','button','img','video','audio'],
    FORBID_ATTR: ['id','name'],
  });
  return enhanceCodeBlocks(clean) + (limited ? '<p class="preview-limit">预览仅显示前 64K 字符，完整草稿仍保留在输入框。</p>' : '');
}

export async function copyTextToClipboard(text: string): Promise<boolean> {
  if (!text) return false;
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }
  } catch {
    // Fallback below
  }
  try {
    const ta = document.createElement('textarea');
    ta.value = text;
    ta.style.position = 'fixed';
    ta.style.opacity = '0';
    document.body.append(ta);
    ta.select();
    const ok = document.execCommand('copy');
    ta.remove();
    return ok;
  } catch {
    return false;
  }
}

export function handleProseClick(event: MouseEvent, openExternal: (url: string) => Promise<void>) {
  const target = event.target instanceof Element ? event.target : null;
  const copyBtn = target?.closest<HTMLButtonElement>('button[data-copy-code]');
  if (copyBtn) {
    event.preventDefault();
    event.stopPropagation();
    const codeText = copyBtn.closest('.code-block-wrap')?.querySelector('pre code')?.textContent ?? '';
    void copyTextToClipboard(codeText).then(ok => {
      if (!ok) return;
      copyBtn.textContent = '✓';
      copyBtn.dataset.copied = 'true';
      setTimeout(() => {
        copyBtn.textContent = 'Copy';
        delete copyBtn.dataset.copied;
      }, 1500);
    });
    return;
  }
  proseLinkClick(event, openExternal);
}
