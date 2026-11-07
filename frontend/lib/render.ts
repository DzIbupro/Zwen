import { Marked, type Tokens } from 'marked';
import DOMPurify from 'dompurify';
import katex from 'katex';

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
  return clean + (limited ? '<p class="preview-limit">预览仅显示前 64K 字符，完整草稿仍保留在输入框。</p>' : '');
}
