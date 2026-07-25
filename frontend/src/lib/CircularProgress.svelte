<script lang="ts">
  // Same construction as Zed's agent-panel context ring (crates/ui/src/components/progress/circular_progress.rs):
  // a track circle plus a foreground arc swept clockwise from 12 o'clock via stroke-dasharray.
  let {
    progress,
    size = 16,
    strokeWidth = 2,
    color = "var(--text-faint)",
    track = "var(--border)",
  }: { progress: number; size?: number; strokeWidth?: number; color?: string; track?: string } = $props();

  const clamped = $derived(Math.max(0, Math.min(1, progress)));
  const radius = $derived((size - strokeWidth) / 2);
  const circumference = $derived(2 * Math.PI * radius);
  const dashoffset = $derived(circumference * (1 - clamped));
</script>

<svg width={size} height={size} viewBox="0 0 {size} {size}" aria-hidden="true">
  <circle cx={size / 2} cy={size / 2} r={radius} fill="none" stroke={track} stroke-width={strokeWidth} />
  <circle
    cx={size / 2}
    cy={size / 2}
    r={radius}
    fill="none"
    stroke={color}
    stroke-width={strokeWidth}
    stroke-dasharray={circumference}
    stroke-dashoffset={dashoffset}
    transform="rotate(-90 {size / 2} {size / 2})"
  />
</svg>
