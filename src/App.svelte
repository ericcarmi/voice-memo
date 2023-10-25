<script lang="ts">
  import Recordings from "./lib/Recordings.svelte";
  import TimePlot from "./lib/TimePlot.svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";
  import { readDir, BaseDirectory } from "@tauri-apps/api/fs";
  import type { Recording } from "./lib/types.svelte";
  import Menu from "./lib/Menu.svelte";


  let prefix = ""
  let counter = 0;
  let recordings: Record<string, Recording> = {};
  let uid = 0;
  let entries: any;
  let data: Array<number> = [];
  onMount(async () => {
    entries = await readDir("assets", {
      dir: BaseDirectory.Resource,
      recursive: true,
    });
    for (const entry of entries) {
      if (entry["name"].includes(".wav") && !entry["name"].includes("input")) {
        let meta: string = await invoke("file_metadata", {
          path: entry["path"],
        });
        // console.log(entry, meta);

        recordings[entry["name"]] = {
          created: meta,
          uid: uid,
        };
        uid += 1;
      }
    }
  });
  let selectedRecording = "";
  let isDragging = false;

  // Object.entries(recordings).sort(function (a, b) {
  //   let x = new Date(b[1].created);
  //   let y = new Date(a[1].created);
  //   return x - y;
  // });
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
  <div style='display:flex; flex-direction: column;'>
    <Menu bind:counter={counter} bind:prefix={prefix}/>
    <Recordings {recordings} {uid} bind:selectedRecording  bind:counter bind:prefix />
    </div>
    <TimePlot bind:selectedRecording bind:isDragging />
  </div>
</main>

<style>
  div {
    user-select: none;
  }
</style>
