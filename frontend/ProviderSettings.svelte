<script lang="ts">
 import type {SettingsState,ProviderConfig} from './lib/provider';
 let {settings,desktop,onEdit,onSave,onReload,onSaveSecret,onClearSecret}:{settings:SettingsState;desktop:boolean;onEdit:(name:keyof ProviderConfig,value:string)=>void;onSave:()=>void;onReload:()=>void;onSaveSecret:(value:string)=>Promise<void>;onClearSecret:()=>Promise<void>}=$props();
 const busy=$derived(settings.status==='loading'||settings.status==='saving');
 let secretValue=$state('');
 const secretBusy=$derived(settings.secretSaving);
 const keyStateText=$derived(settings.keySource==='keyring'?'已保存到系统钥匙环，启动自动使用':settings.keySource==='environment'?'当前使用启动环境提供的密钥':'未检测到密钥');
 async function submitSecret(){await onSaveSecret(secretValue);if(!settings.secretNotice)secretValue='';}
</script>
<section class="settings" aria-label="模型设置">
 <h1>模型与凭据</h1>
 {#if !desktop}<p class="hint" role="alert">浏览器仅用于界面预览；请在桌面应用中保存配置和使用模型。</p>{/if}
 <form onsubmit={e=>{e.preventDefault();onSave();}}>
  <div class="group-card">
   <h2>模型服务</h2>
   <label for="api-root">API 根地址</label><input id="api-root" type="url" required maxlength="2048" value={settings.fields.baseUrl} disabled={busy||!settings.loaded} oninput={e=>onEdit('baseUrl',e.currentTarget.value)} placeholder="https://你的服务地址/v1" title="包含版本路径（如 /v1），不包含 /chat/completions"/>
   <label for="model-name">模型标识</label><input id="model-name" required maxlength="200" value={settings.fields.model} disabled={busy||!settings.loaded} oninput={e=>onEdit('model',e.currentTarget.value)} placeholder="填写服务提供的模型名称"/>
   <label for="image-model">图片模型（可选）</label><input id="image-model" maxlength="200" value={settings.fields.imageModel??''} disabled={busy||!settings.loaded} oninput={e=>onEdit('imageModel',e.currentTarget.value)} placeholder="截图提问使用的视觉模型，例如 glm-4.6v-flash" title="仅截图提问使用；留空时不能发送截图"/>
  </div>
  <div class="group-card">
   <h2>访问凭据</h2>
   <label for="key-environment">密钥环境变量名</label><input id="key-environment" required maxlength="128" pattern="[A-Z_][A-Z0-9_]*" value={settings.fields.keyEnv} disabled={busy||!settings.loaded} oninput={e=>onEdit('keyEnv',e.currentTarget.value)} spellcheck="false" title="只填变量名，不填密钥"/>
   {#if desktop}
    <label for="api-secret">模型密钥（可选，保存到系统钥匙环）</label><input id="api-secret" type="password" autocomplete="off" maxlength="4096" value={secretValue} disabled={secretBusy||!settings.loaded} oninput={e=>secretValue=e.currentTarget.value} placeholder="粘贴一次，之后启动无需再输入"/>
    <div class="secret-row">
     <button type="button" disabled={secretBusy||!settings.loaded||settings.status!=='saved'||!settings.config||!secretValue.trim()} onclick={()=>void submitSecret()}>保存密钥</button>
     <button type="button" disabled={secretBusy||!settings.loaded} onclick={()=>void onClearSecret()}>清除已保存密钥</button>
     <span class="secret-state" role="status">{secretBusy?'正在操作钥匙环…':keyStateText}</span>
    </div>
    {#if settings.secretNotice}<div class="failure" role="alert">{settings.secretNotice}</div>{/if}
   {/if}
   <details class="hint-disclosure">
    <summary>配置与密钥说明</summary>
    <div class="hint-disclosure-body">
     <p>填写支持 OpenAI 兼容聊天协议的服务，保存配置不会发送草稿或验证联网。</p>
     <p>API 根地址需包含版本路径（如 <code>/v1</code>），不包含 <code>/chat/completions</code>；使用 HTTPS，本机回环地址可使用 HTTP。</p>
     <p>图片模型仅截图提问使用，留空时不能发送截图，与文字模型共用同一密钥环境变量。</p>
     <p>环境变量栏只填变量名；密钥可优先从启动环境读取，或先保存上方设置后再写入系统钥匙环（gnome-keyring），不进数据库或日志。</p>
    </div>
   </details>
  </div>
  <div class="state" role="status">{settings.status==='saving'?'正在保存…':settings.status==='loading'?'正在读取…':settings.status==='dirty'?'有未保存的修改':settings.config?settings.keyAvailable?'配置已保存 · 已检测到密钥，尚未验证联网':'配置已保存 · 未检测到有效密钥':'尚未配置模型'}</div>
  {#if settings.error}<div class="failure" role="alert">{settings.error}</div>{/if}
  <div class="actions"><button type="button" disabled={busy} onclick={onReload}>重新读取</button><button class="primary" type="submit" disabled={!desktop||busy||!settings.loaded}>保存模型设置</button></div>
 </form>
</section>
<style>
 .settings{max-width:580px;margin:20px auto;padding:0 18px 28px;color:var(--text);font-size:14px;line-height:1.6}
 h1{font-size:20px;font-weight:600;margin:0 0 18px}
 h2{font-size:12.5px;font-weight:600;color:var(--muted);letter-spacing:.02em;margin:0 0 10px}
 .group-card{background:var(--surface);border:1px solid var(--edge);border-radius:var(--radius-card);padding:14px 16px;margin-bottom:14px;display:flex;flex-direction:column}
 p{color:var(--muted);font-size:13px;line-height:1.65}
 form{display:flex;flex-direction:column}
 label{font-size:12.5px;font-weight:500;color:var(--muted);margin:10px 0 5px}
 .group-card h2+label{margin-top:2px}
 input{width:100%;box-sizing:border-box;border:1px solid var(--edge);border-radius:var(--radius-control);padding:8px 11px;background:var(--surface-soft);color:var(--text);font:inherit;font-size:13.5px;transition:border-color 140ms ease,background-color 140ms ease}
 input:focus{outline:2px solid var(--focus-ring);outline-offset:1px;border-color:var(--focus-ring);background:var(--surface)}
 .state{margin-top:2px;color:var(--muted);font-size:12.5px}
 .failure{color:var(--danger);font-size:13px;margin-top:8px}
 .hint{border-left:2px solid var(--edge-strong);padding-left:10px;margin:0 0 12px}
 .actions{display:flex;gap:10px;margin-top:14px;flex-wrap:wrap}
 .secret-row{display:flex;align-items:center;gap:8px;margin-top:10px;flex-wrap:wrap}
 .secret-state{font-size:12.5px;color:var(--muted)}
 button{border:1px solid var(--edge);background:var(--surface);color:var(--text);border-radius:var(--radius-control);padding:7px 13px;font:inherit;font-size:13px;font-weight:500;cursor:pointer;transition:transform 140ms var(--ease-out),background-color 140ms ease,border-color 140ms ease}
 button.primary{background:var(--accent);color:var(--on-accent);border-color:var(--accent)}
 @media(hover:hover) and (pointer:fine){button:hover:not(:disabled){background:var(--surface-hover);border-color:var(--edge-strong)}button.primary:hover:not(:disabled){background:var(--accent);opacity:.9;color:var(--on-accent)}}
 button:active:not(:disabled){transform:scale(0.96)}
 button:focus-visible,input:focus-visible{outline:2px solid var(--focus-ring);outline-offset:2px}
 button:disabled,input:disabled{opacity:.45;cursor:default}
 @media(prefers-reduced-motion:reduce){button{transition-property:background-color,color,opacity;transition-duration:120ms;transform:none}}
 @media(max-width:520px){.settings{padding-inline:0}.secret-row{gap:10px}.secret-state{overflow-wrap:anywhere}}
</style>
