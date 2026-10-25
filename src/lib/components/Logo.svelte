<script lang="ts">
  import logo from "../assets/framewise-logo.png";

  let { size = 26, radius = 6 }: { size?: number; radius?: number } = $props();
</script>

<img
  src={logo}
  width={size}
  height={size}
  alt=""
  class="mark"
  style:border-radius="{radius}px"
/>

<style>
  .mark {
    display: block;
    flex: 0 0 auto;
    object-fit: contain;
    animation: logo-settle 620ms cubic-bezier(0.22, 1, 0.36, 1) both;
    transition: transform 200ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  .mark:hover {
    transform: rotate(6deg) scale(1.06);
  }

  @keyframes logo-settle {
    0% {
      opacity: 0;
      transform: scale(0.82) rotate(-10deg);
    }
    60% {
      opacity: 1;
    }
    100% {
      opacity: 1;
      transform: scale(1) rotate(0);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .mark {
      animation: none;
    }
  }
</style>
