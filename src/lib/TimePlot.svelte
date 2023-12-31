<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";
  import { WebglPlot, WebglLine, ColorRGBA, WebglSquare } from "webgl-plot";
  import { loglin, linlog } from "./types.svelte";

  let webglp: WebglPlot;
  let line: WebglLine;
  export let selectedRecording: string;
  let squ = new WebglSquare(new ColorRGBA(1, 0, 0, 1));

  let freqwebglp: WebglPlot;
  let freqline: WebglLine;

  let canvasMain: any;
  let ctx: any;
  let freqcanvas: any;
  export const freq = 420;
  export const amp = 1;

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
    freqcanvas.width = 400;
    freqcanvas.height = 200;

    freqwebglp = new WebglPlot(freqcanvas);

    ctx = freqcanvas.getContext("2d");
  });

  async function getData() {
    let data: any = await invoke("get_stft_data", { path: selectedRecording });
    console.log(data);

    let id = 0;
    let renderPlot = () => {
      line = new WebglLine(new ColorRGBA(1, 0, 0, 1), data[0].length);

      webglp.removeAllLines();
      webglp.addLine(line);
      line.arrangeX();
      for (let i = 0; i < data[0].length; i+=32) {
        line.setY(i, data[0][i] * 1);
      }
      webglp.update();

      let stft = data[1];
      let freqs = [];
      let fftsize = stft[0].length;

      let twidth = 2 / stft.length;
      let fwidth = 2 / fftsize;
      let vertices: any = [];
      let colors: Array<number> = [];
      // console.log(stft.length * fftsize, freqcanvas.width, freqcanvas.height);
      let freqhop = 20;

      for (let t = 0; t < stft.length; t += 1) {
        for (let freq = 0; freq < fftsize; freq += freqhop) {
          // freqs.push(stft[t][freq]);

          let x1 = -1.0 + twidth * t;
          let y1 = -1.0 + fwidth * freq;
          let x2 = -1.0 + twidth * (t + 1);
          let y2 = -1.0 + fwidth * freq;
          let x3 = -1.0 + twidth * t;
          let y3 = -1.0 + (fwidth + freqhop) * (freq + 1);
          let x4 = -1.0 + twidth * (t + 1);
          let y4 = -1.0 + (fwidth + freqhop) * (freq + 1);
          vertices.push(
            x1,
            y1,
            0.0,
            x2,
            y2,
            0.0,
            x3,
            y3,
            0.0,
            x2,
            y2,
            0.0,
            x3,
            y3,
            0.0,
            x4,
            y4,
            0.0
          );
          let amp = 0.0;
          for (let k = 0; k < freqhop; k++) {
            amp += Math.log10(stft[t][freq + k] + 1);
          }

          if (isNaN(amp)) {
            amp = 1.0;
          }
          colors = colors.concat(Array(18).fill(amp / freqhop*1));
        }
      }
      // console.log("colors", colors);

      const gl = freqcanvas.getContext("webgl");

      // var vertices = [
      //   -1.0, -1.0, 0.0,
      //   -1.0, -0.7, 0.0,
      //   -0.7, -1.0, 0.0,
      //   -1.0, -0.7, 0.0,
      //   -0.7, -1.0, 0.0,
      //   -0.7, -0.7, 0.0,
      // ];
      // var colors = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];

      // var indices = [2, 2, 3];

      // Create an empty buffer object to store the vertex buffer
      var vertex_buffer = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, vertex_buffer);
      gl.bufferData(
        gl.ARRAY_BUFFER,
        new Float32Array(vertices),
        gl.STATIC_DRAW
      );
      gl.bindBuffer(gl.ARRAY_BUFFER, null);

      // Create an empty buffer object and store Index data
      // var Index_Buffer = gl.createBuffer();
      // gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, Index_Buffer);
      // gl.bufferData(
      //   gl.ELEMENT_ARRAY_BUFFER,
      //   new Uint16Array(indices),
      //   gl.STATIC_DRAW
      // );
      // gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, null);

      // Create an empty buffer object and store color data
      var color_buffer = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, color_buffer);
      gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(colors), gl.STATIC_DRAW);

      var vertCode =
        "attribute vec3 coordinates;" +
        "attribute vec3 color;" +
        "varying vec3 vColor;" +
        "void main(void) {" +
        "gl_Position = vec4(coordinates, 1.0);" +
        "vColor = color;" +
        "}";

      var vertShader = gl.createShader(gl.VERTEX_SHADER);
      gl.shaderSource(vertShader, vertCode);
      gl.compileShader(vertShader);

      var fragCode =
        "precision mediump float;" +
        "varying vec3 vColor;" +
        "void main(void) {" +
        "gl_FragColor = vec4(vColor, 1.);" +
        "}";

      var fragShader = gl.createShader(gl.FRAGMENT_SHADER);

      gl.shaderSource(fragShader, fragCode);
      gl.compileShader(fragShader);
      var shaderProgram = gl.createProgram();
      gl.attachShader(shaderProgram, vertShader);
      gl.attachShader(shaderProgram, fragShader);
      gl.linkProgram(shaderProgram);
      gl.useProgram(shaderProgram);
      gl.bindBuffer(gl.ARRAY_BUFFER, vertex_buffer);

      var coord = gl.getAttribLocation(shaderProgram, "coordinates");

      gl.vertexAttribPointer(coord, 3, gl.FLOAT, false, 0, 0);
      gl.enableVertexAttribArray(coord);
      gl.bindBuffer(gl.ARRAY_BUFFER, color_buffer);
      var color = gl.getAttribLocation(shaderProgram, "color");
      gl.vertexAttribPointer(color, 3, gl.FLOAT, false, 0, 0);
      gl.enableVertexAttribArray(color);
      gl.clearColor(0, 0, 0, 1);
      // gl.enable(gl.DEPTH_TEST);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.viewport(0, 0, freqcanvas.width, freqcanvas.height);

      gl.drawArrays(gl.TRIANGLES, 0, vertices.length / 3);
    };
    id = requestAnimationFrame(renderPlot);

    return () => {
      renderPlot = () => {};
      cancelAnimationFrame(id);
    };
  }

  async function getWavData() {
    let data: any = await invoke("get_wav_data", { path: selectedRecording });
    console.log(data);

    let id = 0;
    let renderPlot = () => {
      line = new WebglLine(new ColorRGBA(1, 0, 0, 1), data[0].length);

      webglp.removeAllLines();
      webglp.addLine(line);
      line.arrangeX();
      for (let i = 0; i < data[0].length; i++) {
        line.setY(i, data[0][i] * 1);
      }

      let fftsize = data[1].length;
      freqline = new WebglLine(new ColorRGBA(1, 0, 0, 1), data[1].length);
      freqwebglp.removeAllLines();
      freqwebglp.addLine(freqline);
      freqline.arrangeX();
      let minfreq = 20.0;
      let maxfreq = 20000.0;
      let fr = 44100.0 / fftsize;

      let id = [];
      for (let i = 1; i < fftsize; i++) {
      let y = linlog(i, minfreq, maxfreq);
        freqline.setY(
          i,
          Math.log10(data[1][i] + 0.001) * 0.3 - 0.4
        );
        // freqline.setY(i, Math.log10(data[1][i] + 0.001)*0.3 - 0.4);
        // freqline.setY(loglin(i, minfreq, maxfreq), data[1][i]*0.1);

        id.push(linlog(i, minfreq, maxfreq));
      }

      console.log(id, id[1] - id[0], id[id.length-1], id.length);

      webglp.update();
      freqwebglp.update();
    };
    id = requestAnimationFrame(renderPlot);
    console.log(Math.max(data[1]));

    return () => {
      renderPlot = () => {};
      cancelAnimationFrame(id);
    };
  }

  $: selectedRecording, getData();
  // $: selectedRecording, getWavData();

  export let isDragging = false;
  let timePosition = 0;
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
    border: 2px solid rgb(0, 100, 0);
  }
  #drag-container {
    background: #222222;
    height: 20px;
    display: flex;
    cursor: pointer;
  }
  #draggable {
    background: rgb(0, 100, 0);
    width: 10px;
    height: 20px;
    position: relative;
    cursor: pointer;
  }
  div {
    user-select: none;
  }
</style>
