<script lang="ts">
  import Recordings from "./lib/recordings.svelte";
  import TimePlot from "./lib/plot.svelte";
  import type { Recording } from "./lib/types.svelte";
  import Menu from "./lib/menu.svelte";

  let prefix = "";
  let counter = 0;
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
    <div style="display:flex; flex-direction: column;">
      <Menu bind:counter bind:prefix />
      <Recordings
        {uid}
        {recordings}
        bind:sortedRecordings
        bind:selectedRecording
        bind:counter
        bind:prefix
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
