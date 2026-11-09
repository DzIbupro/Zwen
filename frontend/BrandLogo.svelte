<script lang="ts">
 import logo from './assets/zwen-logo.png';
 let {compact=false}:{compact?:boolean}=$props();
</script>
<img src={logo} alt="" aria-hidden="true" class:compact class="app-logo" width={compact?20:32} height={compact?20:32}/>
<style>
 .app-logo{display:block;flex:none;width:32px;height:32px;border-radius:7px;object-fit:contain;background:transparent;}
 .app-logo.compact{width:20px;height:20px;border-radius:5px;}
</style>
