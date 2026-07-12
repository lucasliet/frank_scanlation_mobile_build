<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { isMobile } from "$lib/platform";

  let { children } = $props();

  onMount(() => {
    if (isMobile()) {
      document.documentElement.classList.add("mobile-layout");
    }
  });
</script>

{@render children()}
