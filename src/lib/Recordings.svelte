<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { BaseDirectory, renameFile } from "@tauri-apps/api/fs";
  import type { Recording } from "./types.svelte";

  export let recordings: Record<string, Recording>;
  export let uid: number;
  export let sortedRecordings: Array<Array<string>> = [];

  export let prefix = "";
  export let counter = 0;

  async function updateFileName(oldname: string, newname: string) {
    // console.log(oldname);
    // console.log(newname);

    let r = await renameFile("assets/" + oldname, "assets/" + newname, {
      dir: BaseDirectory.Resource,
    });
    // console.log(r)
  }

  export let selectedRecording: string;

  let isRecording = false;

  async function record() {
    if (isRecording) return;
    isRecording = true;
    let date = new Date().toISOString();
    // let s = date.split("T");
    // const fname = s[0] + "--" + s[1].split(".")[0] + ".wav";
    let fname;
    if (prefix === "") {
      fname = uid + ".wav";
    } else {
      fname = prefix + counter + ".wav";
      counter += 1;
    }

    recordings[fname] = { created: date, uid: uid };
    // add newest to front
    sortedRecordings = [[fname, date], ...sortedRecordings];

    await invoke("record", {
      name: fname,
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
  <p>recordings({sortedRecordings.length})</p>
  <div class="list">
    {#each sortedRecordings as recording}
      <div
        class="recording"
        data-attribute={selectedRecording === recording[0]}
        on:keydown={() => {}}
        role="button"
        tabindex={1}
        on:click={() => {
          selectedRecording = recording[0];
        }}
      >
        <input
          class="filename"
          value={recording[0].slice(0, -4)}
          on:focus={(e) => {
            oldName = e.currentTarget.value;
            console.log(oldName);
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
          on:blur={() => {
            updateFileName(oldName, tempName);
          }}
        />
        <span
          >{recording[1].split("T")[0] +
            " " +
            recording[1].split("T")[1].split(".")[0]}</span
        >
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
    overflow-y: scroll;
    height: 250px;
    z-index: 1;
  }
  .recording {
    background: #333333;
    border: 1px solid rgb(100, 100, 100);
    cursor: pointer;
    z-index: 1;
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
    z-index: 0;
  }
  span {
    width: 100%;
    flex-grow: 1;
    font-size: 12px;
    color: rgb(0, 200, 0);
  }
</style>
