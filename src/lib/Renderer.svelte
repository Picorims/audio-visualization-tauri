<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri"
  import * as PIXI from "pixi.js"

  let name = "";
  let greetMsg = ""

  async function greet(){
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    greetMsg = await invoke("greet", { name })
  }

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
    visualizer.clear();
    visualizer.beginFill("#ffffff");

    for (let i = 0; i < COUNT; i++) {
      const width = 2;
      const height = i;
      const x = i * (app.screen.width / COUNT);
      const y = 200 - height;
      visualizer.drawRect(x, y, width, height);
    }
});

  // Adding the application's view to the DOM
  document.getElementById("app")?.appendChild(app.view as HTMLCanvasElement);
</script>

