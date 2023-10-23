<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";
  import { readDir, BaseDirectory, renameFile } from "@tauri-apps/api/fs";
  import type { Recording } from "./types.svelte";

  export let recordings: Record<string, Recording>;
  export let uid: number;

  async function updateFileName(oldname: string, newname: string) {
    console.log(oldname);
    console.log(newname);

    let r = await renameFile(
      "assets/" + oldname + ".wav",
      "assets/" + newname + ".wav",
      {
        dir: BaseDirectory.Resource,
      }
    );
    // console.log(r)
  }

  export let selectedRecording: string;

  let isRecording = false;

  async function record() {
    if (isRecording) return;
    isRecording = true;
    let date = new Date().toISOString();
    let s = date.split("T");
    const fname = s[0] + "--" + s[1].split(".")[0] + ".wav";
    recordings[uid + ".wav"] = { created: date, uid: uid };
    await invoke("record", {
      name: uid + ".wav",
    });
    uid += 1;
  }

  async function stopRecording() {
    isRecording = false;
    await invoke("stop_recording", {
      name: "stop",
    });
  }
  let tempName = "";
  let oldName = "";
</script>

<div class="wrapper">
  <p>recordings</p>
  <div class="list">
    {#each Object.entries(recordings) as recording, i}
      <div
        class="recording"
        data-attribute={selectedRecording === recording[0]}
        on:keydown={() => {}}
        role="button"
        tabindex={0}
        on:click={() => (selectedRecording = recording[0])}
      >
        <input
          disabled={selectedRecording !== recording[0]}
          class="filename"
          value={recording[0].replace(".wav","")}
          on:focus={(e) => {
            oldName = e.currentTarget.value;
          }}
          on:keydown={(e) => {
            if (e.key === "Escape") {
              tempName = oldName;
            } else if (e.key === "Enter") {
              tempName = e.currentTarget.value;
              updateFileName(oldName, tempName);
            } else {
              tempName = e.currentTarget.value;
            }
          }}
          on:change={(e) => {
            // tempName = e.currentTarget.value;
          }}
          on:blur={(e) => {
            // blur is equivalent to pressing enter? or escape? big difference, not sure which one to use
              updateFileName(oldName, tempName);
          }}
        />
        <span>{recording[1]["created"].split('T')[0] + " " + recording[1].created.split('T')[1].split('.')[0]}</span>
      </div>
    {/each}
  </div>
  <button on:click={() => (isRecording ? stopRecording() : record())}
    >{isRecording ? "stop" : "record"}</button
  >
</div>

<style>
  .wrapper {
    width: 200px;
    border: 1px solid #333333;
  }
  .list {
    height: 300px;
    overflow-y: scroll;
  }
  .recording {
    background: #333333;
    border: 1px solid rgb(100, 100, 100);
    transition: border 0.15s;
  }
  .recording:hover {
    border: 1px solid rgb(200, 200, 200);
  }
  .recording[data-attribute="true"] {
    border: 1px solid rgb(200, 200, 200);
  }
  .filename {
    border: none;
    outline: none;
    height: auto;
    background: #333333;
    color: white;
    font-size: 14px;
  }
  span {
    width: 100%;
    flex-grow: 1;
    font-size: 12px;
    color: rgb(0,200,0);
  }
</style>
