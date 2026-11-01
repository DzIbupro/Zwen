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
 <div class="eyebrow">YOUR MODEL, YOUR CHOICE</div><h1>连接你的模型<span>。</span></h1>
 <p>填写支持 OpenAI 兼容聊天协议的服务。保存配置不会发送草稿或验证联网。</p>
 {#if !desktop}<p class="hint">浏览器仅用于界面预览；请在桌面应用中保存配置和使用模型。</p>{/if}
 <form onsubmit={e=>{e.preventDefault();onSave();}}>
  <label for="api-root">API 根地址</label><input id="api-root" type="url" required maxlength="2048" value={settings.fields.baseUrl} disabled={busy||!settings.loaded} oninput={e=>onEdit('baseUrl',e.currentTarget.value)} placeholder="https://你的服务地址/v1"/>
  <small>包含版本路径（如 /v1），不包含 /chat/completions。使用 HTTPS；本机回环地址可使用 HTTP。</small>
  <label for="model-name">模型标识</label><input id="model-name" required maxlength="200" value={settings.fields.model} disabled={busy||!settings.loaded} oninput={e=>onEdit('model',e.currentTarget.value)} placeholder="填写服务提供的模型名称"/>
  <label for="image-model">图片模型（可选）</label><input id="image-model" maxlength="200" value={settings.fields.imageModel??''} disabled={busy||!settings.loaded} oninput={e=>onEdit('imageModel',e.currentTarget.value)} placeholder="截图提问使用的视觉模型，例如 glm-4.6v-flash"/>
  <small>仅截图提问使用；留空时不能发送截图。图片模型与文字模型共用同一个密钥环境变量。</small>
  <label for="key-environment">密钥环境变量名</label><input id="key-environment" required maxlength="128" pattern="[A-Z_][A-Z0-9_]*" value={settings.fields.keyEnv} disabled={busy||!settings.loaded} oninput={e=>onEdit('keyEnv',e.currentTarget.value)} spellcheck="false"/>
  <small>这里只填变量名，不填密钥。密钥从启动 Zwen 的环境读取；更改环境后需完全退出并重启应用。</small>
  {#if desktop}
   <label for="api-secret">模型密钥（可选，保存到系统钥匙环）</label><input id="api-secret" type="password" autocomplete="off" maxlength="4096" value={secretValue} disabled={secretBusy||!settings.loaded} oninput={e=>secretValue=e.currentTarget.value} placeholder="粘贴一次，之后启动无需再输入"/>
   <div class="secret-row">
    <button type="button" disabled={secretBusy||!settings.loaded||!secretValue.trim()} onclick={()=>void submitSecret()}>保存密钥</button>
    <button type="button" disabled={secretBusy||!settings.loaded} onclick={()=>void onClearSecret()}>清除已保存密钥</button>
    <span class="secret-state" role="status">{secretBusy?'正在操作钥匙环…':keyStateText}</span>
   </div>
   <small>密钥只写入本机系统钥匙环（gnome-keyring），不进数据库或日志；界面不会显示已保存的密钥。启动环境中提供了同名密钥时优先使用环境。</small>
   {#if settings.secretNotice}<div class="failure" role="alert">{settings.secretNotice}</div>{/if}
  {/if}
  <div class="state" role="status">{settings.status==='saving'?'正在保存…':settings.status==='loading'?'正在读取…':settings.status==='dirty'?'有未保存的修改':settings.config?settings.keyAvailable?'配置已保存 · 已检测到密钥，尚未验证联网':'配置已保存 · 未检测到有效密钥':'尚未配置模型'}</div>
  {#if settings.error}<div class="failure" role="alert">{settings.error}</div>{/if}
  <div class="actions"><button type="button" disabled={busy} onclick={onReload}>重新读取</button><button class="primary" type="submit" disabled={!desktop||busy||!settings.loaded}>保存模型设置</button></div>
 </form>
</section>
<style>
 .settings{max-width:620px;margin:28px auto;padding:0 18px 28px;color:#333d36}.eyebrow{font-size:10px;letter-spacing:.18em;color:#8d9f86}h1{font-size:29px;font-weight:550;letter-spacing:-1px;margin:12px 0}h1 span{color:#67946d}p,small{color:#697663;font-size:12px;line-height:1.8}form{display:flex;flex-direction:column;margin-top:28px}label{font-size:12px;margin:18px 0 8px}input{width:100%;box-sizing:border-box;border:1px solid #d7dece;border-radius:8px;padding:11px 12px;background:#fffef9;color:#333d36;font:inherit;font-size:13px}input:focus{outline:2px solid #417f65;outline-offset:2px}small{font-size:11px;margin-top:7px}.state{margin-top:23px;color:#3b7157;font-size:12px}.failure{color:#a44134;font-size:12px;margin-top:10px}.hint{border-left:2px solid #b6c8a7;padding-left:12px}.actions{display:flex;gap:12px;margin-top:20px}button{border:1px solid #d5ddcb;background:transparent;color:#55634b;border-radius:8px;padding:10px 14px;font:inherit;font-size:12px;cursor:pointer}button.primary{background:#3b7157;color:#f4f7ec;border-color:#3b7157}button:disabled,input:disabled{opacity:.5;cursor:default}
 .secret-row{display:flex;align-items:center;gap:10px;margin-top:10px;flex-wrap:wrap}.secret-row button{padding:8px 12px}.secret-state{font-size:11px;color:#55634b}
</style>
