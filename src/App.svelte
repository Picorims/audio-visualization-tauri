<script lang="ts">
  import Renderer from './lib/Renderer.svelte'
  import { invoke } from "@tauri-apps/api/tauri";
  import { open } from '@tauri-apps/api/dialog';
  
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
      invoke("cache_audio", {path: selectedPath});
    }
  }
</script>

<fieldset>
  <legend>Audio File</legend>
  <button on:click={chooseFile}>Pick</button>
</fieldset>

<Renderer></Renderer>
