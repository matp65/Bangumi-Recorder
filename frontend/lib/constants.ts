export const STATUS_OPTIONS = [
  { value: 0, label: "想看" },
  { value: 1, label: "在看" },
  { value: 2, label: "看过" },
  { value: 3, label: "搁置" },
  { value: 4, label: "抛弃" },
] as const;

export const STATUS_LABELS: Record<number, string> = Object.fromEntries(STATUS_OPTIONS.map((item) => [item.value, item.label]));

export const TYPE_LABELS: Record<number, string> = {
  1: "TV", 2: "剧场版", 3: "OVA", 4: "ONA", 5: "TV SP", 6: "音乐", 7: "书籍", 8: "其他", 9: "游戏", 10: "三次元",
};

export function statusVariant(status: number | null | undefined): "default" | "success" | "warning" | "destructive" | "outline" {
  if (status === 1) return "default";
  if (status === 2) return "success";
  if (status === 3) return "warning";
  if (status === 4) return "destructive";
  return "outline";
}
