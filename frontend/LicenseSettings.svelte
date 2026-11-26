<script module lang="ts">
  type Component = {
    name: string;
    version: string;
    license: string;
    source: string;
    notices: {name: string; key: string}[];
  };
  type Catalog = {components: Component[]; texts: Record<string, string>};

  const noticeFiles = import.meta.glob<string>('./license-data/texts-*.json', {query: '?raw', import: 'default'});
  let pending: Promise<Catalog> | undefined;

  function loadCatalog(): Promise<Catalog> {
    pending ??= Promise.all([
      import('./license-data/catalog.json?raw'),
      Promise.all(Object.values(noticeFiles).map(load => load())),
    ]).then(([catalog, files]) => ({
      components: JSON.parse(catalog.default),
      texts: Object.assign({}, ...files.map(file => JSON.parse(file))),
    })).catch(error => {
      pending = undefined;
      throw error;
    });
    return pending;
  }
</script>

<script lang="ts">
  import {isDesktop, openExternal} from './lib/bridge';
  import {getLocale} from './lib/i18n.svelte';

  const labels = {
    'zh-CN': {
      title: '第三方许可证', scope: '随应用分发的界面组件、字体和图标，以及 Linux x86_64 桌面构建的运行时依赖。',
      search: '搜索组件、版本或许可证', empty: '没有匹配的组件。', loading: '正在加载许可证…', error: '无法加载许可证。请合上后重新展开。', source: '上游来源',
    },
    en: {
      title: 'Third-party licenses', scope: 'Bundled interface components, fonts, and icons, plus runtime dependencies for the Linux x86_64 desktop build.',
      search: 'Search components, versions, or licenses', empty: 'No matching components.', loading: 'Loading licenses…', error: 'Could not load licenses. Close and reopen this section to try again.', source: 'Upstream source',
    },
    ja: {
      title: 'サードパーティライセンス', scope: '同梱の UI コンポーネント、フォント、アイコン、および Linux x86_64 デスクトップ版の実行時依存関係。',
      search: 'コンポーネント、バージョン、ライセンスを検索', empty: '一致するコンポーネントがありません。', loading: 'ライセンスを読み込み中…', error: 'ライセンスを読み込めませんでした。閉じてから開き直してください。', source: '配布元',
    },
    ko: {
      title: '타사 라이선스', scope: '앱에 포함된 인터페이스 구성 요소, 글꼴, 아이콘과 Linux x86_64 데스크톱 빌드의 런타임 종속성입니다.',
      search: '구성 요소, 버전 또는 라이선스 검색', empty: '일치하는 구성 요소가 없습니다.', loading: '라이선스 불러오는 중…', error: '라이선스를 불러올 수 없습니다. 이 섹션을 닫았다가 다시 여세요.', source: '원본 소스',
    },
    es: {
      title: 'Licencias de terceros', scope: 'Componentes de interfaz, fuentes e iconos incluidos, y dependencias de ejecución de la versión de escritorio para Linux x86_64.',
      search: 'Buscar componentes, versiones o licencias', empty: 'No hay componentes que coincidan.', loading: 'Cargando licencias…', error: 'No se pudieron cargar las licencias. Cierra y vuelve a abrir esta sección.', source: 'Fuente original',
    },
  };

  let copy = $derived(labels[getLocale()]);
  let open = $state(false);
  let query = $state('');
  let terms = $derived(query.toLowerCase().trim().split(/\s+/).filter(Boolean));
  let active = $state<string | null>(null);

  function matches(component: Component) {
    const description = `${component.name} ${component.version} ${component.license}`.toLowerCase();
    return terms.every(term => description.includes(term));
  }
</script>

<details class="licenses" bind:open>
  <summary>{copy.title}</summary>
  {#if open}
    {#await loadCatalog()}
      <p class="status" role="status">{copy.loading}</p>
    {:then catalog}
      <p class="status">{copy.scope}</p>
      <input type="search" aria-label={copy.search} placeholder={copy.search} bind:value={query} />
      <ul class="components" aria-label={copy.title}>
        {#each catalog.components.filter(matches) as component (component.source)}
          <li>
            <details name="zwen-license-component" ontoggle={event => {
              if (event.currentTarget.open) active = component.source;
            }}>
              <summary>
                <span class="component-info">
                  <span>{component.name} <span class="version">{component.version}</span></span>
                  <span class="expression">{component.license}</span>
                </span>
              </summary>
              {#if active === component.source}
                <div class="notices">
                  <a href={component.source} target="_blank" rel="noopener noreferrer" onclick={event => {
                    if (isDesktop) {
                      event.preventDefault();
                      void openExternal(component.source);
                    }
                  }}>{copy.source}</a>
                  {#each component.notices as notice}
                    <h3>{notice.name}</h3>
                    <pre>{catalog.texts[notice.key]}</pre>
                  {/each}
                </div>
              {/if}
            </details>
          </li>
        {:else}
          <li class="status" role="status">{copy.empty}</li>
        {/each}
      </ul>
    {:catch}
      <p class="status" role="alert">{copy.error}</p>
    {/await}
  {/if}
</details>

<style>
  .licenses { border-top: 1px solid var(--edge); margin-top: 10px; }
  summary { cursor: pointer; padding: 10px 4px; }
  .licenses > summary { font-size: .96rem; font-weight: 500; }
  .status { margin: 0 0 8px; color: var(--muted); font-size: .85rem; }
  input {
    display: block; box-sizing: border-box; width: 100%; padding: 8px 10px; font: inherit; font-size: .875rem;
    color: var(--text); background: var(--surface-soft); border: 1px solid var(--edge); border-radius: var(--radius-control);
  }
  input:focus-visible, summary:focus-visible, a:focus-visible { outline: 2px solid var(--focus-ring); outline-offset: 1px; }
  .components { list-style: none; margin: 8px 0 0; padding: 0 4px; max-height: min(50vh, 29rem); overflow: auto; }
  .components > li + li { border-top: 1px solid var(--edge); }
  .components > .status { padding: 8px 0; }
  .component-info { display: inline-flex; vertical-align: middle; width: calc(100% - 1.5rem); justify-content: space-between; gap: 3px 12px; flex-wrap: wrap; overflow-wrap: anywhere; font-size: .875rem; }
  .version, .expression { color: var(--muted); font-size: .8rem; }
  .version { font-family: var(--font-mono); }
  .notices { margin: 0 2px 10px; padding: 10px; background: var(--surface-soft); border: 1px solid var(--edge); border-radius: var(--radius-control); }
  a { color: var(--text); font-size: .8rem; text-decoration: underline; }
  h3 { margin: 12px 0 6px; font-size: .8rem; overflow-wrap: anywhere; }
  pre { margin: 0; font: .8rem/1.5 var(--font-mono); white-space: pre-wrap; overflow-wrap: anywhere; user-select: text; }
</style>
