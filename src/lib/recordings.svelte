<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import type { Recording } from "./types.svelte";
  import { path } from "@tauri-apps/api";
  import { onMount } from "svelte";

  export let recordings: Record<string, Recording>;
  export let uid: number;
  export let sortedRecordings: Array<Array<string>> = [];
  let last_prefix = "";
  let prefix = "";
  let counter = 0;

  let page = 1;
  let num_pages = 1;
  let items_per_page = 10;
  let trim = false;

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

    num_pages = Math.max(
      1,
      Math.ceil(sortedRecordings.length / items_per_page),
    );
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
  let date: string;

  async function record() {
    if (isRecording) return;
    isRecording = true;
    date = new Date().toISOString();

    if (last_prefix !== prefix) {
      counter = 0;
    }

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
    last_prefix = prefix;

    await invoke("record", {
      name: fname,
    });
    uid += 1;
  }

  async function stopRecording() {
    isRecording = false;

    await invoke("stop_recording", {
      name: fname,
      trim: Boolean(trim),
    });
    selectedRecording = fname;
    // sortedRecordings = [[fname, date], ...sortedRecordings];
    num_pages = Math.ceil(sortedRecordings.length / items_per_page);
  }
  let tempName = "";
  let oldName = "";
</script>

<div class="wrapper">
  <div class="menu">
    <div class="menu-wrapper">
      <span>prefix</span>
      <input
        autocomplete="off"
        autocorrect="off"
        autocapitalize="off"
        spellcheck="false"
        bind:value={prefix}
        title="file prefix"
        on:change={(e) => {
          prefix = e.currentTarget.value;
        }}
      />
    </div>

    <div class="menu-wrapper">
      <span>counter</span>
      <input
        placeholder="enter a number"
        value={counter}
        title="next number to append"
        on:change={(e) => {
          prefix = e.currentTarget.value;
        }}
      />
    </div>
  </div>

  <div style="display:flex; flex-direction: column;">
    <label
      style="font-size:12px;margin-right: 1em; align-self: center; display: flex;"
    >
      <input type="checkbox" bind:value={trim} style="align-self: center;" />
      trim
    </label>
    <button on:click={() => (isRecording ? stopRecording() : record())}
      >{isRecording ? "stop" : "record"}</button
    >
  </div>
  <br />
  <span
    >({sortedRecordings.length}) --- {page} of {num_pages}
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
        title={recording[1].split("T")[0] +
          " " +
          recording[1].split("T")[1].split(".")[0]}
      >
        <input
          autocomplete="off"
          autocorrect="off"
          autocapitalize="off"
          spellcheck="false"
          class="filename"
          data-attribute={recording[0] === selectedRecording}
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
            if (selectedRecording !== tempName + ".wav" && tempName !== "") {
              updateFileName(oldName, tempName);
            }
          }}
        />
      </div>
    {/each}
  </div>
</div>

<style>
  .wrapper {
    width: 200px;
    margin-left: 20px;
  }
  .list {
    margin-top: 7px;
  }
  .recording {
    background: var(--sepia4);
    cursor: pointer;
    position: relative;
    z-index: 5;
    height: 25px;
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

  input[data-attribute="false"] {
    border: none;
  }

  .pager {
    padding: 0 4px;
    margin: 2px;
  }

  .menu-wrapper {
    height: 50px;
    font-size: 12px;
    display: flex;
    flex-direction: column;
  }
  .menu {
    display: flex;
  }
  .menu * input {
    width: 80%;
    align-self: center;
    text-align: center;
  }
</style>
