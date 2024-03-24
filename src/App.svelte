<script lang="ts">
  import Renderer from './lib/Renderer.svelte'
  import { convertFileSrc, invoke } from "@tauri-apps/api/tauri";
  import { open } from '@tauri-apps/api/dialog';
  import { listen } from '@tauri-apps/api/event';
  import { fs } from '@tauri-apps/api';
  import { appCacheDir, sep } from '@tauri-apps/api/path';

  let loading = false;
  let audioPath = "";

  async function chooseFile() {
    // Open a selection dialog for image files
    const selectedPath = await open({
      multiple: false,
      filters: [{
        name: 'Audio file',
        extensions: ['wav', 'mp3']
      }]
    }) as string | null;
    if (selectedPath !== null) {
      console.log(selectedPath);
      loading = true;

      let extensionRegexArray = selectedPath.match(/\.[^\/\\.]*$/gm);
      let extensionStr = "";
      if (extensionRegexArray === null) {
        throw new Error("Invalid file extension");
      } else {
        extensionStr = extensionRegexArray[0];
      }

      const cachePath = `${await appCacheDir()}current_audio${extensionStr}`;
      console.log(cachePath);
      audioPath = convertFileSrc(cachePath);
      fs.copyFile(selectedPath, cachePath);
      await invoke("cache_audio", {path: cachePath});
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

<Renderer audioPath={audioPath}></Renderer>