<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  import { WebglPlot, WebglLine, ColorRGBA } from "webgl-plot";
  import { linlog, loglin } from "./types.svelte";
  // import { loglin, linlog } from "./types.svelte";

  let webglp: WebglPlot;
  let line: WebglLine;
  export let selectedRecording: string;

  let canvasMain: any;
  let ctx: CanvasRenderingContext2D;
  let freqcanvas: any;
  export const freq = 440;
  export const amp = 1;

  let width = 600;
  let height = 200;
  const color = { r: 158 / 255, g: 98 / 255, b: 64 / 255 };

  const rgbacolor = new ColorRGBA(color.r, color.g, color.b, 1);
  // const background = { r: 255 / 255, g: 224 / 255, b: 181 / 255 };
  let indicator_position = 0;
  let time_length = 1;

  let time_labels: Array<string> = [];

  let unlisten = listen("time_indicator", (event: any) => {
    // console.log(event.payload)
    indicator_position = (event.payload / time_length) * width;
  });

  onDestroy(() => {
    unlisten.then((f) => f());
  });

  onMount(() => {
    canvasMain = document.getElementById("time_canvas");
    const devicePixelRatio = window.devicePixelRatio || 1;
    canvasMain.width = canvasMain.clientWidth * devicePixelRatio;
    canvasMain.height = canvasMain.clientHeight * devicePixelRatio;

    webglp = new WebglPlot(canvasMain);
    const numX = 1000;

    line = new WebglLine(new ColorRGBA(color.r, color.g, color.b, 1), numX);
    webglp.addLine(line);
    line.arrangeX();

    freqcanvas = document.getElementById("freq_canvas");
    freqcanvas.width = width;
    freqcanvas.height = height;

    ctx = freqcanvas.getContext("2d", { willReadFrequently: true });
  });

  async function getData() {
    if (selectedRecording === "") return;

    let data: any = await invoke("get_stft_data", {
      fileName: selectedRecording,
    });

    let id = 0;
    let renderPlot = () => {
      const fftsize = 512; // already divided by 2
      // setting the width here will scale it properly, much easier than dealing with fractional indices (even more complicated due to stft resampling to match image data structure)
      freqcanvas.width = data[1].length / fftsize;

      const T = width / (data[1].length / fftsize);

      line = new WebglLine(rgbacolor, data[0].length);

      webglp.removeAllLines();
      webglp.addLine(line);
      line.arrangeX();
      // const ratio = data[0].length / width;
      // console.log(data[0].length, ratio);

      time_length = data[0].length;
      time_labels = ["0", (time_length / 44100.0).toFixed(3).toString()];

      // should probably downsample
      for (let i = 0; i < data[0].length; i++) {
        line.setY(i, data[0][i]);
      }
      webglp.update();
      // ctx = freqcanvas.getContext("2d", { willReadFrequently: true });

      const image = ctx.getImageData(0, 0, width, height);
      const image_data = image.data;
      const L = image_data.length;

      /* 
        test case -- bottom half is white, upper half magenta
        weird indexing because image array goes horizontally but stft is drawn vertically first, then horizontally, all rows for one column, shift column
      */
      // var row = 0;
      // var col = 0;
      // for (let i = 0; i < L; i += 4) {
      //   // row = i * height + col;
      //   // console.log(row, col);
      //   // to do this the right way, convert index to frequency
      //   // just like the zoom issue, it is easier if you make functions to transform between data and visual axis
      //   // if data coordinate is fractional, do simple average

      //   // image_data[i] = (row * col) / 100;
      //   // image_data[i + 1] = i*i/100000000;
      //   // image_data[i + 2] = 0;
      //   // image_data[i + 3] = 255;

      //   // if (col > width / 2 && row < height / 2) {
      //   //   image_data[i] = 255;
      //   //   image_data[i + 1] = 0;
      //   //   image_data[i + 2] = 255;
      //   //   image_data[i + 3] = 255;
      //   // } else {
      //   //   image_data[i] = 255;
      //   //   image_data[i + 1] = 255;
      //   //   image_data[i + 2] = 255;
      //   //   image_data[i + 3] = 255;
      //   // }
      //   if (i > 100) {
      //     image_data[i] = 255;
      //     image_data[i + 1] = 0;
      //     image_data[i + 2] = 255;
      //     image_data[i + 3] = 255;
      //   } else {
      //     image_data[i] = 255;
      //     image_data[i + 1] = 255;
      //     image_data[i + 2] = 255;
      //     image_data[i + 3] = 255;
      //   }
      //   if ((i / 4) % width == width - 1) {
      //     row += 1;
      //     col = 0;
      //   }
      //   col += 1;
      // }

      var row = 0;
      var col = 0;
      let index = 0;
      let amp = 0;
      let int = 0;
      let precise = 0;
      let remainder = 0;

      let scalar = 255;

      let used_rows: Array<number> = [];
      if (T % 1 === 0) {
        for (let i = 0; i < L; i += 4) {
          const r = Math.floor(loglin(row + 1, 1, height)) - 1;

          if (used_rows.includes(r)) {
            continue;
          }
          used_rows.push(r);
          precise = col * fftsize + r;
          int = Math.round(precise + remainder);
          remainder = precise % 1;

          let x = Math.abs(data[1][int]);
          if (!isNaN(x)) {
            amp = Math.log10(x + 1e-6) * scalar;
          }

          image_data[i] += amp * color.r;
          image_data[i + 1] += amp * color.g;
          image_data[i + 2] += amp * color.b;
          image_data[i + 3] += 255;
          col += 1;
          if (col === width) {
            row += 1;
            col = 0;
          }
        }
      } else {
        for (let i = 0; i < L; i += 4) {
          // if (used_rows.includes(row)) {

          // row += 1;
          // col = 0;
          // continue;
          // }
          // used_rows.push(row);
          // for mel spectrum, convert row to freq then convert to mel then back to row?
          // const freq = row/height * 44100 / (fftsize*4)
          // const m = mel(freq)
          // const r = Math.floor(m);
          const r = Math.floor(linlog(row + 1, 1, height)) - 1;
          // console.log(used_rows);

          // if (used_rows.includes(r)) {
          //   continue;
          // }
          precise = col * fftsize + r;
          int = Math.round(precise + remainder);
          remainder = precise % 1;

          let x = Math.abs(data[1][int]);
          if (!isNaN(x)) {
            amp = Math.log10(x + 1e-6) * scalar;
          }

          image_data[i] += amp * color.r;
          image_data[i + 1] += amp * color.g;
          image_data[i + 2] += amp * color.b;
          image_data[i + 3] += 255;
          col += 1;
          if (col === width) {
            row += 1;
            col = 0;
            // used_rows.push(r);
          }
        }
      }

      ctx.putImageData(image, 0, 0);
    };
    id = requestAnimationFrame(renderPlot);

    return () => {
      renderPlot = () => {};
      cancelAnimationFrame(id);
    };
  }

  $: selectedRecording, getData();
</script>

<div>
  <canvas id="freq_canvas" />
  <div>
    <canvas id="time_canvas" />
    <div style="width: 100%;text-align: right; font-size: 10px;">
      {#if time_labels.length > 1}
        <span>{time_labels[1]}</span>
      {/if}
      <button
        on:click={() => {
          invoke("play", { name: selectedRecording }).then(() => {});
        }}>play</button
      >
    </div>
  </div>
  <div class="indicator" style="margin-left:{indicator_position}px" />
</div>

<style>
  .indicator {
    width: 5px;
    height: 403px;
    position: absolute;
    top: 5px;
    background: rgba(200, 100, 0, 0.5);
  }
  canvas {
    width: 600px;
    height: 200px;
    background: black;
  }
  #freq_canvas {
    transform: scale(1, -1);
    margin-top: 5px;
  }
  div {
    user-select: none;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  button {
    width: max-content;
    align-self: center;
    position: absolute;
  }
</style>
