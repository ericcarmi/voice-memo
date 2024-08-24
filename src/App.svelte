<script lang="ts">
  import Recordings from "./lib/Recordings.svelte";
  import TimePlot from "./lib/TimePlot.svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";
  import type { Recording } from "./lib/types.svelte";
  import Menu from "./lib/Menu.svelte";
  import { path } from "@tauri-apps/api";

  let prefix = "";
  let counter = 0;
  let recordings: Record<string, Recording> = {};
  let sortedRecordings: Array<Array<string>> = [];
  let uid = 0;
  let entries: any;
  let selectedRecording = "";
  let isDragging = false;
  let rename_flag = false;


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
        {rename_flag}
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
