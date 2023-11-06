<script lang="ts">
  import Renderer from './lib/Renderer.svelte'
  import { invoke } from "@tauri-apps/api/tauri";
  import { open } from '@tauri-apps/api/dialog';
  import { listen } from '@tauri-apps/api/event';

  let loading = false;
  
  async function chooseFile() {
    // Open a selection dialog for image files
    const selectedPath = await open({
      multiple: false,
      filters: [{
        name: 'Audio file',
        extensions: ['wav', 'mp3']
      }]
    });
    if (selectedPath !== null) {
      console.log(selectedPath);
      loading = true;
      await invoke("cache_audio", {path: selectedPath});
    }
  }

  listen("audio_cached", (e) => {
    loading = false;
    alert((e.payload as {error: boolean, message: string}).message);
  });
</script>

<fieldset>
  <legend>Audio File</legend>
  <button on:click={chooseFile}>Pick</button>
  {#if loading}
    <progress></progress>
  {/if}
</fieldset>

<Renderer></Renderer>