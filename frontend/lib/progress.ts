export interface ParsedProgress { episode: number | null; time: string | null }

export function parseProgress(value: string | null | undefined): ParsedProgress {
  if (!value) return { episode: null, time: null };
  const [episodeText, time] = value.split("|", 2);
  const episode = Number.parseInt(episodeText ?? "", 10);
  return { episode: Number.isFinite(episode) ? episode : null, time: time || null };
}

export function buildProgress(episode: number, time?: string) {
  return time ? `${episode}|${time}` : String(episode);
}

export function progressPercent(value: string | null | undefined, total: number) {
  const { episode } = parseProgress(value);
  if (!episode || total <= 0) return 0;
  return Math.min(100, Math.max(0, Math.round((episode / total) * 100)));
}

export function formatSeconds(seconds: number | null | undefined) {
  if (!seconds || seconds <= 0) return "0:00";
  const total = Math.floor(seconds);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const remainder = total % 60;
  return hours > 0 ? `${hours}:${String(minutes).padStart(2, "0")}:${String(remainder).padStart(2, "0")}` : `${minutes}:${String(remainder).padStart(2, "0")}`;
}
