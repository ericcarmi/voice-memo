<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";
  import { WebglPlot, WebglLine, ColorRGBA } from "webgl-plot";
  // import { loglin, linlog } from "./types.svelte";

  function canvas2stft(
    fft_size: number,
    time_segments: number,
    row: number,
    col: number,
  ) {
    // needs to hop around 1 dimensional data
    return col * fft_size + row;
  }

  let webglp: WebglPlot;
  let line: WebglLine;
  export let selectedRecording: string;

  let canvasMain: any;
  let ctx: CanvasRenderingContext2D;
  let freqcanvas: any;
  export const freq = 440;
  export const amp = 1;

  let width = 400;
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
    // freqcanvas.width = freqcanvas.clientWidth * devicePixelRatio;
    // freqcanvas.height = freqcanvas.clientHeight * devicePixelRatio;
    freqcanvas.width = width;
    freqcanvas.height = height;

    // freqwebglp = new WebglPlot(freqcanvas);

    ctx = freqcanvas.getContext("2d", { willReadFrequently: true });
  });

  async function getWavData() {
    console.log(selectedRecording);

    let data: any = await invoke("get_stft_data", {
      fileName: selectedRecording,
    });

    let id = 0;
    let renderPlot = () => {
      line = new WebglLine(new ColorRGBA(1, 0, 0, 1), data[0].length);

      webglp.removeAllLines();
      webglp.addLine(line);
      line.arrangeX();
      for (let i = 0; i < data[0].length; i++) {
        line.setY(i, data[0][i] * 1);
      }
      webglp.update();
      ctx = freqcanvas.getContext("2d", { willReadFrequently: true });

      const fftsize = 1024;
      // number of time slices not necessarily equal to canvas width
      const T = width / (data[1].length / fftsize);
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

      //   if (col > width / 2 && row < height / 2) {
      //     image_data[i] = 255;
      // image_data[i] = (row * col) / 100;
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
      console.log(T);

      var row = 0;
      var col = 0;
      let index = 0;
      let amp = 0;
      let frac = 0;
      let int = 0;
      let scalar = 1000;
      for (let i = 0; i < L; i += 4) {
        // row = i * height + col;
        // console.log(row, col);
        // to do this the right way, convert index to frequency
        // just like the zoom issue, it is easier if you make functions to transform between data and visual axis
        // if data coordinate is fractional, do simple average
        // not just a simple conversion from matrix to matrix
        // first get row/col of canvas, then convert to 1d representation of stft

        // index = canvas2stft(fftsize, T, row, col);
        // index = 0;
        // console.log(index, fftsize, row, col, data[1][index]);

        col += 1;
        index = (col * fftsize) / T + row;
        frac = index % 1;
        int = (col * fftsize) / Math.floor(T) + row;

        if (frac !== 0) {
          amp =
            data[1][int + 1] * scalar * frac +
            data[1][int] * scalar * (1 - frac);
        } else {
          amp = data[1][index] * scalar;
        }

        image_data[i] = amp;
        image_data[i + 1] = 0;
        image_data[i + 2] = 0;
        image_data[i + 3] = 255;
        if ((i / 4) % width == width - 1) {
          row += 1;
          col = 0;
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
    width: 400px;
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
