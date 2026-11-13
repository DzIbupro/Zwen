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
 <h1>模型设置</h1>
 {#if !desktop}<p class="hint" role="alert">浏览器仅用于界面预览；请在桌面应用中保存配置和使用模型。</p>{/if}
 <form onsubmit={e=>{e.preventDefault();onSave();}}>
  <label for="api-root">API 根地址</label><input id="api-root" type="url" required maxlength="2048" value={settings.fields.baseUrl} disabled={busy||!settings.loaded} oninput={e=>onEdit('baseUrl',e.currentTarget.value)} placeholder="https://你的服务地址/v1" title="包含版本路径（如 /v1），不包含 /chat/completions"/>
  <label for="model-name">模型标识</label><input id="model-name" required maxlength="200" value={settings.fields.model} disabled={busy||!settings.loaded} oninput={e=>onEdit('model',e.currentTarget.value)} placeholder="填写服务提供的模型名称"/>
  <label for="image-model">图片模型（可选）</label><input id="image-model" maxlength="200" value={settings.fields.imageModel??''} disabled={busy||!settings.loaded} oninput={e=>onEdit('imageModel',e.currentTarget.value)} placeholder="截图提问使用的视觉模型，例如 glm-4.6v-flash" title="仅截图提问使用；留空时不能发送截图"/>
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
  <div class="state" role="status">{settings.status==='saving'?'正在保存…':settings.status==='loading'?'正在读取…':settings.status==='dirty'?'有未保存的修改':settings.config?settings.keyAvailable?'配置已保存 · 已检测到密钥，尚未验证联网':'配置已保存 · 未检测到有效密钥':'尚未配置模型'}</div>

    {#if settings.error}<div class="failure" role="alert">{settings.error}</div>{/if}
  <div class="actions"><button type="button" disabled={busy} onclick={onReload}>重新读取</button><button class="primary" type="submit" disabled={!desktop||busy||!settings.loaded}>保存模型设置</button></div>
 </form>
</section>
<style>
 .settings{max-width:620px;margin:28px auto;padding:0 18px 28px;color:var(--text)}h1{font-size:29px;font-weight:550;letter-spacing:-1px;margin:12px 0}p{color:var(--muted);font-size:13px;line-height:1.8}form{display:flex;flex-direction:column;margin-top:28px}label{font-size:13px;margin:18px 0 8px}input{width:100%;box-sizing:border-box;border:1px solid var(--edge);border-radius:12px;padding:11px 12px;background:var(--surface);color:var(--text);font:inherit;font-size:13px}input:focus{outline:2px solid var(--focus-ring);outline-offset:2px}.state{margin-top:23px;color:var(--accent);font-size:13px}.failure{color:var(--danger);font-size:13px;margin-top:10px}.hint{border-left:2px solid var(--edge);padding-left:12px}.actions{display:flex;gap:12px;margin-top:20px}button{border:1px solid var(--edge);background:transparent;color:var(--muted);border-radius:12px;padding:10px 14px;font:inherit;font-size:13px;cursor:pointer}button.primary{background:var(--accent);color:var(--on-accent);border-color:var(--accent)}button:disabled,input:disabled{opacity:.5;cursor:default}
 .secret-row{display:flex;align-items:center;gap:10px;margin-top:10px;flex-wrap:wrap}.secret-row button{padding:8px 12px}.secret-state{font-size:13px;color:var(--muted)}

 button{transition:transform 140ms var(--ease-out),box-shadow 160ms var(--ease-out),background-color 160ms ease;border-radius:12px;cursor:pointer;box-shadow:var(--raised-shadow)}
 @media(hover:hover) and (pointer:fine){button:hover:not(:disabled){box-shadow:var(--hover-shadow)}}
 button:active:not(:disabled){box-shadow:var(--pressed-shadow);transform:scale(0.96)}
 button:focus-visible,input:focus-visible{outline:2px solid var(--focus-ring);outline-offset:3px}
 button:disabled{opacity:.5;cursor:default}
 @media(prefers-reduced-motion:reduce){button{transition-property:background-color,color,opacity;transition-duration:120ms;transform:none}}
 .settings{font-size:16px;line-height:1.65}h1{font-size:26px;margin:0}p{font-size:16px;line-height:1.65}label{font-size:14px}input{font-size:16px;box-shadow:var(--inset-shadow)}button{font-size:14px;background:var(--surface);color:var(--text)}.actions{flex-wrap:wrap}.failure{color:var(--danger)}@media(max-width:520px){.settings{padding-inline:0}.secret-row{gap:12px}.secret-state{overflow-wrap:anywhere}}
</style>
