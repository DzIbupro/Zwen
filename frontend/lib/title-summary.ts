/**
 * Pure deterministic conversation title summarizer (no extra LLM call needed).
 * Handles quick-action selection prompts (translate/explain), paper reading prompts,
 * attached file/code blocks, and conversational filler prefixes.
 */
const FILLER_PREFIXES = [
  /^(?:你好|您好|嗨|哈喽|请问一下|请问|请帮我|请帮忙|帮我|帮忙|麻烦你|麻烦帮我|麻烦|我想问一下|我想问|我想知道|能否帮我|能不能帮我|可以帮我|能不能|可以)[，,、：:\s]*/u,
  /^(?:给我|帮我|请)?(?:详细|简单|通俗地|深入)?(?:讲一下|讲讲|介绍一下|介绍下|解释一下|解释下|说说|聊聊)[，,、：:\s]*/u,
  /^(?:什么是|何谓|请问什么是)[，,、：:\s]*/u,
  /^(?:hi|hello|hey|please|can you|could you|would you|help me|i want to know|tell me about|tell me)[,\s:]+/i,
];

const QUICK_ACTION_PATTERNS: readonly { regex: RegExp; prefix: string }[] = [
  { regex: /^请(?:将|把)?以下(?:选中)?(?:文字|内容|文本).*?翻译.*$/u, prefix: '翻译：' },
  { regex: /^请(?:详细|深入|简单)?解释以下(?:选中)?(?:文字|内容|文本).*$/u, prefix: '解释：' },
  { regex: /^请(?:总结|概括|提炼)以下(?:选中)?(?:文字|内容|文本).*$/u, prefix: '总结：' },
  { regex: /^请(?:润色|改写|优化|整理)以下(?:选中)?(?:文字|内容|文本).*$/u, prefix: '整理：' },
  { regex: /^Please translate the following selected text.*$/i, prefix: 'Translate: ' },
  { regex: /^Please explain the following selected text.*$/i, prefix: 'Explain: ' },
  { regex: /^Please organize the following selected text.*$/i, prefix: 'Doc: ' },
  { regex: /^以下の選択されたテキストを.*翻訳してください.*$/u, prefix: '翻訳：' },
  { regex: /^以下の選択されたテキストについて.*解説してください.*$/u, prefix: '解説：' },
  { regex: /^다음 선택한 텍스트를.*번역해 주세요.*$/u, prefix: '번역: ' },
  { regex: /^다음 선택한 텍스트를.*설명해 주세요.*$/u, prefix: '설명: ' },
  { regex: /^Por favor, traduce el siguiente texto seleccionado.*$/i, prefix: 'Traducir: ' },
  { regex: /^Por favor, explica detalladamente el siguiente texto seleccionado.*$/i, prefix: 'Explicar: ' },
];

function visualLength(str: string): number {
  let len = 0;
  for (const ch of str) {
    len += ch.charCodeAt(0) > 0x2e80 ? 2 : 1;
  }
  return len;
}

function truncateVisual(str: string, maxUnits: number): string {
  let units = 0;
  let out = '';
  for (const ch of str) {
    const w = ch.charCodeAt(0) > 0x2e80 ? 2 : 1;
    if (units + w > maxUnits) {
      return `${out.trimEnd()}…`;
    }
    units += w;
    out += ch;
  }
  return out;
}

function cleanPaperName(rawName: string): string {
  return rawName
    .trim()
    .replace(/\.pdf$/i, '')
    // Strip leading numeric index like "01_", "02-", "10 "
    .replace(/^\d{1,3}[_\-\s]+/, '')
    // Replace underscores with spaces
    .replace(/_+/g, ' ')
    // Insert space between lowercase-to-uppercase CamelCase boundaries (e.g. WhySayIt -> Why Say It)
    .replace(/([a-z])([A-Z])/g, '$1 $2')
    .replace(/\s+/g, ' ')
    .trim();
}

function cleanClause(rawLine: string, maxUnits: number): string {
  let candidate = rawLine
    .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')
    .replace(/^[#>*\-\d.\s]+/, '')
    .replace(/[`*~]+/g, '')
    .replace(/^[“"‘'「『]+|[”"’'」』]+$/g, '')
    .replace(/\s+/g, ' ')
    .trim();

  for (let pass = 0; pass < 2; pass++) {
    for (const pattern of FILLER_PREFIXES) {
      const stripped = candidate.replace(pattern, '').trim();
      if (stripped.length >= 2) {
        candidate = stripped;
      }
    }
  }

  const sentenceMatch = candidate.match(/^(.+?)(?:[。？！；]|[.?!;](?:\s|$))/u);
  if (sentenceMatch && visualLength(sentenceMatch[1]) >= 4 && visualLength(sentenceMatch[1]) <= maxUnits) {
    candidate = sentenceMatch[1].trim();
  }

  candidate = candidate.replace(/[，,、：:；;。？！?!]+$/u, '').trim();
  return candidate;
}

export function summarizeTitle(raw: string, maxUnits = 52): string {
  const source = (raw ?? '').trim();
  if (!source) return '未命名对话';

  // 1. Handle Paper Reading prompts ("论文阅读 · xxx.pdf\n...")
  const paperMatch = source.match(/^论文阅读\s*[·•:]\s*([^\r\n]+)/u);
  if (paperMatch?.[1]) {
    const cleanedPaper = cleanPaperName(paperMatch[1]);
    const restLines = source
      .slice(paperMatch[0].length)
      .split(/\r?\n/)
      .map(l => l.trim())
      .filter(l => l && !l.startsWith('请按阅读要求分析这篇论文') && !l.startsWith('阅读模式：') && !/^\d+\.\s/.test(l));
    if (restLines[0]) {
      const follow = cleanClause(restLines[0], 24);
      if (follow) {
        return truncateVisual(`${cleanedPaper} · ${follow}`, maxUnits);
      }
    }
    return truncateVisual(`论文：${cleanedPaper}`, maxUnits);
  }

  // 2. Detect attached file blocks like "[文件: foo.ts]\n```...```"
  const fileMatch = source.match(/\[文件:\s*([^\]]+)\]/);
  const textWithoutCode = source
    .replace(/```[\s\S]*?```/g, ' ')
    .replace(/\[文件:\s*[^\]]+\]/g, ' ');

  const lines = textWithoutCode
    .split(/\r?\n/)
    .map(l => l.replace(/\s+/g, ' ').trim())
    .filter(Boolean);

  // 3. Handle Quick Action prompts (e.g. "请将以下选中文字翻译成中文...\n\n<selection>")
  if (lines.length >= 1) {
    for (const { regex, prefix } of QUICK_ACTION_PATTERNS) {
      if (regex.test(lines[0])) {
        const selectionLine = lines.slice(1).join(' ').trim();
        if (selectionLine) {
          const cleanedSelection = cleanClause(selectionLine, maxUnits - visualLength(prefix));
          if (cleanedSelection) {
            return truncateVisual(`${prefix}${cleanedSelection}`, maxUnits);
          }
        }
      }
    }
  }

  // 4. Standard prompt summarization
  let candidate = lines[0] ? cleanClause(lines[0], maxUnits) : '';

  if (!candidate) {
    if (fileMatch?.[1]) {
      candidate = `文件：${fileMatch[1].trim()}`;
    } else {
      candidate = source.replace(/\s+/g, ' ').trim();
    }
  }

  return truncateVisual(candidate, maxUnits);
}
