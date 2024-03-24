<script lang="ts">
    import { invoke } from "@tauri-apps/api";
  import * as PIXI from "pixi.js"
    import { LinearToLog, MappedArray } from "./array_utils";

  export let audioPath = "";
  let audio: HTMLAudioElement | null = null;
  let playing = false;
  let fps = 60;
  let fftCache: number[][] = [];

  function init() {
    console.log("init pixi...");
    const app = new PIXI.Application({
      background: '#1099bb',
      width: 1280,
      height: 720,
    });
  
    // build visualizer
    const COUNT = 256;
    const height = 200;
    const visualizer = new PIXI.Graphics();
    app.stage.addChild(visualizer);
    visualizer.x = 0;
    visualizer.y = app.screen.height - height;
  
  
    // Listen for animate update
    app.ticker.add((delta) =>
    {
      if (!playing || !audio) {
        return;
      }

      const frame = Math.floor(audio.currentTime * fps);
      if (fftCache[frame] === undefined) {
        return;
      }
      let fft = LinearToLog(MappedArray(fftCache[frame], COUNT, 1, fftCache[frame].length-1));
      const max = Math.max(...fft);
      fft = fft.map((x) => x / max);

      visualizer.clear();
      visualizer.beginFill("#ffffff");
  
      for (let i = 0; i < COUNT; i++) {
        const width = 2;
        const height = fft[i] * 255;
        const x = i * (app.screen.width / COUNT);
        const y = 200 - height;
        visualizer.drawRect(x, y, width, height);
      }
      visualizer.drawRect(0,0,audio.currentTime/audio.duration * app.screen.width, 10);
    });
  
    // Adding the application's view to the DOM
    document.getElementById("pixi-box")?.appendChild(app.view as HTMLCanvasElement);
  }

  async function play() {
    if (audio !== null) {
      fftCache = [];
      for (let i = 0; i < audio.duration * fps; i++) {
        console.log(`getting fft for frame ${i}`);
        fftCache.push(await invoke("get_frame_fft", {frame: i}));
      }
      console.log(fftCache);
      audio.play();
      playing = true;
    }
  }

  async function stop() {
    if (audio !== null) {
      audio.pause();
      audio.currentTime = 0;
      playing = false;
    }
  }
</script>

<svelte:window on:load={init}/>
<audio bind:this={audio} src={audioPath}></audio>
<fieldset>
  <legend>controls</legend>
  <button on:click={play}>Play</button>
  <button on:click={stop}>Stop</button>
</fieldset>
<div id="pixi-box">

</div>