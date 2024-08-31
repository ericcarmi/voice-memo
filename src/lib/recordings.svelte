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

  let page = 1;
  let num_pages = 1;
  let items_per_page = 7;

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

    num_pages = Math.ceil(sortedRecordings.length / items_per_page);
  });

  let displayed_recordings: Array<Array<string>> = [];
  $: sortedRecordings,
    (displayed_recordings = sortedRecordings.slice(
      items_per_page * (page - 1),
      items_per_page * page,
    ));

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
    sortedRecordings = [[fname], ...sortedRecordings];
    num_pages = Math.ceil(sortedRecordings.length / items_per_page);
  }
  let tempName = "";
  let oldName = "";
</script>

<div class="wrapper">
  <button on:click={() => (isRecording ? stopRecording() : record())}
    >{isRecording ? "stop" : "record"}</button
  >
  <br />
  <span
    >recordings({sortedRecordings.length}) -- {page} of {num_pages}
    <button
      class="pager"
      on:click={() => {
        page = Math.max(1, page - 1);
      }}>{"<"}</button
    ><button
      class="pager"
      on:click={() => {
        page = Math.min(num_pages, page + 1);
      }}>{">"}</button
    >
  </span>

  <div class="list">
    {#each displayed_recordings as recording}
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
            if (selectedRecording !== tempName + ".wav" && tempName !== "") {
              updateFileName(oldName, tempName);
            }
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
</div>

<style>
  .wrapper {
    width: 200px;
    margin-left: 1em;
  }
  .list {
    height: 350px;
  }
  .recording {
    background: var(--sepia4);
    cursor: pointer;
    position: relative;
    z-index: 5;
  }
  .recording * {
    color: var(--sepia1);
  }
  .recording:hover {
    background: var(--sepia5);
  }
  .recording:hover * {
    background: var(--sepia5);
  }
  .recording[data-attribute="true"] {
    background: var(--sepia5);
  }
  .recording[data-attribute="true"] * {
    background: var(--sepia5);
  }
  .filename {
    height: auto;
    font-size: 14px;
    position: relative;
  }
  span {
    width: 100%;
    flex-grow: 1;
    font-size: 12px;
  }

  button {
    margin: 0.5em 0;
  }

  .pager {
    padding: 0 4px;
    margin: 1px;
  }
</style>
