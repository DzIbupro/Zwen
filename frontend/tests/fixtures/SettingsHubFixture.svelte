<script lang="ts">
 import SettingsHub from '../../SettingsHub.svelte';
 import ProviderSettings from '../../ProviderSettings.svelte';
 import type {SettingsState} from '../../lib/provider';
 let {onSave=()=>{},onReload=()=>{}}:{onSave?:()=>void;onReload?:()=>void}=$props();
 let settings=$state<SettingsState>({config:null,revision:0,keyAvailable:false,keySource:null,fields:{baseUrl:'https://example.test/v1',model:'fixture',keyEnv:'ZWEN_API_KEY',imageModel:''},status:'dirty',loaded:true,error:null,secretSaving:false,secretNotice:null});
</script>
<SettingsHub>
 {#snippet providerPanel()}<ProviderSettings {settings} desktop={true} onEdit={(name,value)=>settings.fields[name]=value} {onSave} {onReload} onSaveSecret={async()=>{}} onClearSecret={async()=>{}}/>{/snippet}
 {#snippet appearancePanel()}<h1>外观内容</h1>{/snippet}
 {#snippet selectionPanel()}<h1>划词内容</h1>{/snippet}
 {#snippet quickPanel()}<h1>快捷内容</h1>{/snippet}
</SettingsHub>
