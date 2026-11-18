<script>
  let { ancho } = $props();
</script>

<img src="/logo-dark.svg" width={ancho} alt="Font Time" draggable="false" data-tauri-drag-region />

<style>
  img { display: block; max-width: 100%; height: auto; aspect-ratio: 1660 / 420; }
  :global([data-theme="dark"]) img { content: url("/logo-white.svg"); }
</style>
