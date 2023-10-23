<script lang="ts">
  import Recordings from "./lib/Recordings.svelte";
  import TimePlot from "./lib/TimePlot.svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";
  import { readDir, BaseDirectory } from "@tauri-apps/api/fs";
  import type { Recording } from "./lib/types.svelte";

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
        console.log(entry, meta);

        recordings[entry["name"]] = {
          created: meta,
          uid: uid,
        };
        uid += 1;
      }
    }
  });
  let selectedRecording = "";

  // Object.entries(recordings).sort(function (a, b) {
  //   let x = new Date(b[1].created);
  //   let y = new Date(a[1].created);
  //   return x - y;
  // });
</script>

<main class="container">
  <Recordings {recordings} {uid} bind:selectedRecording={selectedRecording} />
  <TimePlot bind:data={data} bind:selectedRecording={selectedRecording}/>
</main>

<style>
</style>
