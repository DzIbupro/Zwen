<script lang="ts">
 import {reasoningLevelsOf,type SettingsState,type ProviderConfig,type ReasoningEffort} from './lib/provider';
 import {getLocale,t,type Locale} from './lib/i18n.svelte';
 let {settings,desktop,onEdit,onSave,onReload,onSaveSecret,onClearSecret}:{settings:SettingsState;desktop:boolean;onEdit:(name:keyof ProviderConfig,value:string)=>void;onSave:()=>void;onReload:()=>void;onSaveSecret:(value:string)=>Promise<void>;onClearSecret:()=>Promise<void>}=$props();
 const busy=$derived(settings.status==='loading'||settings.status==='saving');
 const levels=$derived(reasoningLevelsOf(settings));
 const reasoningLabels:Record<Locale,Record<ReasoningEffort|'label'|'default',string>>={
  'zh-CN':{label:'推理档位',default:'由服务决定',none:'关闭',minimal:'极低',low:'低',medium:'中',high:'高',xhigh:'极高',max:'最高'},
  en:{label:'Reasoning effort',default:'Service default',none:'Off',minimal:'Minimal',low:'Low',medium:'Medium',high:'High',xhigh:'Extra high',max:'Maximum'},
  ja:{label:'推論レベル',default:'サービスの既定値',none:'オフ',minimal:'最小',low:'低',medium:'中',high:'高',xhigh:'非常に高い',max:'最大'},
  ko:{label:'추론 수준',default:'서비스 기본값',none:'끔',minimal:'최소',low:'낮음',medium:'보통',high:'높음',xhigh:'매우 높음',max:'최대'},
  es:{label:'Nivel de razonamiento',default:'Valor predeterminado del servicio',none:'Desactivado',minimal:'Mínimo',low:'Bajo',medium:'Medio',high:'Alto',xhigh:'Muy alto',max:'Máximo'},
 };
 const reasoningText=$derived(reasoningLabels[getLocale()]);
 let secretValue=$state('');
 const secretBusy=$derived(settings.secretSaving);
 const keyStateText=$derived(settings.keySource==='keyring'?t('provider.keyKeyring'):settings.keySource==='environment'?t('provider.keyEnvironment'):t('provider.keyNone'));
 async function submitSecret(){await onSaveSecret(secretValue);if(!settings.secretNotice)secretValue='';}
