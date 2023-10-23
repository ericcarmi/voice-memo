<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";
  import { WebglPlot, WebglLine, ColorRGBA } from "webgl-plot";

  let webglp: WebglPlot;
  let line: WebglLine;
  export let data: Array<number> = [];
  export let selectedRecording: string;

  let freqwebglp: WebglPlot;
  let freqline: WebglLine;

  let canvasMain: any;
  let freqcanvas: any;
  export let freq = 420;
  export let amp = 1;

  onMount(() => {
    canvasMain = document.getElementById("time_canvas");
    const devicePixelRatio = window.devicePixelRatio || 1;
    canvasMain.width = canvasMain.clientWidth * devicePixelRatio;
    canvasMain.height = canvasMain.clientHeight * devicePixelRatio;

    webglp = new WebglPlot(canvasMain);
    const numX = 1000;

    line = new WebglLine(new ColorRGBA(1, 0, 0, 1), numX);
    webglp.addLine(line);
    line.arrangeX();

    freqcanvas = document.getElementById("freq_canvas");
    freqcanvas.width = freqcanvas.clientWidth * devicePixelRatio;
    freqcanvas.height = freqcanvas.clientHeight * devicePixelRatio;

    freqwebglp = new WebglPlot(freqcanvas);

    freqline = new WebglLine(new ColorRGBA(1, 0, 0, 1), numX);
    freqwebglp.addLine(freqline);
    freqline.arrangeX();
  });


  async function getData() {
    let data: any = await invoke("get_wav_data", {path: selectedRecording})
    update();

    let id = 0;
    let renderPlot = () => {
      for (let i = 0; i < data[1].length; i++) {
        freqline.setY(i, data[1][i] * 0.05 - 1);
      }
      for (let i = 0; i < data[0].length; i++) {
        line.setY(i, data[0][i] * 10);
      }
      webglp.update();
      freqwebglp.update();
    };
    id = requestAnimationFrame(renderPlot);

    return () => {
      renderPlot = () => {};
      cancelAnimationFrame(id);
    };
  }

  $: selectedRecording, getData();

  function update() {
    let id = 0;
    let renderPlot = () => {

      // for (let i = 0; i < line.numPoints; i++) {
      //   const ySin = Math.sin((2 * i * Math.PI * freq) / 44100);
      //   line.setY(i, ySin * amp);
      // }
      for (let i = 0; i < data.length; i++) {
        line.setY(i, data[i] * 100);
      }
      // id = requestAnimationFrame(renderPlot);
      webglp.update();
      freqwebglp.update();
    };
    id = requestAnimationFrame(renderPlot);

    return () => {
      renderPlot = () => {};
      cancelAnimationFrame(id);
    };
  }
</script>

<div>
<canvas id="freq_canvas" />
<canvas id="time_canvas" />
</div>

<style>
  canvas {
    width: 70%;
    height: 200px;
    border: 1px solid gray;
  }
</style>
