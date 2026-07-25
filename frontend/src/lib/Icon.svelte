<script lang="ts">
  // Path data lifted from Zed's icon set (assets/icons/*.svg), stroke recolored
  // to currentColor so icons inherit the button's theme color.
  const paths: Record<string, string> = {
    send: "M3.377 3.028a.25.25 0 0 0-.292.045.291.291 0 0 0-.067.303l1.496 4.236c.088.25.088.526 0 .776l-1.496 4.236a.291.291 0 0 0 .067.303.25.25 0 0 0 .292.045l9.472-4.72a.267.267 0 0 0 .11-.103.288.288 0 0 0-.11-.4L3.377 3.028ZM5 8h8",
    plus: "M3.33325 8H12.6666 M8 3.33333V12.6667",
    "chevron-down": "M4.15186 6.47321L7.99258 10.1696L11.8483 6.47321",
    "rotate-ccw": "m3 5.778 1.256-1.256A5.417 5.417 0 0 1 8 3a5 5 0 1 1-4.583 7 M3 3v3h3",
    copy: "M12.486 6.2H7.24795C6.66895 6.2 6.19995 6.669 6.19995 7.248V12.486C6.19995 13.064 6.66895 13.533 7.24795 13.533H12.486C13.064 13.533 13.533 13.064 13.533 12.486V7.248C13.533 6.669 13.064 6.2 12.486 6.2Z M3.91712 10.203C3.63951 10.2022 3.37351 10.0915 3.1773 9.89511C2.98109 9.69872 2.87064 9.43261 2.87012 9.155V3.917C2.87091 3.63956 2.98147 3.37371 3.17765 3.17753C3.37383 2.98135 3.63968 2.87079 3.91712 2.87H9.15512C9.43273 2.87053 9.69883 2.98097 9.89523 3.17718C10.0916 3.37339 10.2023 3.63939 10.2031 3.917",
    check: "M4.625 8.98121L7.03402 10.7714L11.3437 4.75989",
  };

  let { name, size = 14 }: { name: keyof typeof paths; size?: number } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 16 16"
  fill="none"
  stroke="currentColor"
  stroke-width="1.2"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  <path d={paths[name]} />
</svg>