</script>
<section class="settings" aria-label={t('provider.aria')}>
 <h1>{t('provider.title')}</h1>
 {#if !desktop}<p class="hint" role="alert">{t('provider.previewHint')}</p>{/if}
 <form onsubmit={e=>{e.preventDefault();onSave();}}>
  <div class="group-card">
   <h2>{t('provider.serviceGroup')}</h2>
   <label for="api-root">{t('provider.apiRoot')}</label><input id="api-root" type="url" required maxlength="2048" value={settings.fields.baseUrl} disabled={busy||!settings.loaded} oninput={e=>onEdit('baseUrl',e.currentTarget.value)} placeholder={t('provider.apiRootPlaceholder')} title={t('provider.apiRootTitle')}/>
   <label for="model-name">{t('provider.modelName')}</label><input id="model-name" required maxlength="200" value={settings.fields.model} disabled={busy||!settings.loaded} oninput={e=>onEdit('model',e.currentTarget.value)} placeholder={t('provider.modelPlaceholder')}/>
   <label for="image-model">{t('provider.imageModel')}</label><input id="image-model" maxlength="200" value={settings.fields.imageModel??''} disabled={busy||!settings.loaded} oninput={e=>onEdit('imageModel',e.currentTarget.value)} placeholder={t('provider.imageModelPlaceholder')} title={t('provider.imageModelTitle')}/>
   {#if levels.length}
    <label for="reasoning-effort">{reasoningText.label}</label>
    <select id="reasoning-effort" value={settings.fields.reasoningEffort??''} disabled={busy||!settings.loaded} onchange={e=>onEdit('reasoningEffort',e.currentTarget.value)}>
     <option value="">{reasoningText.default}</option>
     {#each levels as level (level)}<option value={level}>{reasoningText[level]}</option>{/each}
    </select>
   {/if}
  </div>
  <div class="group-card">
   <h2>{t('provider.credGroup')}</h2>
   <label for="key-environment">{t('provider.keyEnv')}</label><input id="key-environment" required maxlength="128" pattern="[A-Z_][A-Z0-9_]*" value={settings.fields.keyEnv} disabled={busy||!settings.loaded} oninput={e=>onEdit('keyEnv',e.currentTarget.value)} spellcheck="false" title={t('provider.keyEnvTitle')}/>
   {#if desktop}
    <label for="api-secret">{t('provider.apiSecret')}</label><input id="api-secret" type="password" autocomplete="off" maxlength="4096" value={secretValue} disabled={secretBusy||!settings.loaded} oninput={e=>secretValue=e.currentTarget.value} placeholder={t('provider.apiSecretPlaceholder')}/>
    <div class="secret-row">
     <button type="button" disabled={secretBusy||!settings.loaded||settings.status!=='saved'||!settings.config||!secretValue.trim()} onclick={()=>void submitSecret()}>{t('provider.saveSecret')}</button>
     <button type="button" disabled={secretBusy||!settings.loaded} onclick={()=>void onClearSecret()}>{t('provider.clearSecret')}</button>
     <span class="secret-state" role="status">{secretBusy?t('provider.secretBusy'):keyStateText}</span>
    </div>
    {#if settings.secretNotice}<div class="failure" role="alert">{settings.secretNotice}</div>{/if}
   {/if}
   <details class="hint-disclosure">
    <summary>{t('provider.helpSummary')}</summary>
    <div class="hint-disclosure-body">
     <p>{t('provider.help1')}</p>
     <p>{t('provider.help2')}</p>
     <p>{t('provider.help3')}</p>
     <p>{t('provider.help4')}</p>
    </div>
   </details>
  </div>
  <div class="state" role="status">{settings.status==='saving'?t('provider.statusSaving'):settings.status==='loading'?t('provider.statusLoading'):settings.status==='dirty'?t('provider.statusDirty'):settings.config?settings.keyAvailable?t('provider.statusSavedReady'):t('provider.statusSavedNoKey'):t('provider.statusUnconfigured')}</div>
  {#if settings.error}<div class="failure" role="alert">{settings.error}</div>{/if}
  <div class="actions"><button type="button" disabled={busy} onclick={onReload}>{t('general.reload')}</button><button class="primary" type="submit" disabled={!desktop||busy||!settings.loaded}>{t('provider.save')}</button></div>
 </form>
</section>
<style>
 .settings{max-width:600px;margin:20px auto;padding:0 18px 28px;color:var(--text);font-size:1rem;line-height:1.6}
 h1{font-size:1.35rem;font-weight:600;margin:0 0 18px}
 h2{font-size:.86rem;font-weight:600;color:var(--muted);letter-spacing:.02em;margin:0 0 10px}
 .group-card{background:var(--surface);border:1px solid var(--edge);border-radius:var(--radius-card);padding:14px 16px;margin-bottom:14px;display:flex;flex-direction:column}
 p{color:var(--muted);font-size:.9rem;line-height:1.65}
 form{display:flex;flex-direction:column}
 label{font-size:.88rem;font-weight:500;color:var(--muted);margin:10px 0 5px}
 .group-card h2+label{margin-top:2px}
 input,select{width:100%;box-sizing:border-box;border:1px solid var(--edge);border-radius:var(--radius-control);padding:8px 11px;background:var(--surface-soft);color:var(--text);font:inherit;font-size:.96rem;transition:border-color 140ms ease,background-color 140ms ease}
 input:focus,select:focus{outline:2px solid var(--focus-ring);outline-offset:1px;border-color:var(--focus-ring);background:var(--surface)}
 .state{margin-top:2px;color:var(--muted);font-size:.86rem}
 .failure{color:var(--danger);font-size:.9rem;margin-top:8px}
 .hint{border-left:2px solid var(--edge-strong);padding-left:10px;margin:0 0 12px}
 .actions{display:flex;gap:10px;margin-top:14px;flex-wrap:wrap}
 .secret-row{display:flex;align-items:center;gap:8px;margin-top:10px;flex-wrap:wrap}
 .secret-state{font-size:.86rem;color:var(--muted)}
 button{border:1px solid var(--edge);background:var(--surface);color:var(--text);border-radius:var(--radius-control);padding:7px 13px;font:inherit;font-size:.92rem;font-weight:500;cursor:pointer;transition:transform 140ms var(--ease-out),background-color 140ms ease,border-color 140ms ease}
 button.primary{background:var(--accent);color:var(--on-accent);border-color:var(--accent)}
 @media(hover:hover) and (pointer:fine){button:hover:not(:disabled){background:var(--surface-hover);border-color:var(--edge-strong)}button.primary:hover:not(:disabled){background:var(--accent);opacity:.9;color:var(--on-accent)}}
 button:active:not(:disabled){transform:scale(0.96)}
 button:focus-visible,input:focus-visible,select:focus-visible{outline:2px solid var(--focus-ring);outline-offset:2px}
 button:disabled,input:disabled,select:disabled{opacity:.45;cursor:default}
 @media(prefers-reduced-motion:reduce){button{transition-property:background-color,color,opacity;transition-duration:120ms;transform:none}}
 @media(max-width:520px){.settings{padding-inline:0}.secret-row{gap:10px}.secret-state{overflow-wrap:anywhere}}
</style>
