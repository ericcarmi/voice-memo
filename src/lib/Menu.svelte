<script lang="ts">
	export let prefix = "";
	export let counter = 0;
</script>

<div class='outer'>
	<div class="wrapper">
		<span>prefix</span>
		<input
			bind:value={prefix}
			title="file prefix"
			on:change={(e) => {
				prefix = e.currentTarget.value;
			}}
		/>
	</div>

	<div class="wrapper">
		<span>counter</span>
		<input
			placeholder="enter a prefix"
			bind:value={counter}
			title="next number to append"
			on:change={(e) => {
				prefix = e.currentTarget.value;
			}}
		/>
	</div>
</div>

<style>
	.outer {
		display: flex;
	}
	.wrapper {
		height: 50px;
		font-size: 12px;
		display: flex;
		flex-direction: column;
	}
	input {
		background: #333333;
		width: 80%;
		align-self: center;
		text-align: center;
		color: white;
	}
</style>
