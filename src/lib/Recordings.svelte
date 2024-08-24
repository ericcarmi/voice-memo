<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import type { Recording } from "./types.svelte";
  import { path } from "@tauri-apps/api";
  import { onMount } from "svelte";

  export let recordings: Record<string, Recording>;
  export let uid: number;
  export let sortedRecordings: Array<Array<string>> = [];

  export let prefix = "";
  export let counter = 0;

  async function get_files() {
    let entries: Array<string> = await invoke("get_wavs");
    sortedRecordings = [];

    let dir = await path.resourceDir();
    if (dir.includes("\\")) {
      dir += "assets\\";
    } else {
      dir += "assets/";
    }

    for (const entry of entries) {
      if (entry.includes(".wav")) {
        let meta: string = await invoke("file_metadata", {
          path: dir + entry,
        });
        sortedRecordings = [[entry, meta], ...sortedRecordings];
        uid += 1;
      }
    }
    sortedRecordings
      .sort(function (a, b) {
        var c = Date.parse(a[1]);
        var d = Date.parse(b[1]);
        return c - d;
      })
      .reverse();
  }

  onMount(async () => {
    await get_files();
  });

  async function updateFileName(oldname: string, newname: string) {
    invoke("rename_file", {
      old: oldname + ".wav",
      new: newname + ".wav",
    })
      .then(() => {
        const index = sortedRecordings.findIndex((itm) => {
          return itm[0] === oldname + ".wav";
        });
        sortedRecordings[index][0] = newname + ".wav";
        selectedRecording = newname + ".wav";
      })
      .catch((e) => {
        console.error(e);
      });
  }

  export let selectedRecording: string;

  let isRecording = false;
  let fname: string;

  async function record() {
    if (isRecording) return;
    isRecording = true;
    let date = new Date().toISOString();
    // let s = date.split("T");
    // const fname = s[0] + "--" + s[1].split(".")[0] + ".wav";
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
    selectedRecording = fname;
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
          autocomplete="off"
          autocorrect="off"
          autocapitalize="off"
          spellcheck="false"
          class="filename"
          value={recording[0].slice(0, -4)}
          style="z-index:{recording[0] === selectedRecording ? 0 : -1}"
          on:focus={(e) => {
            oldName = e.currentTarget.value;
          }}
          on:keydown={(e) => {
            if (e.key === "Escape") {
              tempName = oldName;
            } else if (e.key === "Enter") {
              tempName = e.currentTarget.value;
              updateFileName(oldName, tempName);
              e.currentTarget.blur();
            } else {
              tempName = e.currentTarget.value;
            }
          }}
          on:blur={() => {
            console.log(selectedRecording, tempName);

            selectedRecording !== tempName + ".wav" && tempName !== "";
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
  }
  .recording {
    background: #333333;
    border: 1px solid rgb(100, 100, 100);
    cursor: pointer;
    position: relative;
    z-index: 5;
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
    position: relative;
  }
  span {
    width: 100%;
    flex-grow: 1;
    font-size: 12px;
    color: rgb(0, 200, 0);
  }
</style>
