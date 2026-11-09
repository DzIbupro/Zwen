<script lang="ts">
 import mask from '../src-tauri/icons/icon.png';

 // Sample a still field: two broad poles shape density and weight without noise or motion.
 const field = Array.from({ length: 35 * 69 }, (_, index) => {
  const column = index % 69;
  const row = Math.floor(index / 69);
  const x = 53 + column * 2.2;
  const y = 63 + row * 3.7;
  const pole = (px: number, py: number, radius: number) => Math.exp(-((x - px) ** 2 + (y - py) ** 2) / radius ** 2);
  const strength = Math.min(1, .18 + .8 * pole(163, 84, 35) + .68 * pole(79, 159, 29));
  const ripple = (Math.sin(column * .67 + row * .31) + 1) / 2;
  return {
   x,
   y,
   width: .42 + strength * .57,
   height: 1.25 + strength * 1.85 + ripple * .3,
   opacity: .2 + strength * .7,
   angle: 19 * Math.sin((x - 112) / 42) * Math.cos((y - 123) / 53),
   // Thinner regions breathe; strong poles retain the full quantized lattice.
   visible: strength > .45 || (column + row * 2) % 4 !== 0,
  };
 });
</script>
<div class="welcome-mark" data-welcome-background aria-hidden="true">
 <svg viewBox="54 64 148 124" focusable="false">
  <defs>
   <!-- Keep the original silhouette; threshold its near-black texture out of the mask. -->
   <filter id="zwen-field-threshold" color-interpolation-filters="sRGB">
    <feComponentTransfer><feFuncR type="discrete" tableValues="0 0 0 1"/><feFuncG type="discrete" tableValues="0 0 0 1"/><feFuncB type="discrete" tableValues="0 0 0 1"/></feComponentTransfer>
   </filter>
   <mask id="zwen-field-mask" maskUnits="userSpaceOnUse" x="0" y="0" width="256" height="256" style="mask-type:luminance"><image href={mask} x="0" y="0" width="256" height="256" filter="url(#zwen-field-threshold)"/></mask>
  </defs>
  <g fill="currentColor" mask="url(#zwen-field-mask)">
   {#each field as sample}
    {#if sample.visible}
     <rect x={sample.x} y={sample.y} width={sample.width} height={sample.height} rx={sample.width / 2} opacity={sample.opacity} transform={`rotate(${sample.angle} ${sample.x} ${sample.y})`}/>
    {/if}
   {/each}
  </g>
 </svg>
</div>
<style>
 .welcome-mark{position:absolute;inset:7% 6%;pointer-events:none;user-select:none;color:var(--text);opacity:.25;z-index:0;overflow:hidden}
 svg{display:block;width:100%;height:100%}
 :global(:root[data-theme=dark]) .welcome-mark{opacity:.2}
</style>
