<script lang="ts">
  import Recordings from "./lib/recordings.svelte";
  import TimePlot from "./lib/plot.svelte";
  import type { Recording } from "./lib/types.svelte";

  let recordings: Record<string, Recording> = {};
  let sortedRecordings: Array<Array<string>> = [];
  let uid = 0;
  let selectedRecording = "";
  let isDragging = false;
</script>

<main class="container">
  <div
    style="display:flex"
    role="button"
    tabindex={0}
    on:mousemove={(e) => {
      if (isDragging) {
        e.preventDefault(); // need this to not change to text cursor
        let ele = document.getElementById("draggable");
        ele && (ele.style.left = Math.max(0, e.clientX - 190) + "px");
      }
    }}
    on:mouseup={() => (isDragging = false)}
  >
    <div style="display:flex; flex-direction: column; width:240px">
      <Recordings
        {uid}
        {recordings}
        bind:sortedRecordings
        bind:selectedRecording
      />
    </div>
    <TimePlot bind:selectedRecording />
  </div>
</main>

<style>
  div {
    user-select: none;
  }
</style>
