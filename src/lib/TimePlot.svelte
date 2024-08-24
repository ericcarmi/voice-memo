<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";
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
  let height = 180;

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
    freqcanvas.width = width;
    freqcanvas.height = height;

    ctx = freqcanvas.getContext("2d", { willReadFrequently: true });
  });

  async function getWavData() {
    if (selectedRecording === "") return;

    let data: any = await invoke("get_stft_data", {
      fileName: selectedRecording,
    });

    let id = 0;
    let renderPlot = () => {
      const fftsize = 512;
      // setting the width here will scale it properly, much easier than dealing with fractional indices (even more complicated due to stft resampling to match image data structure)
      freqcanvas.width = data[1].length / fftsize;

      const T = width / (data[1].length / fftsize);

      line = new WebglLine(new ColorRGBA(1, 0.5, 0, 1), data[0].length);
      webglp.removeAllLines();
      webglp.addLine(line);
      line.arrangeX();
      var t = 0;
      for (let i = 0; i < data[0].length; i++) {
        // t = Math.round(i * T)

        line.setY(i, data[0][i] * 1);
      }
      webglp.update();
      ctx = freqcanvas.getContext("2d", { willReadFrequently: true });

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
      let int2 = 0;
      let precise = 0;
      let remainder = 0;

      let scalar = 255;
      if (T % 1 === 0) {
        for (let i = 0; i < L; i += 4) {
          const r = Math.floor(loglin(row + 1, 1, height * 20));
          index = col * fftsize + r;
          let x = data[1][index];
          if (!isNaN(x)) {
            amp = Math.log10(x + 1e-6) * scalar;
          }

          col += 1;

          image_data[i] = amp;
          image_data[i + 1] = amp / 2;
          image_data[i + 2] = 0;
          image_data[i + 3] = 255;
          if (col === width) {
            row += 1;
            col = 0;
          }
        }
      } else {
        for (let i = 0; i < L; i += 4) {
          const r = Math.floor(linlog(row + 1, 1, height));
          precise = col * fftsize + r;
          int = Math.round(precise + remainder);
          remainder = precise % 1;
          // int2 = Math.ceil(col * fftsize + row);
          // console.log(int, int2)

          let x = data[1][int];
          if (!isNaN(x)) {
            amp = Math.log10(x + 1e-6) * scalar;
          }

          image_data[i] += amp;
          image_data[i + 1] += amp / 2;
          image_data[i + 2] = 0;
          image_data[i + 3] = 255;
          col += 1;
          if (col === width) {
            row += 1;
            col = 0;
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

  $: selectedRecording, getWavData();
</script>

<div>
  <canvas id="freq_canvas" />
  <canvas id="time_canvas" />
  <!--
  <div
    id="drag-container"
    role="button"
    tabindex={0}
    on:mousedown={() => (isDragging = true)}
    on:mouseup={() => (isDragging = false)}
  >
    <div
      id="draggable"
      role="button"
      tabindex={0}
      on:mousedown={() => (isDragging = true)}
      on:mouseup={() => (isDragging = false)}
    />
  </div>
-->
</div>

<style>
  canvas {
    width: 600px;
    height: 180px;
    border: 1px solid rgb(40, 40, 40);
  }
  #freq_canvas {
    transform: scale(1, -1);
  }
  div {
    user-select: none;
  }
</style>
