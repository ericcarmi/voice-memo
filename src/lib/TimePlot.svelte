<script lang="ts">
  import { onMount } from "svelte";
  import { WebglPlot, WebglLine, ColorRGBA } from "webgl-plot";

  let webglp: WebglPlot;
  let line: WebglLine;


  let canvasMain: any;
  export let freq = 420;
  export let amp = 1;

  onMount(() => {
    canvasMain = document.getElementById("my_canvas");
    const devicePixelRatio = window.devicePixelRatio || 1;
    canvasMain.width = canvasMain.clientWidth * devicePixelRatio;
    canvasMain.height = canvasMain.clientHeight * devicePixelRatio;

    webglp = new WebglPlot(canvasMain);
    const numX = 1000;

    line = new WebglLine(new ColorRGBA(1, 0, 0, 1), numX);
    webglp.addLine(line);
    line.arrangeX();
  });

  $: freq, update();

  function update() {
    let id = 0;
    let renderPlot = () => {
      for (let i = 0; i < line.numPoints; i++) {
        const ySin = Math.sin((2 * i * Math.PI * freq) / 44100);
        line.setY(i, ySin * amp);
      }
      id = requestAnimationFrame(renderPlot);
      webglp.update();
    };
    id = requestAnimationFrame(renderPlot);

    return () => {
      renderPlot = () => {};
      cancelAnimationFrame(id);
    };
  }

</script>

<canvas id="my_canvas" />

<style>
  canvas {
    width: 400px;
    height: 300px;
  }
</style>
