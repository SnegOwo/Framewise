import { writable } from "svelte/store";

export type Celebration = {
  id: number;
  title: string;
  detail: string;
  tint: string;
};

export const celebration = writable<Celebration | null>(null);

let sequence = 0;

export function celebrate(
  title = "Goal reached",
  detail = "Every minute counted.",
  tint = "var(--accent)",
) {
  celebration.set({ id: ++sequence, title, detail, tint });
}

export function clearCelebration(id?: number) {
  celebration.update((current) => (id != null && current?.id !== id ? current : null));
}
