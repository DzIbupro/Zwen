import {execFile} from 'node:child_process';
import {createHash} from 'node:crypto';
import {existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync} from 'node:fs';
import {dirname, join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {promisify} from 'node:util';
import {build} from 'vite';

const root = fileURLToPath(new URL('../', import.meta.url));
const dataDirectory = join(root, 'frontend/license-data');
const supplements = join(root, 'scripts/third-party-notices');
const target = 'x86_64-unknown-linux-gnu';
const noticeFilename = /^(?:LICEN[CS]E|UNLICENSE|COPYING|COPYRIGHT|NOTICE)(?:[._-]|$)/i;
const run = promisify(execFile);
const readText = path => readFileSync(path, 'utf8').replaceAll('\r\n', '\n').trimEnd();
const readJson = path => JSON.parse(readText(path));
const byKey = (a, b) => a < b ? -1 : a > b ? 1 : 0;

function packageNotices(directory, licenseFile) {
  const names = readdirSync(directory, {withFileTypes: true})
    .filter(entry => entry.isFile() && noticeFilename.test(entry.name))
    .map(entry => entry.name);
  if (licenseFile && !names.includes(licenseFile)) names.push(licenseFile);
  return names.sort(byKey).map(name => ({name, text: readText(join(directory, name))}));
}

async function frontendComponents() {
  const result = await build({
    root,
    logLevel: 'silent',
    build: {write: false},
    plugins: [{
      name: 'zwen-license-inventory',
      enforce: 'pre',
      resolveId(id, importer) {
        // Allow rebuilding the inventory from a clean checkout without generated files.
        if (id === './license-data/catalog.json?raw' && importer?.endsWith('/LicenseSettings.svelte')) {
          return '\0zwen-license-catalog';
        }
      },
      load(id) {
        if (id === '\0zwen-license-catalog') return 'export default "[]"';
      },
    }],
  });
  const packageDirectories = new Set();
  for (const output of [result].flat().flatMap(result => result.output)) {
    if (output.type !== 'chunk') continue;
    for (const [id, module] of Object.entries(output.modules)) {
      const packageModule = id.match(/^(.*\/node_modules\/(?:@[^/]+\/)?[^/]+)\//);
      if (module.renderedLength && packageModule) packageDirectories.add(packageModule[1]);
    }
  }
  return [...packageDirectories].map(directory => {
    const {name, version, license} = readJson(join(directory, 'package.json'));
    return {
      name, version, license,
      source: `https://www.npmjs.com/package/${name}/v/${version}`,
      notices: packageNotices(directory),
    };
  });
}

async function desktopComponents() {
  const {stdout} = await run('cargo', [
    'metadata', '--locked', '--offline', '--format-version', '1', '--filter-platform', target,
  ], {cwd: root, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024});
  const metadata = JSON.parse(stdout);
  const packages = new Map(metadata.packages.map(pkg => [pkg.id, pkg]));
  const graph = new Map(metadata.resolve.nodes.map(node => [node.id, node.deps]));
  const pending = [metadata.packages.find(pkg => pkg.name === 'zwen-desktop').id];
  const visited = new Set();
  const runtime = [];
  while (pending.length) {
    const id = pending.pop();
    if (visited.has(id)) continue;
    visited.add(id);
    const pkg = packages.get(id);
    if (pkg.targets.some(target => target.kind.includes('proc-macro'))) continue;
    for (const dependency of graph.get(id)) {
      if (dependency.dep_kinds.some(edge => edge.kind === null)) pending.push(dependency.pkg);
    }
    if (pkg.source) runtime.push(pkg);
  }
  const fromCrate = (name, version) => {
    // A workspace sibling can carry a common upstream license omitted from a packaged subdirectory.
    const pkg = metadata.packages.find(pkg => pkg.name === name && pkg.version === version);
    return packageNotices(dirname(pkg.manifest_path), pkg.license_file);
  };
  return runtime.map(pkg => {
    const {name, version, license} = pkg;
    let notices = packageNotices(dirname(pkg.manifest_path), pkg.license_file);
    if (!notices.length) {
      switch (`${name}@${version}`) {
        case 'alloc-stdlib@0.3.0':
          notices = fromCrate('alloc-no-stdlib', '3.0.0');
          break;
        case 'dlopen2@0.8.2':
          notices = [{name: 'LICENSE', text: readText(join(supplements, 'dlopen2-0.8.2-MIT.txt'))}];
          break;
        case 'libappindicator-sys@0.9.0':
          notices = fromCrate('libappindicator', version);
          break;
        case 'selectors@0.38.0':
          notices = [{name: 'MPL-2.0', text: readText(join(root, 'node_modules/dompurify/LICENSE-MPL'))}];
          break;
      }
    }
    return {name, version, license, source: `https://crates.io/crates/${name}/${version}`, notices};
  });
}

function fontLicense(path) {
  const data = readFileSync(path);
  for (let offset = 12; offset < 12 + 16 * data.readUInt16BE(4); offset += 16) {
    if (data.toString('ascii', offset, offset + 4) !== 'name') continue;
    const table = data.readUInt32BE(offset + 8);
    const count = data.readUInt16BE(table + 2);
    const strings = table + data.readUInt16BE(table + 4);
    for (let record = table + 6; record < table + 6 + 12 * count; record += 12) {
      const platform = data.readUInt16BE(record);
      const id = data.readUInt16BE(record + 6);
      if (platform !== 3 || id !== 13) continue;
      const length = data.readUInt16BE(record + 8);
      const start = strings + data.readUInt16BE(record + 10);
      return Buffer.from(data.subarray(start, start + length)).swap16().toString('utf16le').trimEnd();
    }
  }
  throw new Error(`Missing font license metadata in ${path}`);
}

function assetComponents() {
  const katex = join(root, 'node_modules/katex');
  const {version} = readJson(join(katex, 'package.json'));
  const fontDirectory = join(katex, 'dist/fonts');
  const fontNotices = new Set(readdirSync(fontDirectory).sort(byKey)
    .filter(name => name.endsWith('.ttf'))
    .map(name => fontLicense(join(fontDirectory, name))));
  if (!fontNotices.size || [...fontNotices].some(text => !text.includes('SIL Open Font License'))) {
    throw new Error('Review the current KaTeX font licenses');
  }
  const icons = join(root, 'frontend/assets/icons');
  const lucide = readText(join(icons, 'SOURCE.md')).match(/pinned tag ([\d.]+)\./)?.[1];
  if (!lucide) throw new Error('Missing pinned Lucide icon version');
  return [{
    name: 'KaTeX fonts', version, license: 'OFL-1.1',
    source: `https://github.com/KaTeX/KaTeX/tree/v${version}/src/fonts`,
    notices: [
      {name: 'Font copyright notices', text: [...fontNotices].join('\n\n')},
      {name: 'OFL-1.1', text: readText(join(supplements, 'OFL-1.1.txt'))},
    ],
  }, {
    name: 'Lucide icons', version: lucide, license: 'ISC AND MIT',
    source: `https://github.com/lucide-icons/lucide/tree/${lucide}/icons`,
    notices: [
      ...packageNotices(icons),
      {name: 'MIT (Feather portions)', text: readText(join(supplements, 'lucide-feather-MIT.txt'))},
    ],
  }];
}

const check = process.argv.includes('--check');
const [frontend, desktop] = await Promise.all([frontendComponents(), desktopComponents()]);
const components = [...frontend, ...desktop, ...assetComponents()]
  .sort((left, right) => byKey(left.name.toLowerCase(), right.name.toLowerCase()) || byKey(left.version, right.version));
const sources = new Set();
const textFiles = [{}, {}, {}];
const catalog = components.map(({notices, ...component}) => {
  if (!component.license || !notices.length || sources.has(component.source)) {
    throw new Error(`Review the source and license notices for ${component.name}@${component.version}`);
  }
  sources.add(component.source);
  return {...component, notices: notices.map(({name, text}) => {
    if (!text.trim()) throw new Error(`Empty ${name} for ${component.name}@${component.version}`);
    // Share identical notices, including copies whose only differences are line wrapping or spacing.
    const key = createHash('sha256').update(text.replace(/\s+/g, ' ').trim()).digest('hex').slice(0, 16);
    const file = textFiles[parseInt(key[0], 16) % textFiles.length];
    file[key] ??= text;
    return {name, key};
  })};
});

function output(filename, text) {
  const path = join(dataDirectory, filename);
  if (!check) {
    writeFileSync(path, text);
  } else if (!existsSync(path) || readFileSync(path, 'utf8') !== text) {
    console.error(`${filename} needs regeneration`);
    process.exitCode = 1;
  }
}

if (!check) mkdirSync(dataDirectory, {recursive: true});
output('catalog.json', `[\n${catalog.map(item => JSON.stringify(item)).join(',\n')}\n]\n`);
for (const [index, texts] of textFiles.entries()) {
  output(`texts-${index}.json`, JSON.stringify(Object.fromEntries(Object.entries(texts).sort()), null, 2) + '\n');
}
if (!process.exitCode) console.log(`${catalog.length} third-party components ${check ? 'verified' : 'recorded'}`);
