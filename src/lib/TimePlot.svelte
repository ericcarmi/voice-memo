<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";
  import { WebglPlot, WebglLine, ColorRGBA } from "webgl-plot";

  let webglp: WebglPlot;
  let line: WebglLine;
  export let data: Array<number> = [];
  export let selectedRecording: string;

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

  $: data, console.log(data);


  async function getData() {
    let data: Array<number> = await invoke("get_wav_data", {path: selectedRecording})
    update();
    console.log(data)
    console.log(selectedRecording)

    let id = 0;
    let renderPlot = () => {

      // for (let i = 0; i < line.numPoints; i++) {
      //   const ySin = Math.sin((2 * i * Math.PI * freq) / 44100);
      //   line.setY(i, ySin * amp);
      // }
      for (let i = 0; i < data.length; i++) {
        line.setY(i, data[i] * 10);
      }
      // id = requestAnimationFrame(renderPlot);
      webglp.update();
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
    width: 70%;
    height: 300px;
    border: 1px solid gray;
  }
</style>
