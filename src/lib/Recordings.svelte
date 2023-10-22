<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { open } from "@tauri-apps/api/dialog";

  let uid = 0;
  let recordings = {"test": uid};

  let isRecording = false;

  async function record() {
    if (isRecording) return;
    isRecording = true;
    let date = new Date().toISOString();
    let s = date.split("T");
    const fname = s[0] + "--" + s[1].split(".")[0] + ".wav";
    uid += 1;
    // recordings = Object.assign(recordings, {fname: uid})
    recordings[fname] = uid

    await invoke("record", {
      name: "assets/" + fname,
    });
  }

  async function stopRecording() {
    isRecording = false;
    console.log("stop");
    await invoke("stop_recording", {
      name: "stop",
    });
  }
</script>

<div class="wrapper">
  <p>recordings</p>
  <div class="list">
    {#each Object.entries(recordings) as recording}
      <div class="recording">
        <input class="filename" value={recording[0]} />
        created
      </div>
    {/each}
  </div>
  <button on:click={() => (isRecording ? stopRecording() : record())}
    >record</button
  >
</div>

<style>
  .wrapper {
    width: 200px;
    border: 1px solid green;
  }
  .list {
    height: 300px;
    overflow-y: scroll;
  }
  .recording {
    background: #333333;
    border: 1px solid rgb(200, 200, 200);
    transition: border 0.15s;
  }
  .recording:hover {
    border: 1px solid rgb(100, 100, 100);
  }
  .filename {
    border: none;
    outline: none;
    height: auto;
    background: #333333;
    color: white;
    font-size: 14px;
  }
</style>
